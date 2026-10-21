//! HIR to MIR: the flow graph, with every check site, drop, lend and
//! failure edge written out as an instruction.

use std::collections::HashMap;

use crate::ast::{BinOp, Tier, UnOp, Verb};
use crate::check::{Instance, Program as Checked};
use crate::hir::{self, Builtin, FnInst};
use crate::source::Span;
use crate::types::{Bound, IntW, ParamKind, Ty};

use super::domain::Domain;
use super::*;

pub fn build(checked: Checked) -> Program {
    // Every const worked out once, in order, to the literal it is — so each
    // use is that literal, and Precog knows it exactly.
    let mut consts: Vec<Option<Const>> = Vec::new();
    for v in &checked.const_values {
        let c = const_eval(v, &consts, &checked.items);
        consts.push(c);
    }
    let mut b = Arena { functions: Vec::new(), inst_idx: HashMap::new(), instances: checked.instances, items: checked.items, subst: checked.subst, consts };
    // Every instance with a body gets its index first, so a call can name
    // one that has not been lowered yet.
    let mut order = Vec::new();
    for i in 0..b.instances.len() {
        if b.instances[i].body.is_some() {
            let idx = FnIdx(b.functions.len() as u32);
            b.inst_idx.insert(FnInst(i as u32), idx);
            // The slot is taken now: a task body lowered while this one is
            // being built appends after it, and every index stays what a
            // call already wrote down.
            let body = b.instances[i].body.clone().unwrap();
            b.functions.push(placeholder(&body));
            order.push((FnInst(i as u32), idx));
        }
    }
    for (inst, idx) in &order {
        let f = lower_instance(&mut b, *inst);
        b.functions[idx.0 as usize] = f;
    }
    let main = checked.main.and_then(|m| b.inst_idx.get(&m).copied());
    let mut p = Program { functions: b.functions, main, items: b.items, instances: b.instances, tables: Vec::new() };
    // Precog's half of `closed` (owner, 2026-09-23): the maker lets go of a
    // channel where it provably never names it again.
    crate::mir::channels::release_early(&mut p);
    nocheat_clones(&mut p);
    drop_unreached(&mut p);
    p
}

/// The calls a `no_cheat` statement makes go to copies of their functions
/// that really do the work (owner, 2026-10-18: "(b)", calls and recursion
/// included), and so do the calls those copies make, all the way down: a
/// function called both ways keeps its ordinary body for the ordinary
/// calls. A copy's loops are `no_cheat` loops.
fn nocheat_clones(p: &mut Program) {
    let mut copy_of: HashMap<usize, usize> = HashMap::new();
    let mut work: Vec<(usize, Option<Vec<u32>>)> = (0..p.functions.len()).filter(|&f| !p.functions[f].nocheat_blocks.is_empty()).map(|f| (f, Some(p.functions[f].nocheat_blocks.clone()))).collect();
    // The copy of `idx`, made the first time it is asked for.
    fn copy(p: &mut Program, idx: usize, copy_of: &mut HashMap<usize, usize>, work: &mut Vec<(usize, Option<Vec<u32>>)>) -> usize {
        if p.functions[idx].nocheat {
            return idx;
        }
        if let Some(c) = copy_of.get(&idx) {
            return *c;
        }
        let mut g = p.functions[idx].clone();
        g.nocheat = true;
        g.nocheat_of = Some(FnIdx(idx as u32));
        g.is_main = false;
        g.is_test = false;
        g.nocheat_blocks.clear();
        g.nocheat_entries.clear();
        for fl in &mut g.fors {
            fl.nocheat = true;
        }
        let c = p.functions.len();
        p.functions.push(g);
        copy_of.insert(idx, c);
        work.push((c, None));
        c
    }
    // A call through a held function (owner, 2026-10-19: "(a)") cannot be
    // pointed at a copy here: which function it holds is known as it runs.
    // Every function the program holds gets a copy, and the generated code
    // calls the copy of the one held (`codegen`, `nocheat_held`).
    let mut held = false;
    let mut all_held = false;
    loop {
        while let Some((f, only)) = work.pop() {
            let blocks: Vec<usize> = match &only {
                Some(bs) => bs.iter().map(|b| *b as usize).collect(),
                None => (0..p.functions[f].blocks.len()).collect(),
            };
            let mut tasks: HashMap<u32, usize> = HashMap::new();
            for b in blocks {
                // A task started inside runs no_cheat too (owner, 2026-10-19).
                for si in 0..p.functions[f].blocks[b].stmts.len() {
                    let StmtKind::Task { body, id, .. } = &p.functions[f].blocks[b].stmts[si].kind else { continue };
                    let (idx, id) = (body.0 as usize, *id);
                    let c = copy(p, idx, &mut copy_of, &mut work);
                    tasks.insert(id, c);
                    if let StmtKind::Task { body, .. } = &mut p.functions[f].blocks[b].stmts[si].kind {
                        *body = FnIdx(c as u32);
                    }
                }
                match &p.functions[f].blocks[b].term {
                    Terminator::Call { callee: Callee::Value(_), .. } => held = true,
                    Terminator::Call { callee: Callee::Fn(idx), .. } => {
                        let c = copy(p, idx.0 as usize, &mut copy_of, &mut work);
                        if let Terminator::Call { callee: Callee::Fn(i), .. } = &mut p.functions[f].blocks[b].term {
                            *i = FnIdx(c as u32);
                        }
                    }
                    _ => {}
                }
            }
            // A started task's failure is read where its block ends, which
            // may be past the statement: by the copy's reasons, the same.
            if !tasks.is_empty() {
                for b in &mut p.functions[f].blocks {
                    if let Terminator::TaskOutcome { id, body, .. } = &mut b.term {
                        if let Some(c) = tasks.get(id) {
                            *body = FnIdx(*c as u32);
                        }
                    }
                }
            }
        }
        if !held || all_held {
            break;
        }
        all_held = true;
        let mut named: Vec<usize> = Vec::new();
        for g in &p.functions {
            for b in &g.blocks {
                for s in &b.stmts {
                    match &s.kind {
                        StmtKind::Assign { rvalue: Rvalue::FnRef(i), .. } => named.push(i.0 as usize),
                        StmtKind::Assign { rvalue: Rvalue::Closure { body, .. }, .. } => named.push(body.0 as usize),
                        _ => {}
                    }
                }
            }
        }
        named.sort_unstable();
        named.dedup();
        for idx in named {
            copy(p, idx, &mut copy_of, &mut work);
        }
    }
}

/// A function of std no start of the program reaches — `MAIN`, a `test
/// fn`, and whatever they call, hold, or start as a task — is emptied: no
/// body, no sites, so nothing is analysed, checked or generated for it. A file with
/// no CyJet.cyr gets all of std (owner, 2026-10-06), and its every function
/// was being proven and compiled into every program. A kit with no `MAIN`
/// is a library, and a board names its own functions: both keep everything.
fn drop_unreached(p: &mut Program) {
    let Some(main) = p.main else { return };
    if p.items.board_output.is_some() || p.items.board_exit.is_some() {
        return;
    }
    let n = p.functions.len();
    // Every way a body names a function — a call, a held function, a
    // closure, a task and its outcome — prints as `FnIdx(n)`.
    let named = |f: &Function| -> Vec<usize> {
        let text = format!("{:?}", f.blocks);
        text.match_indices("FnIdx(").filter_map(|(at, w)| text[at + w.len()..].split(')').next().and_then(|d| d.parse().ok())).collect()
    };
    // A function reached is reached in its `no_cheat` copy too: a call
    // through a held function names the original, and the generated code
    // calls the copy (`nocheat_clones`).
    let mut copies: HashMap<usize, Vec<usize>> = HashMap::new();
    for (c, g) in p.functions.iter().enumerate() {
        if let Some(o) = g.nocheat_of {
            copies.entry(o.0 as usize).or_default().push(c);
        }
    }
    let mut seen = vec![false; n];
    let mut work: Vec<usize> = std::iter::once(main.0 as usize).chain((0..n).filter(|&f| p.functions[f].is_test)).collect();
    while let Some(f) = work.pop() {
        if f >= n || std::mem::replace(&mut seen[f], true) {
            continue;
        }
        work.extend(named(&p.functions[f]));
        work.extend(copies.get(&f).into_iter().flatten().copied());
    }
    // Only std's: a program's own functions can be reached in ways no body
    // names them (a key type's `hash` and `equal_to`, found by the map code
    // itself), and they were all compiled before std was given to every file.
    let std_root = p.items.nerve_cord;
    let of_std: Vec<bool> = p.functions.iter().map(|f| {
        let Some(root) = std_root else { return false };
        f.inst.and_then(|i| p.instances.get(i.0 as usize)).and_then(|i| p.items.fns.get(i.key.fn_id.0 as usize)).is_some_and(|d| p.items.kit_root(d.module) == root)
    }).collect();
    for (f, func) in p.functions.iter_mut().enumerate() {
        if seen[f] || !of_std[f] {
            continue;
        }
        let span = func.blocks.first().map(|b| b.term_span).unwrap_or(SpanId(0));
        func.dropped = true;
        func.blocks = vec![BasicBlock { stmts: Vec::new(), term: Terminator::Unreachable, term_span: span }];
        func.sites.clear();
        func.fors.clear();
        func.ranges.clear();
        func.operand_ranges.clear();
    }
}

pub struct Arena {
    pub functions: Vec<Function>,
    inst_idx: HashMap<FnInst, FnIdx>,
    pub instances: Vec<Instance>,
    pub items: crate::resolve::Items,
    /// The checker's answers for the types it had to work out.
    pub subst: HashMap<crate::types::InferId, Ty>,
    /// Each const's value, when it is a literal the program can hold.
    pub consts: Vec<Option<Const>>,
}

/// A const's value at compile time (`functions.md`, consts): literals,
/// exact whole-number arithmetic (each `div.X` and `mod.X` as named),
/// comparisons, `f64` arithmetic, joined texts, and earlier consts. Any
/// other shape (a list, a call) stays a reference.
fn const_eval(e: &hir::Expr, done: &[Option<Const>], items: &crate::resolve::Items) -> Option<Const> {
    use crate::ast::{BinOp as B, UnOp as U};
    let _ = items;
    let is_f64 = |e: &hir::Expr| matches!(e.ty.base(), Ty::Bin(crate::types::BinW::Fixed(64)));
    match &e.kind {
        hir::ExprKind::Int(v) => Some(Const::Int(*v)),
        hir::ExprKind::BigInt(v) => Some(Const::Big(v.clone())),
        hir::ExprKind::Bin(s) => Some(Const::Bin(s.clone())),
        hir::ExprKind::Str(s) => Some(Const::Str(s.clone())),
        hir::ExprKind::Char(c) => Some(Const::Char(c.clone())),
        hir::ExprKind::Bool(b) => Some(Const::Bool(*b)),
        hir::ExprKind::Duration(n) => Some(Const::Duration(*n)),
        hir::ExprKind::Const(id) => done.get(id.0 as usize).cloned().flatten(),
        // A file's bytes (owner, 2026-10-10).
        hir::ExprKind::Call { callee: hir::Callee::Builtin(hir::Builtin::InclBytes), args, .. } => match args.first() {
            Some(hir::Arg::Value(hir::Expr { kind: hir::ExprKind::Str(hex), .. })) => Some(Const::Bytes((0..hex.len() / 2).filter_map(|i| u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok()).collect())),
            _ => None,
        },
        // An array of literals, each worked out.
        hir::ExprKind::List(items_) if matches!(e.ty.base(), Ty::Seq { kind: crate::types::SeqKind::Array, .. }) => {
            let Ty::Seq { elem, .. } = e.ty.base() else { unreachable!() };
            let mut out = Vec::new();
            for a in items_ {
                let hir::Arg::Value(x) = a else { return None };
                match const_eval(x, done, items)? {
                    c @ (Const::Int(_) | Const::Bool(_) | Const::Char(_) | Const::Bin(_)) => out.push(c),
                    _ => return None,
                }
            }
            Some(Const::Array { elem: (**elem).clone(), items: out })
        }
        hir::ExprKind::Unary { op, expr } => match (op, const_eval(expr, done, items)?) {
            (U::Neg, Const::Int(v)) => Some(Const::Int(v.checked_neg()?)),
            // `-'x'` of a zero is `0`, `negate.ieee` IEEE's `-0` (owner, 2026-10-07).
            (U::Neg, Const::Bin(s)) if is_f64(e) => Some(Const::Bin(format!("{:?}", -s.replace('_', "").parse::<f64>().ok()? + 0.0))),
            (U::NegIeee, Const::Bin(s)) if is_f64(e) => Some(Const::Bin(format!("{:?}", -s.replace('_', "").parse::<f64>().ok()?))),
            (U::Not, Const::Bool(b)) => Some(Const::Bool(!b)),
            _ => None,
        },
        hir::ExprKind::Binary { op, lhs, rhs, .. } => {
            let (a, b) = (const_eval(lhs, done, items)?, const_eval(rhs, done, items)?);
            match (a, b) {
                (Const::Int(x), Const::Int(y)) => {
                    Some(match op {
                        B::Add => Const::Int(x.checked_add(y)?),
                        B::Sub => Const::Int(x.checked_sub(y)?),
                        B::Mul => Const::Int(x.checked_mul(y)?),
                        B::Div(r) => Const::Int(crate::rounding::int_divmod(x, y, *r)?.0),
                        B::Mod(r) => Const::Int(crate::rounding::int_divmod(x, y, *r)?.1),
                        B::Pow if y >= 0 => Const::Int(x.checked_pow(u32::try_from(y).ok()?)?),
                        B::Eq => Const::Bool(x == y),
                        B::Ne => Const::Bool(x != y),
                        B::Lt => Const::Bool(x < y),
                        B::Le => Const::Bool(x <= y),
                        B::Gt => Const::Bool(x > y),
                        B::Ge => Const::Bool(x >= y),
                        _ => return None,
                    })
                }
                // Only at `f64`, where the host's double is the target's
                // (`reproducible-builds.md`: never round at the host's width).
                (Const::Bin(x), Const::Bin(y)) if is_f64(e) => {
                    let (x, y) = (x.replace('_', "").parse::<f64>().ok()?, y.replace('_', "").parse::<f64>().ok()?);
                    // No result is `-0` but a `.ieee` form's (owner,
                    // 2026-10-07: "`a`"); `+ 0.0` turns one into `0`.
                    let v = match op {
                        B::AddIeee => x + y,
                        B::SubIeee => x - y,
                        B::MulIeee => x * y,
                        B::DivIeee(r) => crate::rounding::bin_divmod(x, y, *r).0,
                        B::ModIeee(r) if *r != crate::ast::Round::Near => crate::rounding::bin_divmod(x, y, *r).1,
                        B::Add => x + y + 0.0,
                        B::Sub => x - y + 0.0,
                        B::Mul => x * y + 0.0,
                        B::Div(r) => crate::rounding::bin_divmod(x, y, *r).0 + 0.0,
                        B::Mod(r) if *r != crate::ast::Round::Near => crate::rounding::bin_divmod(x, y, *r).1 + 0.0,
                        _ => return None,
                    };
                    // No infinity or NaN is a bin (owner, 2026-09-29): left
                    // to run, where its site says so.
                    if !v.is_finite() {
                        return None;
                    }
                    Some(Const::Bin(format!("{:?}", v)))
                }
                (Const::Str(x), Const::Str(y)) if *op == B::Join => Some(Const::Str(x + &y)),
                (Const::Bool(x), Const::Bool(y)) => match op {
                    B::And => Some(Const::Bool(x && y)),
                    B::Or => Some(Const::Bool(x || y)),
                    B::Eq => Some(Const::Bool(x == y)),
                    B::Ne => Some(Const::Bool(x != y)),
                    _ => None,
                },
                _ => None,
            }
        }
        _ => None,
    }
}

/// A type as it finally is: inference filled in, a `simd`'s lane count
/// taken from the target's shape for its lane type.
pub fn resolved(t: &Ty, subst: &HashMap<crate::types::InferId, Ty>) -> Ty {
    match t {
        Ty::Infer(id) => match subst.get(id) {
            Some(b) => resolved(b, subst),
            None => t.clone(),
        },
        // A whole number a use pinned, or one nothing did: a bare one.
        Ty::Int(crate::types::IntW::Free(id)) if id.0 != 0 => match subst.get(id) {
            Some(b) => resolved(b, subst),
            None => Ty::Int(crate::types::IntW::Free(crate::types::InferId(0))),
        },
        Ty::Seq { elem, kind } => Ty::Seq { elem: Box::new(resolved(elem, subst)), kind: *kind },
        Ty::Channel(e) => Ty::Channel(Box::new(resolved(e, subst))),
        Ty::Ptr { mutable, elem } => Ty::Ptr { mutable: *mutable, elem: Box::new(resolved(elem, subst)) },
        Ty::Map { key, val, order } => Ty::Map { key: Box::new(resolved(key, subst)), val: Box::new(resolved(val, subst)), order: *order },
        Ty::Simd { elem, lanes } => {
            let e = resolved(elem, subst);
            let n = if *lanes == 0 { crate::types::simd_final_lanes(&e) } else { *lanes };
            Ty::Simd { elem: Box::new(e), lanes: n }
        }
        Ty::Struct { id, args } => Ty::Struct { id: *id, args: args.iter().map(|a| resolved(a, subst)).collect() },
        Ty::Enum { id, args } => Ty::Enum { id: *id, args: args.iter().map(|a| resolved(a, subst)).collect() },
        Ty::Fn(f) => Ty::Fn(Box::new(crate::types::FnTy {
            params: f.params.iter().map(|(k, t)| (*k, resolved(t, subst))).collect(),
            ret: resolved(&f.ret, subst),
            fails: f.fails.iter().map(|(n, t)| (n.clone(), t.as_ref().map(|t| resolved(t, subst)))).collect(),
            owning: f.owning,
        })),
        Ty::Bundle(ts) => Ty::Bundle(ts.iter().map(|t| resolved(t, subst)).collect()),
        Ty::Bounded { inner, bound } => Ty::Bounded { inner: Box::new(resolved(inner, subst)), bound: bound.clone() },
        Ty::Percent(inner) => Ty::Percent(Box::new(resolved(inner, subst))),
        Ty::Places { inner, max, min } => Ty::Places { inner: Box::new(resolved(inner, subst)), max: *max, min: *min },
        t => t.clone(),
    }
}

fn lower_instance(arena: &mut Arena, inst: FnInst) -> Function {
    let body = arena.instances[inst.0 as usize].body.clone().unwrap();
    let mut fb = FnBuilder::new(arena, Some(inst), &body, false);
    fb.lower_body(&body);
    fb.finish(&body)
}

/// One function being built.
struct FnBuilder<'a> {
    arena: &'a mut Arena,
    inst: Option<FnInst>,
    locals: Vec<LocalDecl>,
    blocks: Vec<BasicBlock>,
    sites: Vec<Site>,
    spans: Vec<Span>,
    span_index: HashMap<(u32, u32, u32), SpanId>,
    cur: BlockId,
    /// Scopes of owned locals, innermost last; a scope's end drops them.
    scopes: Vec<Vec<LocalId>>,
    /// `break` and `continue` targets, with the scope depth to drop to.
    loops: Vec<LoopTargets>,
    /// The tier covering the statement being lowered.
    tier: Option<Tier>,
    /// The blocks a `concur` started in this scope; joined at its end.
    tasks_in_scope: Vec<usize>,
    is_task: bool,
    /// A task body's captures, as locals.
    captures: Vec<LocalId>,
    /// The checker's local table: a task's body shares its parent's.
    hir_locals: Vec<hir::Local>,
    /// Per scope, for each task started in it that can fail: which task,
    /// its body, the arms of its `otherwise`, and the block its handler
    /// falls out of — which cannot know where to go until the wait it
    /// belongs to exists, so it is patched when the block ends.
    task_fails: Vec<Vec<(u32, FnIdx, Vec<FailArm>, BlockId)>>,
    /// Every `for` lowered so far (`ForLoop`).
    fors: Vec<ForLoop>,
    /// The next loop built is a `for !cheat`.
    nocheat_next: bool,
    /// Inside a `no_cheat` statement (owner, 2026-10-18), how deep.
    nocheat_depth: u32,
    /// The blocks written inside the `no_cheat` statement being lowered.
    nocheat_region: std::collections::BTreeSet<u32>,
    /// Every block written inside a `no_cheat` statement.
    nocheat_blocks: std::collections::BTreeSet<u32>,
    /// Where each `no_cheat` statement starts, and the locals it touches.
    nocheat_entries: Vec<(u32, u32, Vec<LocalId>)>,
    /// Named locals declared with `:=`, whose type Precog picked.
    picked: Vec<LocalId>,
    /// Every `check`, `no_check` and `NO_FUCKING_CHECKS` written.
    markers: Vec<Marker>,
    /// Per scope: the group its tasks join, made when the first one starts.
    task_group: Vec<Option<u32>>,
    next_group: u32,
    next_task: u32,
    /// The slot a `return` fills.
    ret_local: Option<LocalId>,
    /// A detached task's `otherwise`, to be built inside the task.
    catch_otherwise: Option<(hir::Otherwise, Vec<(String, Option<Ty>)>)>,
    /// Where a failure goes instead of out, once that handler is built.
    catch: Option<Vec<FailArm>>,
    /// The span of the instruction most recently lowered, for a terminator
    /// that has no span of its own.
    last_span: SpanId,
}

#[derive(Clone, Copy)]
struct LoopTargets {
    brk: BlockId,
    cont: BlockId,
    depth: usize,
}

impl<'a> FnBuilder<'a> {
    fn new(arena: &'a mut Arena, inst: Option<FnInst>, body: &hir::Body, is_task: bool) -> FnBuilder<'a> {
        let mut fb = FnBuilder {
            arena,
            inst,
            locals: Vec::new(),
            blocks: vec![BasicBlock { stmts: Vec::new(), term: Terminator::Unreachable, term_span: SpanId(0) }],
            sites: Vec::new(),
            spans: Vec::new(),
            span_index: HashMap::new(),
            cur: BlockId(0),
            scopes: vec![Vec::new()],
            loops: Vec::new(),
            tier: None,
            tasks_in_scope: vec![0],
            is_task,
            captures: Vec::new(),
            hir_locals: body.locals.clone(),
            task_fails: vec![Vec::new()],
            fors: Vec::new(),
            nocheat_next: false,
            nocheat_depth: 0,
            nocheat_region: Default::default(),
            nocheat_blocks: Default::default(),
            nocheat_entries: Vec::new(),
            picked: Vec::new(),
            markers: Vec::new(),
            task_group: vec![None],
            next_group: 0,
            next_task: 0,
            ret_local: None,
            catch_otherwise: None,
            catch: None,
            last_span: SpanId(0),
        };
        // One MIR local per HIR local, in the same order, so indices line up.
        for l in &body.locals {
            let span = fb.span_id(l.span);
            let kind = match l.kind {
                hir::LocalKind::Lent | hir::LocalKind::LoopLent => LocalKind::Lent { mutable: false },
                hir::LocalKind::ConstView => LocalKind::Plain,
                hir::LocalKind::LentMut => LocalKind::Lent { mutable: true },
                _ if l.ty.is_owned() => LocalKind::Owned,
                _ => LocalKind::Plain,
            };
            let ty = resolved(&l.ty, &fb.arena.subst);
            fb.locals.push(LocalDecl { name: Some(l.name.clone()), ty: ty.clone(), domain: Domain::of_ty(&ty, &fb.arena.items), kind, span });
        }
        if body.ret != Ty::Nothing {
            let span = fb.span_id(body.span);
            let id = LocalId(fb.locals.len() as u32);
            let ret = resolved(&body.ret, &fb.arena.subst);
            // A borrow handed back is held as one: an address.
            let kind = match &body.ret_lent {
                Some((m, _)) => LocalKind::Lent { mutable: *m },
                None => LocalKind::Return,
            };
            fb.locals.push(LocalDecl { name: Some("the result".into()), ty: ret.clone(), domain: Domain::of_ty(&ret, &fb.arena.items), kind, span });
            fb.ret_local = Some(id);
        }
        // An owned parameter is the function's to give back when it ends,
        // like any value it owns (unless it gave it away first): nothing
        // freed a list handed over with `give`, and every such call leaked.
        for p in &body.params {
            if matches!(fb.locals[p.0 as usize].kind, LocalKind::Owned) {
                fb.scopes[0].push(LocalId(p.0));
            }
        }
        fb
    }

    // ------------------------------------------------------------ plumbing

    fn span_id(&mut self, span: Span) -> SpanId {
        let key = (span.file.0, span.start, span.end);
        if let Some(id) = self.span_index.get(&key) {
            return *id;
        }
        let id = SpanId(self.spans.len() as u32);
        self.spans.push(span);
        self.span_index.insert(key, id);
        id
    }

    fn new_block(&mut self) -> BlockId {
        let id = BlockId(self.blocks.len() as u32);
        self.blocks.push(BasicBlock { stmts: Vec::new(), term: Terminator::Unreachable, term_span: SpanId(0) });
        id
    }

    fn push(&mut self, kind: StmtKind, span: SpanId) {
        self.last_span = span;
        if self.nocheat_depth > 0 {
            self.nocheat_region.insert(self.cur.0);
        }
        self.blocks[self.cur.0 as usize].stmts.push(Statement { kind, span });
    }

    fn terminate(&mut self, term: Terminator) {
        let span = self.last_span;
        if self.nocheat_depth > 0 {
            self.nocheat_region.insert(self.cur.0);
        }
        let b = &mut self.blocks[self.cur.0 as usize];
        if matches!(b.term, Terminator::Unreachable) {
            b.term = term;
            b.term_span = span;
        }
    }

    /// Terminate with the span this instruction came from.
    fn terminate_at(&mut self, term: Terminator, span: SpanId) {
        if self.nocheat_depth > 0 {
            self.nocheat_region.insert(self.cur.0);
        }
        let b = &mut self.blocks[self.cur.0 as usize];
        if matches!(b.term, Terminator::Unreachable) {
            b.term = term;
            b.term_span = span;
        }
    }

    /// Has the current block already left?
    fn closed(&self) -> bool {
        !matches!(self.blocks[self.cur.0 as usize].term, Terminator::Unreachable)
    }

    fn temp(&mut self, ty: &Ty, span: SpanId) -> LocalId {
        let ty = &self.resolve_ty(ty);
        let kind = if ty.is_owned() { LocalKind::Owned } else { LocalKind::Temp };
        let id = LocalId(self.locals.len() as u32);
        self.locals.push(LocalDecl { name: None, ty: ty.clone(), domain: Domain::of_ty(ty, &self.arena.items), kind, span });
        // A temporary holding an owned value is dropped with its scope.
        if kind == LocalKind::Owned {
            self.scopes.last_mut().unwrap().push(id);
        }
        id
    }

    fn site(&mut self, kind: SiteKind, operands: Vec<Operand>, span: SpanId) -> SiteId {
        self.site_of(kind, operands, None, span)
    }

    /// A site whose question is about a type: what the value has to fit.
    fn site_of(&mut self, kind: SiteKind, operands: Vec<Operand>, result_ty: Option<Ty>, span: SpanId) -> SiteId {
        let id = SiteId(self.sites.len() as u32);
        self.sites.push(Site { kind, tier: self.tier, result_ty, span, operands, verdict: Verdict::Unproven });
        self.push(StmtKind::Site(id), span);
        id
    }

    fn assign(&mut self, place: Place, rvalue: Rvalue, span: SpanId) {
        // A list or map written out in full: each element going into the
        // element type is a store like any other, so a narrower one is a
        // check site (as a struct's fields are). Without it
        // `let 'xs': list[i8] = [1, 300];` kept 44.
        let dest_ty = {
            let base = self.locals[place.local.0 as usize].ty.clone();
            self.resolve_ty(&projected_ty(&base, &place.proj, &self.arena.items))
        };
        match (&rvalue, dest_ty.base()) {
            // A width not written (a `:=` list of bare numbers) holds them all.
            (Rvalue::MakeList(_), Ty::Seq { elem, .. }) if matches!(elem.base(), Ty::Int(IntW::Free(_))) || elem.is_exact() => {}
            (Rvalue::MakeList(ops), Ty::Seq { elem, .. }) => {
                let elem = (**elem).clone();
                for op in ops.clone() {
                    let have = self.operand_ty(&op);
                    self.narrow_arg(&op, &have, &elem, span);
                }
            }
            (Rvalue::MakeMap(pairs), Ty::Map { key, val, .. }) => {
                let (key, val) = ((**key).clone(), (**val).clone());
                for (k, v) in pairs.clone() {
                    let (hk, hv) = (self.operand_ty(&k), self.operand_ty(&v));
                    self.narrow_arg(&k, &hk, &key, span);
                    self.narrow_arg(&v, &hv, &val, span);
                }
            }
            _ => {}
        }
        // Writing over a name that owns something gives the old value
        // back first. `'text' = copy 'text' ++ "ab"` in a loop would
        // otherwise leave every round's text behind — the right-hand side
        // is already in a temporary of its own by the time this runs, so
        // there is nothing here that still needs the old one.
        //
        // The first assignment to a name has nothing to give back; the
        // lowering knows that from the flag it keeps per owning local,
        // and the drop costs nothing where the answer never changes.
        if place.proj.is_empty() && matches!(self.locals[place.local.0 as usize].kind, LocalKind::Owned) {
            let ty = self.locals[place.local.0 as usize].ty.clone();
            if ty.is_owned() {
                self.push(StmtKind::Drop(place.clone()), span);
            }
        }
        self.push(StmtKind::Assign { place, rvalue }, span);
    }

    fn into_temp(&mut self, rvalue: Rvalue, ty: &Ty, span: SpanId) -> Operand {
        let t = self.temp(ty, span);
        // A temporary holding a borrow is a view, not a value to free: its
        // scope ends the lend instead of dropping it.
        if let Rvalue::Lend { mutable, .. } = &rvalue {
            self.locals[t.0 as usize].kind = LocalKind::Lent { mutable: *mutable };
        }
        self.assign(Place::local(t), rvalue, span);
        Operand::Read(Place::local(t))
    }

    fn push_scope(&mut self) {
        self.scopes.push(Vec::new());
        self.tasks_in_scope.push(0);
        self.task_fails.push(Vec::new());
        self.task_group.push(None);
    }

    /// Drop this scope's owned values, newest first, and wait for its tasks.
    fn pop_scope(&mut self, span: SpanId) {
        let tasks = self.tasks_in_scope.pop().unwrap_or(0);
        let fails = self.task_fails.pop().unwrap_or_default();
        let group = self.task_group.pop().flatten();
        let locals = self.scopes.pop().unwrap_or_default();
        if !self.closed() {
            if let (true, Some(g)) = (tasks > 0, group) {
                self.join(g, fails);
            }
            self.drop_locals(&locals, span);
        }
    }

    /// The block waits for everything it started, then asks each task that
    /// could fail how it ended, one after another — so when two fail, both
    /// handlers run, each where its `concur` said.
    fn join(&mut self, group: u32, fails: Vec<(u32, FnIdx, Vec<FailArm>, BlockId)>) {
        let mut next = self.new_block();
        self.terminate(Terminator::Join { group, ok: next });
        for (id, body, arms, after) in fails {
            self.cur = next;
            next = self.new_block();
            self.terminate(Terminator::TaskOutcome { id, body, ok: next, fail: arms });
            self.blocks[after.0 as usize].term = Terminator::Goto(next);
        }
        self.cur = next;
    }

    fn drop_locals(&mut self, locals: &[LocalId], span: SpanId) {
        for l in locals.iter().rev() {
            match self.locals[l.0 as usize].kind {
                LocalKind::Owned => self.push(StmtKind::Drop(Place::local(*l)), span),
                LocalKind::Lent { .. } => self.push(StmtKind::EndLend(Place::local(*l)), span),
                _ => {}
            }
        }
    }

    /// Leaving through `return` / `fail` / `break` / `continue`: every scope
    /// down to `depth` ends here too.
    fn unwind_to(&mut self, depth: usize, span: SpanId) {
        let mut i = self.scopes.len();
        while i > depth {
            i -= 1;
            if let (true, Some(g)) = (self.tasks_in_scope.get(i).copied().unwrap_or(0) > 0, self.task_group.get(i).copied().flatten()) {
                let fails = self.task_fails.get(i).cloned().unwrap_or_default();
                self.join(g, fails);
            }
            let locals = self.scopes[i].clone();
            self.drop_locals(&locals, span);
        }
    }

    fn finish(mut self, body: &hir::Body) -> Function {
        let span = self.span_id(body.span);
        if !self.closed() {
            self.unwind_to(0, span);
            self.terminate(Terminator::Return);
        }
        // Any block left without a terminator fell out of a construct that
        // always leaves; nothing reaches it.
        for b in &mut self.blocks {
            if matches!(b.term, Terminator::Unreachable) && b.stmts.is_empty() {
                b.term = Terminator::Unreachable;
            }
        }
        self.widen_bare_ints();
        let params = body.params.iter().map(|p| LocalId(p.0)).collect();
        Function {
            dropped: false,
            nocheat: false,
            nocheat_of: None,
            nocheat_blocks: self.nocheat_blocks.iter().copied().collect(),
            nocheat_entries: std::mem::take(&mut self.nocheat_entries),
            picked: std::mem::take(&mut self.picked),
            markers: std::mem::take(&mut self.markers),
            name: body.name.clone(),
            inst: self.inst,
            params,
            locals: self.locals,
            blocks: self.blocks,
            ret: resolved(&body.ret, &self.arena.subst),
            ret_local: self.ret_local,
            ret_lent: body.ret_lent.clone(),
            fails: body.fails.clone(),
            sites: self.sites,
            spans: self.spans,
            is_main: body.is_main,
            is_test: body.is_test,
            is_task: self.is_task,
            variadic: self.inst.map_or(false, |i| self.arena.instances[i.0 as usize].key.bundle.is_some()),
            captures: self.captures,
            ranges: Default::default(),
            operand_ranges: Default::default(),
            fors: self.fors,
        }
    }

    /// A whole number nothing gave a type, `let 'a' := 10000000000000000000;`:
    /// `:=` means Precog chooses its width (types.md), and 64 bits cannot
    /// hold what it is given here, so it is given as many as its value
    /// needs, at least 128 — and so is whatever is worked out from it —
    /// rather than the value silently wrapping.
    fn widen_bare_ints(&mut self) {
        // The signed width a constant needs, if more than 64 bits.
        let need = |c: &Const| -> Option<u32> {
            match c {
                Const::Int(v) if i64::try_from(*v).is_err() => Some(128),
                Const::Big(v) => Some((crate::bigint::Big::parse(v)?.bits() as u32 + 1).max(128)),
                _ => None,
            }
        };
        let bare = |t: &Ty| matches!(t, Ty::Int(IntW::Free(_)));
        let mut wide: HashMap<usize, u32> = HashMap::new();
        loop {
            let mut grew = false;
            for b in &self.blocks {
                for st in &b.stmts {
                    let StmtKind::Assign { place, rvalue } = &st.kind else { continue };
                    let l = place.local.0 as usize;
                    if !place.proj.is_empty() || !bare(&self.locals[l].ty) {
                        continue;
                    }
                    let of = |o: &Operand| match o {
                        Operand::Const(c) => need(c),
                        _ => o.place().filter(|p| p.proj.is_empty()).and_then(|p| wide.get(&(p.local.0 as usize)).copied()),
                    };
                    let w = match rvalue {
                        Rvalue::Use(o) | Rvalue::Unary { operand: o, .. } => of(o),
                        Rvalue::Binary { lhs, rhs, .. } => of(lhs).max(of(rhs)),
                        _ => None,
                    };
                    if let Some(w) = w {
                        if wide.get(&l).map_or(true, |had| *had < w) {
                            wide.insert(l, w);
                            grew = true;
                        }
                    }
                }
            }
            if !grew {
                break;
            }
        }
        if wide.is_empty() {
            return;
        }
        for (l, w) in &wide {
            self.locals[*l].ty = Ty::int(true, *w);
            self.locals[*l].domain = Domain::of_ty(&self.locals[*l].ty, &self.arena.items);
        }
        // The site guarding a widened answer asks what the oracle asks of a
        // bare number's arithmetic: that some 64-bit type holds the answer
        // (the exact type at 64 bits). A negated literal is a literal, and
        // asks only that its width holds it.
        for b in &self.blocks {
            for w in b.stmts.windows(2) {
                let (StmtKind::Site(id), StmtKind::Assign { place, rvalue }) = (&w[0].kind, &w[1].kind) else { continue };
                if !place.proj.is_empty() || !wide.contains_key(&(place.local.0 as usize)) {
                    continue;
                }
                let site = &mut self.sites[id.0 as usize];
                if site.result_ty.as_ref().is_some_and(bare) {
                    let literal = matches!(rvalue, Rvalue::Unary { operand: Operand::Const(_), .. });
                    site.result_ty = Some(if literal { self.locals[place.local.0 as usize].ty.clone() } else { Ty::exact() });
                }
            }
        }
    }

    // ------------------------------------------------------------ statements

    fn lower_body(&mut self, body: &hir::Body) {
        // A detached task handles its own failure, in the task, the moment
        // it fails (owner, 2026-09-23): the handler is built here, beside the
        // body, at the depth where the captures still live, so a retry can
        // use them; when it is done the task ends.
        if let Some((o, fails)) = self.catch_otherwise.take() {
            let span = self.span_id(body.span);
            let (arms, after) = self.fail_blocks(&o, &fails, None, span);
            let saved = self.cur;
            self.cur = after;
            self.unwind_to(0, span);
            self.terminate(Terminator::Return);
            self.cur = saved;
            self.catch = Some(arms);
        }
        self.block(&body.block);
    }

    /// Leave by failing: out to the caller, or — in a detached task, which
    /// has no caller to hear it — into the task's own handler.
    fn leave_failing(&mut self, index: usize, payload: Option<Operand>, span: SpanId) {
        if let Some(arm) = self.catch.as_ref().and_then(|arms| arms.iter().find(|a| a.index == index)).cloned() {
            // Everything the body opened goes; the captures stay for the
            // handler.
            self.unwind_to(1, span);
            if let (Some(place), Some(p)) = (&arm.payload, payload) {
                self.assign(place.clone(), Rvalue::Use(p), span);
            }
            self.terminate(Terminator::Goto(arm.block));
            return;
        }
        self.unwind_to(0, span);
        self.terminate(Terminator::Fail { index, payload });
    }

    fn block(&mut self, b: &hir::Block) {
        self.push_scope();
        for s in &b.stmts {
            if self.closed() {
                break;
            }
            self.stmt(s);
        }
        let span = self.span_id(b.span);
        self.pop_scope(span);
    }

    fn stmt(&mut self, s: &hir::Stmt) {
        let span = self.span_id(s.span);
        let saved_tier = self.tier;
        if s.tier.is_some() {
            self.tier = s.tier;
        }
        // A check word written here, not one this statement only sits
        // under: what it covers is listed in the build report.
        let written = s.tier.filter(|t| Some(*t) != saved_tier);
        let at = (self.cur.0, self.blocks.len() as u32, self.sites.len() as u32);
        self.stmt_kind(s, span);
        if let Some(tier) = written {
            self.markers.push(Marker { tier, span, start: at.0, from: at.1, to: self.blocks.len() as u32, end: self.cur.0, sites: (at.2, self.sites.len() as u32) });
        }
        self.tier = saved_tier;
    }

    fn stmt_kind(&mut self, s: &hir::Stmt, span: SpanId) {
        match &s.kind {
            // `no_cheat` (owner, 2026-10-18): its blocks are marked, its
            // loops are `no_cheat` loops, and where it starts the locals it
            // touches pass through a barrier, so nothing in it is worked out
            // beforehand. The calls in it go to no-cheat copies
            // (`nocheat_clones`).
            hir::StmtKind::NoCheat(stmts) => {
                let entry = (self.cur.0, self.blocks[self.cur.0 as usize].stmts.len() as u32);
                let outer = std::mem::take(&mut self.nocheat_region);
                self.nocheat_depth += 1;
                for st in stmts {
                    self.stmt(st);
                }
                self.nocheat_depth -= 1;
                let region = std::mem::replace(&mut self.nocheat_region, outer);
                let mut touched: std::collections::BTreeSet<u32> = Default::default();
                for b in &region {
                    let text = format!("{:?}", self.blocks[*b as usize]);
                    touched.extend(text.match_indices("LocalId(").filter_map(|(at, w)| text[at + w.len()..].split(')').next().and_then(|d| d.parse::<u32>().ok())));
                }
                self.nocheat_entries.push((entry.0, entry.1, touched.into_iter().map(LocalId).collect()));
                self.nocheat_blocks.extend(region.iter().copied());
                if self.nocheat_depth > 0 {
                    self.nocheat_region.extend(region);
                }
            }
            hir::StmtKind::Let { local, value, typed } => {
                if !*typed && self.locals[local.0 as usize].name.is_some() {
                    self.picked.push(LocalId(local.0));
                }
                let place = Place::local(LocalId(local.0));
                let ty = self.locals[local.0 as usize].ty.clone();
                self.store(place.clone(), value, &ty, *typed, span);
                // An owned local is this scope's to drop.
                match self.locals[local.0 as usize].kind {
                    LocalKind::Owned | LocalKind::Lent { .. } => self.scopes.last_mut().unwrap().push(LocalId(local.0)),
                    _ => {}
                }
            }
            hir::StmtKind::Assign { place, value } => {
                let p = self.place(place, span);
                let ty = self.resolve_ty(&place.ty);
                self.store(p, value, &ty, true, span);
            }
            hir::StmtKind::Expr(e) => {
                let _ = self.eval(e, span);
            }
            hir::StmtKind::If { cond, then, else_ } => {
                let c = self.eval(cond, span);
                let then_b = self.new_block();
                let else_b = self.new_block();
                let join = self.new_block();
                self.terminate(Terminator::If { cond: c, then: then_b, else_: else_b });
                self.cur = then_b;
                self.block(then);
                self.terminate(Terminator::Goto(join));
                self.cur = else_b;
                match else_ {
                    Some(e) => self.block(e),
                    None => {}
                }
                self.terminate(Terminator::Goto(join));
                self.cur = join;
            }
            hir::StmtKind::Loop(body) => {
                let head = self.new_block();
                let exit = self.new_block();
                self.terminate(Terminator::Goto(head));
                self.cur = head;
                self.loops.push(LoopTargets { brk: exit, cont: head, depth: self.scopes.len() });
                self.block(body);
                self.terminate(Terminator::Goto(head));
                self.loops.pop();
                self.cur = exit;
            }
            hir::StmtKind::While { cond, body } => {
                let head = self.new_block();
                let body_b = self.new_block();
                let exit = self.new_block();
                self.terminate(Terminator::Goto(head));
                self.cur = head;
                let c = self.eval(cond, span);
                self.terminate(Terminator::If { cond: c, then: body_b, else_: exit });
                self.cur = body_b;
                self.loops.push(LoopTargets { brk: exit, cont: head, depth: self.scopes.len() });
                self.block(body);
                self.terminate(Terminator::Goto(head));
                self.loops.pop();
                self.cur = exit;
            }
            hir::StmtKind::ForRange { counter, start, stop, step, exclusive, body, perm, seq, para, nocheat } => {
                self.nocheat_next = *nocheat;
                self.for_range(*counter, start, stop, step.as_ref(), *exclusive, body, *perm, *seq, *para, span);
            }
            hir::StmtKind::ForEach { elem, pos, iter, body, perm, seq, para, nocheat } => {
                self.nocheat_next = *nocheat;
                self.for_each(*elem, *pos, iter, body, *perm, *seq, *para, span);
            }
            hir::StmtKind::ForNext { elem, call, value_case, none_case, body, perm } => {
                self.for_next(*elem, call, *value_case, *none_case, body, *perm, span);
            }
            hir::StmtKind::Return(v) => {
                match v {
                    Some(e) => {
                        let ret_ty = self.ret_ty();
                        let ret = self.ret_local.unwrap_or_else(|| self.temp(&ret_ty, span));
                        self.store(Place::local(ret), e, &ret_ty, true, span);
                        self.unwind_to(0, span);
                        self.terminate(Terminator::Return);
                    }
                    None => {
                        self.unwind_to(0, span);
                        self.terminate(Terminator::Return);
                    }
                }
            }
            hir::StmtKind::Fail { index, payload, .. } => {
                let p = payload.as_ref().map(|e| self.eval(e, span));
                self.leave_failing(*index, p, span);
            }
            hir::StmtKind::Break => {
                if let Some(t) = self.loops.last().copied() {
                    self.unwind_to(t.depth, span);
                    self.terminate(Terminator::Goto(t.brk));
                }
            }
            hir::StmtKind::Continue => {
                if let Some(t) = self.loops.last().copied() {
                    self.unwind_to(t.depth, span);
                    self.terminate(Terminator::Goto(t.cont));
                }
            }
            hir::StmtKind::Block(b) => self.block(b),
            hir::StmtKind::Match { value, arms } => self.lower_match(value, arms, None, span),
            hir::StmtKind::Concur { detached, captures, body, otherwise, fails } => {
                self.concur(*detached, captures, body, otherwise.as_ref(), fails, span);
            }
            hir::StmtKind::Whichever { arms, otherwise } => {
                self.whichever(arms, otherwise.as_ref(), span);
            }
            hir::StmtKind::InlineAsm { text, ins, outs, lends, clobbers } => {
                let ins = ins.iter().map(|(l, r)| (Place::local(LocalId(l.0)), r.clone())).collect();
                let outs = outs.iter().map(|(l, r)| (Place::local(LocalId(l.0)), r.clone())).collect();
                let lends = lends.iter().map(|(l, m, r)| (Place::local(LocalId(l.0)), *m, r.clone())).collect();
                self.push(StmtKind::Asm { text: text.clone(), ins, outs, lends, clobbers: clobbers.clone() }, span);
            }
        }
    }

    fn ret_ty(&self) -> Ty {
        match self.inst {
            Some(i) => self.arena.instances[i.0 as usize].sig.ret.clone(),
            None => Ty::Nothing,
        }
    }

    /// `for 'i' from a to b by s` — the counter moves by the step and the
    /// body cannot assign to it, so the header alone says the trip count.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::too_many_arguments)]
    fn for_range(&mut self, counter: hir::LocalId, start: &hir::Expr, stop: &hir::Expr, step: Option<&hir::Expr>, exclusive: bool, body: &hir::Block, perm: bool, seq: bool, para: bool, span: SpanId) {
        let c = LocalId(counter.0);
        let ty = self.locals[c.0 as usize].ty.clone();
        // The counter holds every value the range names, so each end of the
        // range is a value that has to fit its type (`control-flow.md`:
        // "`'i': i32` → explicit, every value must fit").
        let s = self.eval(start, span);
        self.site_of(SiteKind::Narrow { to: ty.clone() }, vec![s.clone()], Some(ty.clone()), span);
        self.assign(Place::local(c), Rvalue::Use(s.clone()), span);
        let first_v = s;
        let stop_v = self.eval(stop, span);
        let stop_t = self.temp(&ty, span);
        if !exclusive {
            self.site_of(SiteKind::Narrow { to: ty.clone() }, vec![stop_v.clone()], Some(ty.clone()), span);
            self.assign(Place::local(stop_t), Rvalue::Use(stop_v.clone()), span);
        }
        // "The step is not one of those values" (owner, 2026-09-23): it is
        // its own signed number, so a `u64` counter can count down by
        // -1. It lives in a signed 64-bit temporary, not in the counter's
        // type, where -1 would read as the type's largest value.
        let step_v = match step {
            Some(e) => self.eval(e, span),
            None => Operand::Const(Const::Int(1)),
        };
        let step_ty = Ty::int(true, 64);
        self.site_of(SiteKind::Narrow { to: step_ty.clone() }, vec![step_v.clone()], Some(step_ty.clone()), span);
        let step_t = self.temp(&step_ty, span);
        self.assign(Place::local(step_t), Rvalue::Use(step_v.clone()), span);
        // `before` (owner, 2026-10-12): the end is left out, and is never the
        // counter's value. A range already empty — from 0 before 0 — runs no
        // rounds; any other runs to the value one before the end, which lies
        // between the first value and the end, so the loop below is the
        // inclusive one, unchanged.
        let after = exclusive.then(|| self.new_block());
        if let Some(after) = after {
            let wide = Ty::int(true, 128);
            let end = self.into_temp(Rvalue::Convert { to: wide.clone(), operand: stop_v, useful: false, bits: false }, &wide, span);
            let first = self.into_temp(Rvalue::Convert { to: wide.clone(), operand: Operand::Read(Place::local(c)), useful: false, bits: false }, &wide, span);
            let up = self.into_temp(Rvalue::Binary { op: BinOp::Ge, lhs: Operand::Read(Place::local(step_t)), rhs: Operand::Const(Const::Int(0)) }, &Ty::Bool, span);
            let up_b = self.new_block();
            let down_b = self.new_block();
            let go = self.new_block();
            self.terminate(Terminator::If { cond: up, then: up_b, else_: down_b });
            for (b, op, by) in [(up_b, BinOp::Lt, 1i128), (down_b, BinOp::Gt, -1)] {
                self.cur = b;
                let room = self.into_temp(Rvalue::Binary { op, lhs: first.clone(), rhs: end.clone() }, &Ty::Bool, span);
                let last_b = self.new_block();
                self.terminate(Terminator::If { cond: room, then: last_b, else_: after });
                self.cur = last_b;
                let one = self.into_temp(Rvalue::Convert { to: wide.clone(), operand: Operand::Const(Const::Int(by)), useful: false, bits: false }, &wide, span);
                let last = self.into_temp(Rvalue::Binary { op: BinOp::Sub, lhs: end.clone(), rhs: one }, &wide, span);
                self.site_of(SiteKind::Narrow { to: ty.clone() }, vec![last.clone()], Some(ty.clone()), span);
                self.assign(Place::local(stop_t), Rvalue::Convert { to: ty.clone(), operand: last, useful: false, bits: false }, span);
                self.terminate(Terminator::Goto(go));
            }
            self.cur = go;
        }

        let head = self.new_block();
        let body_b = self.new_block();
        let next = self.new_block();
        let exit = self.new_block();
        let pre = self.cur;
        self.terminate(Terminator::Goto(head));

        // The stop is inclusive, and a negative step counts down.
        self.cur = head;
        let going_up = self.into_temp(
            Rvalue::Binary { op: BinOp::Ge, lhs: Operand::Read(Place::local(step_t)), rhs: Operand::Const(Const::Int(0)) },
            &Ty::Bool,
            span,
        );
        let up_b = self.new_block();
        let down_b = self.new_block();
        self.terminate(Terminator::If { cond: going_up, then: up_b, else_: down_b });
        self.cur = up_b;
        let le = self.into_temp(
            Rvalue::Binary { op: BinOp::Le, lhs: Operand::Read(Place::local(c)), rhs: Operand::Read(Place::local(stop_t)) },
            &Ty::Bool,
            span,
        );
        self.terminate(Terminator::If { cond: le, then: body_b, else_: exit });
        self.cur = down_b;
        let ge = self.into_temp(
            Rvalue::Binary { op: BinOp::Ge, lhs: Operand::Read(Place::local(c)), rhs: Operand::Read(Place::local(stop_t)) },
            &Ty::Bool,
            span,
        );
        self.terminate(Terminator::If { cond: ge, then: body_b, else_: exit });

        self.cur = body_b;
        self.loops.push(LoopTargets { brk: exit, cont: next, depth: self.scopes.len() });
        self.block(body);
        self.terminate(Terminator::Goto(next));
        self.loops.pop();

        // "The compiler compares before it steps" (owner, 2026-09-23): the
        // step is taken only when there is room for it — `stop - 'i'` is at
        // least the step going up, at most it going down. The difference is
        // taken in 128 bits, where two 64-bit values always fit, so the test
        // itself can never overflow whatever the counter's type; and the step
        // past the last value — `range[250, 255, 3]` would reach 256 — is
        // never computed, because the program never needs it.
        self.cur = next;
        let wide = Ty::int(true, 128);
        let rem = self.into_temp(
            Rvalue::Convert { to: wide.clone(), operand: Operand::Read(Place::local(stop_t)), useful: false, bits: false },
            &wide,
            span,
        );
        let at = self.into_temp(Rvalue::Convert { to: wide.clone(), operand: Operand::Read(Place::local(c)), useful: false, bits: false }, &wide, span);
        let rem = self.into_temp(Rvalue::Binary { op: BinOp::Sub, lhs: rem, rhs: at.clone() }, &wide, span);
        let by = self.into_temp(Rvalue::Convert { to: wide.clone(), operand: Operand::Read(Place::local(step_t)), useful: false, bits: false }, &wide, span);
        let up2 = self.into_temp(
            Rvalue::Binary { op: BinOp::Ge, lhs: Operand::Read(Place::local(step_t)), rhs: Operand::Const(Const::Int(0)) },
            &Ty::Bool,
            span,
        );
        let room_up = self.new_block();
        let room_down = self.new_block();
        let step_b = self.new_block();
        self.terminate(Terminator::If { cond: up2, then: room_up, else_: room_down });
        self.fors.push(ForLoop { pre, head, exit, counter: c, stop: stop_t, step: Some(step_t), start: Some(first_v), exits_from: vec![up_b, down_b, room_up, room_down], seq, para, perm, nocheat: std::mem::take(&mut self.nocheat_next) || self.nocheat_depth > 0, span });
        self.cur = room_up;
        let more_up = self.into_temp(Rvalue::Binary { op: BinOp::Ge, lhs: rem.clone(), rhs: by.clone() }, &Ty::Bool, span);
        self.terminate(Terminator::If { cond: more_up, then: step_b, else_: exit });
        self.cur = room_down;
        let more_down = self.into_temp(Rvalue::Binary { op: BinOp::Le, lhs: rem, rhs: by.clone() }, &Ty::Bool, span);
        self.terminate(Terminator::If { cond: more_down, then: step_b, else_: exit });
        self.cur = step_b;
        // The next value, in 128 bits, then into the counter — a narrowing
        // Precog proves from the test just made: there was room.
        let cty = self.locals[c.0 as usize].ty.clone();
        let next_v = self.into_temp(Rvalue::Binary { op: BinOp::Add, lhs: at, rhs: by }, &wide, span);
        self.site_of(SiteKind::Narrow { to: cty.clone() }, vec![next_v.clone()], Some(cty.clone()), span);
        self.assign(Place::local(c), Rvalue::Convert { to: cty, operand: next_v, useful: false, bits: false }, span);
        self.terminate(Terminator::Goto(head));

        self.cur = exit;
        if let Some(after) = after {
            self.terminate(Terminator::Goto(after));
            self.cur = after;
        }
        if perm {
            // The counter outlives the loop, holding what the loop last gave it.
            self.scopes.last_mut().unwrap().push(c);
        }
    }

    /// A `for` over a list, a text walk or a map walk: by position, so the
    /// element's own bounds check is the one the loop already proves.
    #[allow(clippy::too_many_arguments)]
    fn for_each(&mut self, elem: hir::LocalId, pos: Option<hir::LocalId>, iter: &hir::Expr, body: &hir::Block, perm: bool, seq_word: bool, para: bool, span: SpanId) {
        let e = LocalId(elem.0);
        let iter_op = self.eval(iter, span);
        let iter_ty = self.resolve_ty(&iter.ty);
        let seq = self.temp(&iter_ty, span);
        self.locals[seq.0 as usize].kind = match &iter_op {
            Operand::Move(_) => LocalKind::Owned,
            _ => LocalKind::Lent { mutable: false },
        };
        // A borrow is bound by a lend, or by another borrow passed along;
        // a plain read into a borrow writes *through* it (`codegen`: the
        // store into `for 'x' in lend.mut 'xs'`'s 'x'). A walk over a value
        // made for it (`str.each.byte[…]`) borrows that value, so it says so.
        let rv = match &iter_op {
            Operand::Read(p) if !matches!(self.locals[p.local.0 as usize].kind, LocalKind::Lent { .. }) => Rvalue::Lend { place: p.clone(), mutable: false },
            _ => Rvalue::Use(iter_op),
        };
        self.assign(Place::local(seq), rv, span);

        let u64_ty = Ty::u64();
        let len = self.temp(&u64_ty, span);
        let length_op = match iter_ty.base() {
            Ty::Seq { kind: crate::types::SeqKind::Array, .. } => Op::ArrayLength,
            _ => Op::ListLength,
        };
        self.push(StmtKind::Op { dest: Some(Place::local(len)), op: length_op, args: vec![Operand::Read(Place::local(seq))] }, span);

        let i = self.temp(&u64_ty, span);
        self.assign(Place::local(i), Rvalue::Use(Operand::Const(Const::Int(1))), span);

        let head = self.new_block();
        let body_b = self.new_block();
        let next = self.new_block();
        let exit = self.new_block();
        let pre = self.cur;
        self.terminate(Terminator::Goto(head));
        self.fors.push(ForLoop { pre, head, exit, counter: i, stop: len, step: None, start: None, exits_from: vec![head], seq: seq_word, para, perm, nocheat: std::mem::take(&mut self.nocheat_next) || self.nocheat_depth > 0, span });

        self.cur = head;
        let cmp = self.into_temp(
            Rvalue::Binary { op: BinOp::Le, lhs: Operand::Read(Place::local(i)), rhs: Operand::Read(Place::local(len)) },
            &Ty::Bool,
            span,
        );
        self.terminate(Terminator::If { cond: cmp, then: body_b, else_: exit });

        self.cur = body_b;
        // The index is the loop's own, between 1 and the length: the site is
        // here so Precog has something to prove rather than nothing to see.
        self.site(SiteKind::Bounds, vec![Operand::Read(Place::local(i)), Operand::Read(Place::local(len))], span);
        let mut src = Place::local(seq);
        src.proj.push(Proj::Index(Operand::Read(Place::local(i))));
        let rv = match self.locals[e.0 as usize].kind {
            LocalKind::Lent { mutable } => Rvalue::Lend { place: src, mutable },
            LocalKind::Owned => Rvalue::Use(Operand::Move(src)),
            _ => Rvalue::Use(Operand::Read(src)),
        };
        self.assign(Place::local(e), rv, span);
        if let Some(p) = pos {
            self.assign(Place::local(LocalId(p.0)), Rvalue::Use(Operand::Read(Place::local(i))), span);
        }
        self.loops.push(LoopTargets { brk: exit, cont: next, depth: self.scopes.len() });
        self.block(body);
        self.terminate(Terminator::Goto(next));
        self.loops.pop();

        self.cur = next;
        let ity = self.locals[i.0 as usize].ty.clone();
        self.site_of(SiteKind::Overflow(BinOp::Add), vec![Operand::Read(Place::local(i)), Operand::Const(Const::Int(1))], Some(ity), span);
        self.assign(
            Place::local(i),
            Rvalue::Binary { op: BinOp::Add, lhs: Operand::Read(Place::local(i)), rhs: Operand::Const(Const::Int(1)) },
            span,
        );
        self.terminate(Terminator::Goto(head));

        self.cur = exit;
        if perm {
            self.scopes.last_mut().unwrap().push(e);
        }
    }

    /// A `for` over an iterator: `next` at the head, its `none` the exit,
    /// its `value` the item.
    #[allow(clippy::too_many_arguments)]
    fn for_next(&mut self, elem: hir::LocalId, call: &hir::Expr, value_case: usize, none_case: usize, body: &hir::Block, perm: bool, span: SpanId) {
        let e = LocalId(elem.0);
        let head = self.new_block();
        let body_b = self.new_block();
        let exit = self.new_block();
        self.terminate(Terminator::Goto(head));
        self.cur = head;
        let v = self.eval(call, span);
        let v = match v.place() {
            Some(_) => v,
            None => {
                let ty = self.resolve_ty(&call.ty);
                let t = self.temp(&ty, span);
                self.assign(Place::local(t), Rvalue::Use(v), span);
                Operand::Move(Place::local(t))
            }
        };
        self.terminate(Terminator::Switch { value: v.clone(), cases: vec![(value_case, body_b), (none_case, exit)] });
        self.cur = body_b;
        self.push_scope();
        let mut src = v.place().unwrap().clone();
        src.proj.push(Proj::Field(value_case));
        self.assign(Place::local(e), Rvalue::Use(Operand::Read(src)), span);
        if !perm {
            self.scopes.last_mut().unwrap().push(e);
        }
        self.loops.push(LoopTargets { brk: exit, cont: head, depth: self.scopes.len() - 1 });
        self.block(body);
        self.loops.pop();
        self.pop_scope(span);
        self.terminate(Terminator::Goto(head));
        self.cur = exit;
        if perm {
            self.scopes.last_mut().unwrap().push(e);
        }
    }

    fn concur(&mut self, detached: bool, captures: &[hir::CaptureBinding], body: &hir::Block, otherwise: Option<&hir::Otherwise>, fails: &[(String, Option<Ty>)], span: SpanId) {
        // The task's body becomes its own MIR function; its captures are its
        // parameters, taken with the verbs the writer wrote. The checker put
        // the task's locals in the enclosing table, so the numbering is shared.
        let task_body = hir::Body {
            locals: self.hir_locals.clone(),
            params: captures.iter().map(|c| c.inner).collect(),
            ret: Ty::Nothing,
            fails: fails.to_vec(),
            block: body.clone(),
            span: self.spans[span.0 as usize],
            name: format!("a task in {}", self.fn_name()),
            is_main: false,
            is_test: false, ret_lent: None, };
        let idx = FnIdx(self.arena.functions.len() as u32);
        self.arena.functions.push(placeholder(&task_body));
        let mut fb = FnBuilder::new(self.arena, None, &task_body, true);
        fb.captures = captures.iter().map(|c| LocalId(c.inner.0)).collect();
        let handled_inside = detached && otherwise.is_some();
        if handled_inside {
            fb.catch_otherwise = otherwise.map(|o| (o.clone(), fails.to_vec()));
        }
        fb.lower_body(&task_body);
        let mut f = fb.finish(&task_body);
        if handled_inside {
            // Every failure lands in the task's own handler: nothing leaves.
            f.fails.clear();
        }
        self.arena.functions[idx.0 as usize] = f;

        let caps: Vec<(Verb, Place)> = captures.iter().map(|c| (c.verb, Place::local(LocalId(c.outer.0)))).collect();
        // A lend into a task must end before the lender's scope does: the
        // proof structured tasks make possible (`concurrency.md`).
        for (verb, place) in &caps {
            if matches!(verb, Verb::Lend | Verb::LendMut) {
                self.site(SiteKind::TaskLendOutlives, vec![Operand::Read(place.clone())], span);
            }
        }
        let id = self.next_task;
        self.next_task += 1;
        let group = if detached {
            u32::MAX
        } else {
            match *self.task_group.last().unwrap() {
                Some(g) => g,
                None => {
                    let g = self.next_group;
                    self.next_group += 1;
                    *self.task_group.last_mut().unwrap() = Some(g);
                    g
                }
            }
        };
        self.push(StmtKind::Task { detached, body: idx, captures: caps, group, id }, span);
        if !detached {
            *self.tasks_in_scope.last_mut().unwrap() += 1;
        }
        if let (false, Some(o)) = (detached, otherwise) {
            // The handler runs where the block waits, not here.
            let (arms, after) = self.fail_blocks(o, fails, None, span);
            self.task_fails.last_mut().unwrap().push((id, idx, arms, after));
        }
    }

    fn whichever(&mut self, arms: &[hir::WhicheverArm], otherwise: Option<&hir::Otherwise>, span: SpanId) {
        let join = self.new_block();
        let mut marms = Vec::new();
        for arm in arms {
            let ch = self.eval(&arm.channel, span);
            let b = self.new_block();
            marms.push(WhicheverArm { channel: ch, dest: Place::local(LocalId(arm.value.0)), block: b });
        }
        let closed_b = otherwise.map(|_| self.new_block());
        self.terminate(Terminator::Whichever { arms: marms.clone(), closed: closed_b });
        for (arm, marm) in arms.iter().zip(&marms) {
            self.cur = marm.block;
            self.push_scope();
            self.scopes.last_mut().unwrap().push(Place::local(LocalId(arm.value.0)).local);
            for st in &arm.body.stmts {
                if self.closed() {
                    break;
                }
                self.stmt(st);
            }
            self.pop_scope(span);
            self.terminate(Terminator::Goto(join));
        }
        if let (Some(b), Some(o)) = (closed_b, otherwise) {
            self.cur = b;
            match o {
                hir::Otherwise::Arms(arms) => {
                    for arm in arms {
                        self.push_scope();
                        for st in &arm.body.stmts {
                            if self.closed() {
                                break;
                            }
                            self.stmt(st);
                        }
                        self.pop_scope(span);
                    }
                    self.terminate(Terminator::Goto(join));
                }
                _ => self.otherwise(o, None, join, span),
            }
        }
        self.cur = join;
    }

    // ------------------------------------------------------------ stores

    /// Put a value into a place, with the check site a typed store needs.
    fn store(&mut self, place: Place, value: &hir::Expr, target: &Ty, typed: bool, span: SpanId) {
        // A call fills its destination directly, so a failing call has one
        // edge rather than a value to test afterwards.
        if let hir::ExprKind::Call { .. } | hir::ExprKind::Conv { .. } = &value.kind {
            // An answer narrower places cannot take as it is — an `i64`
            // into an `i32`, anything into a range type — lands in a
            // temporary first and is asked there, like any other value.
            // Filled straight into the place, it was never asked: `'c':
            // i32[0, 1] = twice[1]` held 2, and Precog believed the type.
            let from = self.resolve_ty(&value.ty);
            let to = self.resolve_ty(target);
            if typed && needs_narrow_check(&from, &to) && to.base().is_numeric() {
                let t = self.temp(&from, span);
                self.call_into(Some(Place::local(t)), value, span);
                self.site_of(SiteKind::Narrow { to: to.clone() }, vec![Operand::Read(Place::local(t))], Some(to), span);
                self.assign(place.clone(), Rvalue::Use(Operand::Read(Place::local(t))), span);
                self.store_site(&place, target, typed, span);
                return;
            }
            self.call_into(Some(place.clone()), value, span);
            self.store_site(&place, target, typed, span);
            return;
        }
        if let hir::ExprKind::Match { value: m, arms } = &value.kind {
            self.lower_match(m, arms, Some(place.clone()), span);
            self.store_site(&place, target, typed, span);
            return;
        }
        let rv = self.rvalue(value, span);
        let from = self.resolve_ty(&value.ty);
        let to = self.resolve_ty(target);
        // An operator whose answer has its own width — wrapping, and the
        // bitwise ones — keeps it until the answer exists: `'x' +~ 'x'` on
        // a `u32` wraps at 32 bits even stored into a `u64`
        // (`operators.md`). Assigned straight into the wider place, the
        // place's width was the only one left, and layer 1 folded
        // 8000000000 where the program computes 3705032704.
        if let Rvalue::Binary { op, .. } = &rv {
            let own_width = matches!(op, BinOp::WrapAdd | BinOp::WrapSub | BinOp::WrapMul | BinOp::WrapPow | BinOp::AndBit | BinOp::OrBit | BinOp::Xor | BinOp::Shl | BinOp::Shr);
            if own_width && from.base() != to.base() && from.is_int() && to.is_int() {
                let t = self.temp(&from, span);
                self.assign(Place::local(t), rv, span);
                let rv = Rvalue::Use(Operand::Read(Place::local(t)));
                if typed && needs_narrow_check(&from, &to) && to.base().is_numeric() {
                    self.assign(place.clone(), rv, span);
                    self.site_of(SiteKind::Narrow { to: to.clone() }, vec![Operand::Read(Place::local(t))], Some(to), span);
                } else {
                    self.assign(place.clone(), rv, span);
                }
                self.store_site(&place, target, typed, span);
                return;
            }
        }
        if typed && needs_narrow_check(&from, &to) && to.base().is_numeric() {
            // The value has to fit where it is going, and the site is about
            // the value, so it is asked before the store — of a temporary
            // when the value is not already one, because an operation's
            // result has nowhere else to be named.
            let op = match rvalue_operand(&rv) {
                Some(o) => {
                    self.assign(place.clone(), rv, span);
                    o
                }
                None => {
                    let t = self.temp(&from, span);
                    self.assign(Place::local(t), rv, span);
                    self.assign(place.clone(), Rvalue::Use(Operand::Read(Place::local(t))), span);
                    Operand::Read(Place::local(t))
                }
            };
            self.site_of(SiteKind::Narrow { to: to.clone() }, vec![op], Some(to), span);
            self.store_site(&place, target, typed, span);
            return;
        }
        self.assign(place.clone(), rv, span);
        self.store_site(&place, target, typed, span);
    }

    /// A value going into a slot of its own: a field, an element, an
    /// argument. The same question as a store, asked where the value is.
    fn narrow_arg(&mut self, op: &Operand, from: &Ty, to: &Ty, span: SpanId) {
        let from = self.resolve_ty(from);
        let to = self.resolve_ty(to);
        if needs_narrow_check(&from, &to) {
            // A length bound is asked of the length (cord claude: an empty
            // text into a `str.utf8[min 1 bytes]` was a narrowing nothing
            // checked).
            let kind = if matches!(to, Ty::Bounded { bound: Bound::Len { .. }, .. }) { SiteKind::LengthBound { to: to.clone() } } else { SiteKind::Narrow { to: to.clone() } };
            self.site_of(kind, vec![op.clone()], Some(to), span);
        }
    }

    /// After a store into a bounded collection, its bound is a check site.
    fn store_site(&mut self, place: &Place, target: &Ty, typed: bool, span: SpanId) {
        if !typed {
            return;
        }
        if let Ty::Bounded { bound: Bound::Len { .. }, .. } = self.resolve_ty(target) {
            self.site(SiteKind::LengthBound { to: target.clone() }, vec![Operand::Read(place.clone())], span);
        }
    }

    /// A value going into a narrower or bounded type.

    fn resolve_ty(&self, t: &Ty) -> Ty {
        resolved(t, &self.arena.subst)
    }

    /// An operand's type: a bare number has no width of its own yet.
    fn operand_ty(&self, o: &Operand) -> Ty {
        match o {
            Operand::Const(Const::Int(_)) => Ty::Int(IntW::Free(crate::types::InferId(0))),
            Operand::Const(Const::Bin(_)) => Ty::Bin(crate::types::BinW::Free(crate::types::InferId(0))),
            Operand::Const(Const::Bool(_)) => Ty::Bool,
            Operand::Const(Const::Str(_)) => Ty::Str(crate::types::Enc::Utf8),
            Operand::Const(Const::Deci { bits, .. }) => Ty::Deci(*bits),
            Operand::Const(_) => Ty::Nothing,
            Operand::Read(p) | Operand::Move(p) => {
                let base = self.locals[p.local.0 as usize].ty.clone();
                self.resolve_ty(&projected_ty(&base, &p.proj, &self.arena.items))
            }
        }
    }

    // ------------------------------------------------------------ expressions

    /// A value, as an operand. Calls split the block.
    fn eval(&mut self, e: &hir::Expr, span: SpanId) -> Operand {
        let span = self.span_id(e.span).max_of(span);
        match &e.kind {
            hir::ExprKind::Int(v) => Operand::Const(Const::Int(*v)),
            hir::ExprKind::BigInt(v) => Operand::Const(Const::Big(v.clone())),
            hir::ExprKind::Bin(t) => Operand::Const(Const::Bin(t.clone())),
            hir::ExprKind::Deci { bits, value } => Operand::Const(Const::Deci { bits: *bits, value: *value }),
            hir::ExprKind::Str(t) => Operand::Const(Const::Str(t.clone())),
            hir::ExprKind::Char(c) => Operand::Const(Const::Char(c.clone())),
            hir::ExprKind::Bool(b) => Operand::Const(Const::Bool(*b)),
            hir::ExprKind::Duration(n) => Operand::Const(Const::Duration(*n)),
            hir::ExprKind::Const(c) => match self.arena.consts.get(c.0 as usize).cloned().flatten() {
                Some(v) => Operand::Const(v),
                None => Operand::Const(Const::Const(*c)),
            },
            hir::ExprKind::SizeOf { align, ty } => Operand::Const(Const::SizeOf { align: *align, ty: ty.clone() }),
            // The same address in a temporary of the new pointer type, so
            // what reads through it reads a whole T.
            hir::ExprKind::PtrCast(v) => {
                let o = self.eval(v, span);
                let t = self.temp(&e.ty, span);
                self.assign(Place::local(t), Rvalue::Use(o), span);
                Operand::Read(Place::local(t))
            }
            hir::ExprKind::Nothing => Operand::Const(Const::Nothing),
            hir::ExprKind::Local(l) => Operand::Read(Place::local(LocalId(l.0))),
            hir::ExprKind::Read(p) => {
                let place = self.place(p, span);
                Operand::Read(place)
            }
            hir::ExprKind::Give(p) => {
                let place = self.place(p, span);
                Operand::Move(place)
            }
            hir::ExprKind::Match { value, arms } => {
                let ty = self.resolve_ty(&e.ty);
                let t = self.temp(&ty, span);
                self.lower_match(value, arms, Some(Place::local(t)), span);
                Operand::Read(Place::local(t))
            }
            hir::ExprKind::Call { .. } | hir::ExprKind::Conv { .. } => {
                let ty = self.resolve_ty(&e.ty);
                if ty == Ty::Nothing {
                    self.call_into(None, e, span);
                    Operand::Const(Const::Nothing)
                } else {
                    let t = self.temp(&ty, span);
                    // A borrow handed back and used where it stands (`total[first[&'s']]`):
                    // the temporary holds it as a named one would — the address, and
                    // what the call lent stays borrowed while it is in use.
                    if let hir::ExprKind::Call { callee: hir::Callee::Fn(inst), .. } = &e.kind {
                        let fid = self.arena.instances[inst.0 as usize].key.fn_id;
                        if let Some((mutable, _)) = &self.arena.items.fns[fid.0 as usize].ret_lent {
                            self.locals[t.0 as usize].kind = LocalKind::Lent { mutable: *mutable };
                        }
                    }
                    self.call_into(Some(Place::local(t)), e, span);
                    Operand::Read(Place::local(t))
                }
            }
            _ => {
                let rv = self.rvalue(e, span);
                let ty = self.resolve_ty(&e.ty);
                self.into_temp(rv, &ty, span)
            }
        }
    }

    fn rvalue(&mut self, e: &hir::Expr, span: SpanId) -> Rvalue {
        let span = self.span_id(e.span).max_of(span);
        match &e.kind {
            hir::ExprKind::Copy(p) => {
                let place = self.place(p, span);
                Rvalue::Duplicate(place)
            }
            hir::ExprKind::Lend { place, mutable } => {
                let p = self.place(place, span);
                Rvalue::Lend { place: p, mutable: *mutable }
            }
            hir::ExprKind::ExternRead(id) => Rvalue::ExternRead(*id),
            hir::ExprKind::Unary { op, expr } => {
                let operand = self.eval(expr, span);
                if *op == UnOp::Neg && self.resolve_ty(&expr.ty).is_int() {
                    let ty = self.resolve_ty(&e.ty);
                    self.site_of(SiteKind::Overflow(BinOp::Sub), vec![operand.clone()], Some(ty), span);
                }
                Rvalue::Unary { op: *op, operand }
            }
            // `and.then` / `or.else` (owner, 2026-09-25): the right side only
            // when the left has not decided — a branch, so Precog narrows the
            // right side by the left the way an `if` narrows its body
            // (`'i' </== list.length[…] and.then 'xs'['i']` is proven).
            hir::ExprKind::Binary { op: op @ (crate::ast::BinOp::AndThen | crate::ast::BinOp::OrElse), lhs, rhs, .. } => {
                let and = *op == crate::ast::BinOp::AndThen;
                let t = self.temp(&Ty::Bool, span);
                let l = self.eval(lhs, span);
                let right = self.new_block();
                let decided = self.new_block();
                let join = self.new_block();
                let (then_b, else_b) = if and { (right, decided) } else { (decided, right) };
                self.terminate(Terminator::If { cond: l, then: then_b, else_: else_b });
                self.cur = right;
                let r = self.eval(rhs, span);
                self.assign(Place::local(t), Rvalue::Use(r), span);
                self.terminate(Terminator::Goto(join));
                self.cur = decided;
                self.assign(Place::local(t), Rvalue::Use(Operand::Const(Const::Bool(!and))), span);
                self.terminate(Terminator::Goto(join));
                self.cur = join;
                Rvalue::Use(Operand::Read(Place::local(t)))
            }
            hir::ExprKind::Binary { op, lhs, rhs, .. } => {
                // "Bare numbers alone are worked out exactly at compile
                // time" (`operators.md`): no 64-bit window in between, only
                // the landing. `(2^64 - 1 + 1) mod 1` is 0 (check claude,
                // attack a); lowered as instructions, the 2^64 was a
                // "past 64 bits" stop.
                if self.resolve_ty(&e.ty).is_exact() {
                    if let Some(Const::Int(v)) = const_eval(e, &self.arena.consts, &self.arena.items) {
                        return Rvalue::Use(Operand::Const(Const::Int(v)));
                    }
                }
                let l = self.eval(lhs, span);
                let r = self.eval(rhs, span);
                let result = self.resolve_ty(&e.ty);
                self.binary_rvalue(op, l, r, lhs, rhs, result, span)
            }
            hir::ExprKind::UserOp { inst, lhs, rhs } => {
                // An operator is a call under another spelling.
                let args = vec![self.arg(lhs, span), self.arg(rhs, span)];
                let ty = self.resolve_ty(&e.ty);
                let dest = self.temp(&ty, span);
                let idx = self.fn_idx(*inst);
                let ok = self.new_block();
                self.terminate_at(Terminator::Call { callee: Callee::Fn(idx), args, dest: Some(Place::local(dest)), ok, fail: Vec::new() }, span);
                self.cur = ok;
                Rvalue::Use(Operand::Read(Place::local(dest)))
            }
            hir::ExprKind::List(items) => {
                let ops = items.iter().map(|a| self.arg_owned(a, span)).collect();
                Rvalue::MakeList(ops)
            }
            hir::ExprKind::MapLit(pairs) => {
                let ps = pairs.iter().map(|(k, v)| (self.arg_owned(k, span), self.arg_owned(v, span))).collect();
                Rvalue::MakeMap(ps)
            }
            hir::ExprKind::ListEmpty => Rvalue::MakeList(Vec::new()),
            hir::ExprKind::MapEmpty => Rvalue::MakeMap(Vec::new()),
            hir::ExprKind::Construct { id, fields } => {
                let info = self.arena.items.structs[id.0 as usize].clone();
                let args: Vec<Ty> = match self.resolve_ty(&e.ty) {
                    Ty::Struct { args, .. } => args,
                    _ => Vec::new(),
                };
                let mut ops = Vec::new();
                for (i, a) in fields.iter().enumerate() {
                    let op = self.arg_owned(a, span);
                    if let Some(fi) = info.fields.get(i) {
                        let want = fi.ty.subst(&args);
                        let have = self.arg_ty(a);
                        self.narrow_arg(&op, &have, &want, span);
                    }
                    ops.push(op);
                }
                Rvalue::MakeStruct { id: *id, fields: ops }
            }
            hir::ExprKind::EnumCase { id, case, payload } => {
                let p = payload.as_ref().map(|a| self.arg_owned(a, span));
                // A case's data lands where a field's does: checked there
                // (check claude, seed 15174: `E1.c4[0 minus 5]` with a
                // `u64` case wrapped to 2^64 - 5).
                if let (Some(op), Some(a)) = (&p, payload) {
                    let info = self.arena.items.enums[id.0 as usize].clone();
                    let args: Vec<Ty> = match self.resolve_ty(&e.ty) {
                        Ty::Enum { args, .. } => args,
                        _ => Vec::new(),
                    };
                    if let Some(want) = info.cases.get(*case).and_then(|c| c.payload.clone()) {
                        let want = want.subst(&args);
                        let have = self.arg_ty(a);
                        self.narrow_arg(op, &have, &want, span);
                    }
                }
                Rvalue::MakeEnum { id: *id, case: *case, payload: p }
            }
            hir::ExprKind::Channel { elem, capacity } => Rvalue::MakeChannel { elem: elem.clone(), capacity: *capacity },
            hir::ExprKind::FnRef(inst) => {
                let idx = self.fn_idx(*inst);
                Rvalue::FnRef(idx)
            }
            hir::ExprKind::FnLit { inst, captures } => {
                let idx = self.fn_idx(*inst);
                let caps = captures.iter().map(|c| (c.verb, Place::local(LocalId(c.outer.0)))).collect();
                Rvalue::Closure { body: idx, captures: caps }
            }
            hir::ExprKind::BundleElem { place, index } => {
                let mut p = self.place(place, span);
                p.proj.push(Proj::BundleElem(*index));
                Rvalue::Use(Operand::Read(p))
            }
            _ => Rvalue::Use(self.eval(e, span)),
        }
    }

    /// Two operands, worked out, under a binary operator: the sites it
    /// carries, and the instruction.
    #[allow(clippy::too_many_arguments)]
    fn binary_rvalue(&mut self, op: &BinOp, mut l: Operand, mut r: Operand, lhs: &hir::Expr, rhs: &hir::Expr, result: Ty, span: SpanId) -> Rvalue {
        // A bitwise operator works on one width, the answer's: an
        // exact operand has to fit it first, a site like any
        // narrowing (check claude: `'v' xor ('w' minus 13)` with
        // `u64`s took -8 as a bit pattern).
        // `shl.masked` / `shr.masked` (owner, 2026-10-07): the count
        // modulo the width, -1 being width - 1 as the chip masks it,
        // and the bits past the width cut off; no site. Worked as the
        // plain shift on the masked count, its answer read as bits.
        if let BinOp::ShlMasked | BinOp::ShrMasked = op {
            let plain = if *op == BinOp::ShlMasked { BinOp::Shl } else { BinOp::Shr };
            let width = match result.base() {
                Ty::Int(IntW::Fixed { bits, .. }) => *bits,
                _ => 64,
            };
            let whole = Ty::Int(IntW::Fixed { signed: true, bits: 129 });
            let count = self.into_temp(Rvalue::Convert { to: whole.clone(), operand: r, useful: false, bits: false }, &whole, span);
            let masked = self.into_temp(Rvalue::Binary { op: BinOp::Mod(crate::ast::Round::Floor), lhs: count, rhs: Operand::Const(Const::Int(width as i128)) }, &whole, span);
            let from = self.resolve_ty(&lhs.ty);
            let x = if from.base() != result.base() {
                self.into_temp(Rvalue::Convert { to: result.clone(), operand: l, useful: false, bits: true }, &result, span)
            } else {
                l
            };
            let shifted = self.into_temp(Rvalue::Binary { op: plain, lhs: x, rhs: masked }, &result, span);
            return Rvalue::Convert { to: result.clone(), operand: shifted, useful: false, bits: true };
        }
        // An exact operand wider than the limit in between (a group
        // of bare numbers worked at its own size beside a typed
        // one) is held to it where it is used (check claude, seed
        // 36150).
        if matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow | BinOp::Div(_) | BinOp::Mod(_)) {
            if let Some(limit) = result.exact_limit() {
                for (side, of) in [(&mut l, lhs), (&mut r, rhs)] {
                    let from = self.resolve_ty(&of.ty);
                    if from.exact_limit().is_some_and(|w| w > limit) {
                        self.site_of(SiteKind::Narrow { to: result.clone() }, vec![side.clone()], Some(result.clone()), span);
                        *side = self.into_temp(Rvalue::Convert { to: result.clone(), operand: side.clone(), useful: false, bits: false }, &result, span);
                    }
                }
            }
        }
        // `.wrap` takes the operands as their bits instead (owner,
        // 2026-10-07: "yea"): no site, the answer an N-bit pattern.
        let (op, wrap) = match op {
            BinOp::AndBitWrap => (&BinOp::AndBit, true),
            BinOp::OrBitWrap => (&BinOp::OrBit, true),
            BinOp::XorWrap => (&BinOp::Xor, true),
            other => (other, false),
        };
        // A shifted number fits its width first too (owner,
        // 2026-10-07: "a"): its own site, before the count's.
        if matches!(op, BinOp::Shl | BinOp::Shr) && matches!(result.base(), Ty::Int(IntW::Fixed { .. })) && !result.is_exact() {
            let from = self.resolve_ty(&lhs.ty);
            if from.is_exact() {
                self.narrow_arg(&l, &from, &result, span);
                l = self.into_temp(Rvalue::Convert { to: result.clone(), operand: l, useful: false, bits: false }, &result, span);
            }
        }
        if matches!(op, BinOp::AndBit | BinOp::OrBit | BinOp::Xor) && matches!(result.base(), Ty::Int(IntW::Fixed { .. })) && !result.is_exact() {
            for (side, of) in [(&mut l, lhs), (&mut r, rhs)] {
                let from = self.resolve_ty(&of.ty);
                if from.is_exact() || (wrap && from.base() != result.base()) {
                    if !wrap {
                        self.narrow_arg(side, &from, &result, span);
                    }
                    *side = self.into_temp(Rvalue::Convert { to: result.clone(), operand: side.clone(), useful: false, bits: wrap }, &result, span);
                }
            }
        }
        self.binary_sites(&op.plain(), &l, &r, &result, span);
        Rvalue::Binary { op: op.clone(), lhs: l, rhs: r }
    }

    /// The check sites a binary operation carries (`precog.md`).
    fn binary_sites(&mut self, op: &BinOp, l: &Operand, r: &Operand, result_ty: &Ty, span: SpanId) {
        let ty = self.resolve_ty(result_ty);
        // A vector's operators are the lane's, N times (`simd.md`): the same
        // sites, about every lane, at the lane's type. A comparison's
        // answer is a vector of `bool`s and has none.
        let ty = match ty.base() {
            Ty::Simd { elem, .. } if !op.is_comparison() => (**elem).clone(),
            _ => ty,
        };
        let int = ty.is_int();
        match op {
            // A bin never holds an infinity or a NaN (owner, 2026-09-29:
            // a): an answer that would be one is a check site, like integer
            // overflow — past the largest bin, a division by zero, a power
            // with no number (`-1.0 xx 0.5`).
            // A bin's `mod` by 0.0 has no number: the divisor is asked.
            BinOp::Mod(_) if matches!(ty.base(), Ty::Bin(_) | Ty::Brain) => {
                self.site(SiteKind::ModZero, vec![r.clone()], span);
            }
            // A whole quotient can be past the largest bin as well.
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div(_) | BinOp::DivIeee(_) | BinOp::Pow if matches!(ty.base(), Ty::Bin(_) | Ty::Brain) => {
                if op.is_div() {
                    self.site(SiteKind::DivZero, vec![r.clone()], span);
                }
                self.site_of(SiteKind::Overflow(op.clone()), vec![l.clone(), r.clone()], Some(ty.clone()), span);
            }
            // A negative exponent is a fraction, for the wrapping power too
            // (the oracle's reading; the design is silent — cca/README.md).
            BinOp::WrapPow if int => {
                self.site(SiteKind::NegativeExponent, vec![r.clone()], span);
            }
            // Wrapping arithmetic is always defined: never a site.
            BinOp::WrapAdd | BinOp::WrapSub | BinOp::WrapMul | BinOp::WrapPow => {}
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Pow if int => {
                if *op == BinOp::Pow {
                    self.site(SiteKind::NegativeExponent, vec![r.clone()], span);
                }
                self.site_of(SiteKind::Overflow(op.clone()), vec![l.clone(), r.clone()], Some(ty.clone()), span);
            }
            // Wrapped like the hardware (owner, 2026-10-03): only the
            // divisor is asked.
            BinOp::DivWrap(_) if int => {
                self.site(SiteKind::DivZero, vec![r.clone()], span);
            }
            BinOp::Div(_) | BinOp::DivIeee(_) if int => {
                self.site(SiteKind::DivZero, vec![r.clone()], span);
                // The one overflowing division: the lowest value by -1.
                self.site_of(SiteKind::Overflow(op.clone()), vec![l.clone(), r.clone()], Some(ty.clone()), span);
            }
            BinOp::Mod(rounding) if int => {
                self.site(SiteKind::ModZero, vec![r.clone()], span);
                // Rounded up, what is left over has the divisor's other
                // sign: below 0 for an unsigned type (`7 mod.ceil 2` is -1).
                if matches!(rounding, crate::ast::Round::Ceil | crate::ast::Round::Near) && !matches!(ty.base(), Ty::Int(crate::types::IntW::Fixed { signed: true, .. })) {
                    self.site_of(SiteKind::Overflow(op.clone()), vec![l.clone(), r.clone()], Some(ty.clone()), span);
                }
            }
            // A duration is a length of time and is never negative
            // (`concurrency.md`, owner 2026-09-22): taking one past zero is
            // a check site like any narrowing past a bound.
            BinOp::Sub if ty.base() == &Ty::Duration => {
                self.site_of(SiteKind::Narrow { to: Ty::Duration }, vec![l.clone(), r.clone()], Some(Ty::Duration), span);
            }
            BinOp::Shl => {
                self.site_of(SiteKind::ShiftWidth, vec![l.clone(), r.clone()], Some(ty.clone()), span);
            }
            // "The count must be 0 to width - 1, for `shl` and `shr` alike"
            // (owner, 2026-09-23).
            BinOp::Shr => {
                self.site_of(SiteKind::ShiftCount, vec![l.clone(), r.clone()], Some(ty.clone()), span);
            }
            _ => {}
        }
    }

    fn place(&mut self, p: &hir::Place, span: SpanId) -> Place {
        let mut out = Place::local(LocalId(p.base.0));
        for pr in &p.proj {
            match pr {
                hir::Proj::Field(i) => out.proj.push(Proj::Field(*i)),
                hir::Proj::Index(e) => {
                    let idx = self.eval(e, span);
                    // Every index is a bounds site: 1-based, within the length.
                    let base = Place { local: out.local, proj: out.proj.clone() };
                    let base_ty = self.locals[out.local.0 as usize].ty.clone();
                    let here = projected_ty(&base_ty, &out.proj, &self.arena.items);
                    let len = match here.base() {
                        // A vector's length is its shape: nothing to ask at run time.
                        Ty::Simd { lanes, .. } => Operand::Const(Const::Int(*lanes as i128)),
                        _ => {
                            let len_ty = Ty::u64();
                            let t = self.temp(&len_ty, span);
                            self.push(StmtKind::Op { dest: Some(Place::local(t)), op: Op::ListLength, args: vec![Operand::Read(base)] }, span);
                            Operand::Read(Place::local(t))
                        }
                    };
                    self.site(SiteKind::Bounds, vec![idx.clone(), len], span);
                    out.proj.push(Proj::Index(idx));
                }
            }
        }
        out
    }

    /// What each parameter of a callee holds, where that is known.
    fn param_tys(&self, callee: &hir::Callee) -> Vec<Ty> {
        match callee {
            // A variadic's last parameter is the bundle the call builds: each
            // value past the fixed ones goes into its own element type, in
            // order (the instance's key has them, unpacked lists included).
            hir::Callee::Fn(inst) => {
                let ins = &self.arena.instances[inst.0 as usize];
                let mut out: Vec<Ty> = match ins.body.as_ref() {
                    Some(b) => b.params.iter().map(|p| self.resolve_ty(&b.locals[p.0 as usize].ty)).collect(),
                    None => Vec::new(),
                };
                if let Some(bundle) = &ins.key.bundle {
                    out.pop();
                    out.extend(bundle.iter().map(|t| self.resolve_ty(t)));
                }
                out
            }
            hir::Callee::Extern(id) => self.arena.items.externs[id.0 as usize].params.iter().map(|(_, t)| self.resolve_ty(t)).collect(),
            _ => Vec::new(),
        }
    }

    fn fn_idx(&mut self, inst: FnInst) -> FnIdx {
        match self.arena.inst_idx.get(&inst) {
            Some(i) => *i,
            None => {
                // A function with no body (never happens for a call the
                // checker accepted); keep the graph honest rather than panic.
                FnIdx(u32::MAX)
            }
        }
    }

    fn arg(&mut self, a: &hir::Arg, span: SpanId) -> Arg {
        match a {
            hir::Arg::Value(e) => Arg::Value(self.eval(e, span)),
            hir::Arg::Lend { place, mutable } => {
                let p = self.place(place, span);
                Arg::Lend { place: p, mutable: *mutable }
            }
            hir::Arg::Give(place) => {
                let p = self.place(place, span);
                Arg::Give(p)
            }
            hir::Arg::Copy(place) => {
                let p = self.place(place, span);
                Arg::Copy(p)
            }
            hir::Arg::Spread { expr, count } => {
                let (place, verb) = match &**expr {
                    hir::Arg::Lend { place, .. } => (self.place(place, span), None),
                    hir::Arg::Give(place) => (self.place(place, span), Some(Verb::Give)),
                    hir::Arg::Copy(place) => (self.place(place, span), Some(Verb::Copy)),
                    other => {
                        let op = self.arg_operand(other, span);
                        match op.place() {
                            Some(p) => (p.clone(), None),
                            None => (Place::local(LocalId(0)), None),
                        }
                    }
                };
                Arg::Spread { place, count: *count, verb }
            }
        }
    }

    /// An argument that is being stored rather than passed: the operand it is.
    /// An item of a value being built — a list's, a map's, a struct's
    /// field, a case's data. A new value made for it (a nested list, a
    /// `copy`) is moved in, so the temporary that held it is not dropped
    /// too: the value being built owns it now.
    fn arg_owned(&mut self, a: &hir::Arg, span: SpanId) -> Operand {
        let op = self.arg_operand(a, span);
        match (a, op) {
            (hir::Arg::Value(_) | hir::Arg::Copy(_), Operand::Read(p)) if p.proj.is_empty() && self.locals[p.local.0 as usize].name.is_none() && self.locals[p.local.0 as usize].kind == LocalKind::Owned => Operand::Move(p),
            (_, op) => op,
        }
    }

    fn arg_operand(&mut self, a: &hir::Arg, span: SpanId) -> Operand {
        match a {
            hir::Arg::Value(e) => self.eval(e, span),
            hir::Arg::Give(place) => {
                let p = self.place(place, span);
                Operand::Move(p)
            }
            hir::Arg::Copy(place) => {
                let p = self.place(place, span);
                let ty = self.resolve_ty(&place.ty);
                self.into_temp(Rvalue::Duplicate(p), &ty, span)
            }
            hir::Arg::Lend { place, mutable } => {
                let p = self.place(place, span);
                let ty = self.resolve_ty(&place.ty);
                self.into_temp(Rvalue::Lend { place: p, mutable: *mutable }, &ty, span)
            }
            hir::Arg::Spread { expr, .. } => self.arg_operand(expr, span),
        }
    }

    /// A call or a conversion, with its failure edge.
    fn call_into(&mut self, dest: Option<Place>, e: &hir::Expr, span: SpanId) {
        let span = self.span_id(e.span).max_of(span);
        let (callee, args, otherwise, fails) = match &e.kind {
            hir::ExprKind::Call { callee, args, otherwise, fails } => {
                let c = match callee {
                    hir::Callee::Fn(inst) => {
                        let idx = self.fn_idx(*inst);
                        Callee::Fn(idx)
                    }
                    hir::Callee::Extern(id) => Callee::Extern(*id),
                    hir::Callee::Value(v) => {
                        let op = self.eval(v, span);
                        Callee::Value(op)
                    }
                    hir::Callee::Builtin(b) => Callee::Op(builtin_op(b)),
                };
                let mut margs: Vec<Arg> = args.iter().map(|a| self.arg(a, span)).collect();
                // A percent known now is printed as one (owner, 2026-10-17):
                // it goes in through a local of its type, which a constant
                // does not carry.
                if matches!(callee, hir::Callee::Builtin(hir::Builtin::Std(n)) if n.starts_with("print.")) {
                    for (m, a) in margs.iter_mut().zip(args) {
                        if let (Arg::Value(Operand::Const(c)), hir::Arg::Value(v)) = (&*m, a) {
                            let t = self.resolve_ty(&v.ty);
                            if crate::check::expr::percent_inner(&t).is_some() {
                                let o = self.into_temp(Rvalue::Use(Operand::Const(c.clone())), &t, span);
                                *m = Arg::Value(o);
                            }
                        }
                    }
                }
                // An owned value made only for this call — `show[to.stored.
                // str.utf8[5]]`, `show[make[]]` — is the function's from
                // then on: given, as `give` would, so it is not freed here
                // as well. It was, and the text was freed twice.
                // A parameter that only borrows lends it instead, and the
                // temporary is freed after the call (`count[[1, 2, 3]]` into
                // a `lent` list leaked the list).
                if matches!(callee, hir::Callee::Fn(_) | hir::Callee::Value(_)) {
                    let kinds: Vec<crate::types::ParamKind> = match callee {
                        hir::Callee::Fn(inst) => self.arena.instances[inst.0 as usize].sig.params.iter().map(|(k, _)| *k).collect(),
                        _ => Vec::new(),
                    };
                    for (k, m) in margs.iter_mut().enumerate() {
                        if let Arg::Value(Operand::Read(p)) = m {
                            let l = &self.locals[p.local.0 as usize];
                            if p.proj.is_empty() && l.name.is_none() && l.kind == LocalKind::Owned {
                                *m = match kinds.get(k) {
                                    Some(crate::types::ParamKind::Lent) => Arg::Lend { place: p.clone(), mutable: false },
                                    Some(crate::types::ParamKind::LentMut) => Arg::Lend { place: p.clone(), mutable: true },
                                    _ => Arg::Give(p.clone()),
                                };
                            }
                        }
                    }
                }
                // Into a parameter is a store like any other: a `u64`
                // handed to a `u8` parameter is a narrowing, and a
                // narrowing is a check site (`conversion.md`). It was not one.
                let wants = match (callee, self.resolve_ty(&e.ty).base()) {
                    // `simd.fill`'s value lands in every lane.
                    (hir::Callee::Builtin(hir::Builtin::SimdFill), Ty::Simd { elem, .. }) => vec![(**elem).clone()],
                    _ => self.param_tys(callee),
                };
                // std's own `str.at.X` / `str.slice.X`: each position after
                // the text is 1 or more (owner, 2026-10-17), asked before the
                // parameter's own bound so the message says so.
                let positions = match callee {
                    hir::Callee::Fn(inst) => {
                        let decl = &self.arena.items.fns[self.arena.instances[inst.0 as usize].key.fn_id.0 as usize];
                        self.arena.items.nerve_cord.is_some_and(|c| self.arena.items.kit_root(decl.module) == c) && (decl.path.starts_with("std::str.at.") || decl.path.starts_with("std::str.slice."))
                    }
                    _ => false,
                };
                let mut i = 0usize;
                for (a, m) in args.iter().zip(&margs) {
                    if positions && i >= 1 {
                        if let Arg::Value(op) = m {
                            let at = self.span_id(a.span());
                            self.site(SiteKind::Position, vec![op.clone()], at);
                        }
                    }
                    // An unpacked list stands for as many values as it holds.
                    if let Arg::Spread { count, .. } = m {
                        i += *count;
                        continue;
                    }
                    let Some(want) = wants.get(i) else { break };
                    i += 1;
                    // A lend into a bounded parameter (`lent 'part':
                    // str.utf8[min 1 bytes]`) has to fit the bound as a
                    // value would; any other lend is the value's own type.
                    if let Arg::Lend { place, .. } = m {
                        if matches!(self.resolve_ty(want), Ty::Bounded { .. }) {
                            let have = self.arg_ty(a);
                            let at = self.span_id(a.span());
                            self.narrow_arg(&Operand::Read(place.clone()), &have, want, at);
                        }
                        continue;
                    }
                    let have = self.arg_ty(a);
                    let op = arg_as_operand(m);
                    // At the argument's own place, not the call's: two
                    // arguments of one call are two sites, and a message
                    // (or check claude's matcher) has to tell them apart.
                    let at = self.span_id(a.span());
                    self.narrow_arg(&op, &have, want, at);
                }
                (c, margs, otherwise.as_deref(), fails.clone())
            }
            hir::ExprKind::Conv { target, expr, otherwise, fails, useful, bits, written } => {
                let mut op = self.eval(expr, span);
                let mut from_ty = self.resolve_ty(&expr.ty);
                // `to.i8.bits[…]`: at most the type's bits, -128 to 255
                // for an i8, then read as its (owner, 2026-10-06).
                if *bits {
                    let n = match self.resolve_ty(target).base() {
                        Ty::Int(IntW::Fixed { bits, .. }) => *bits,
                        _ => 64,
                    };
                    let room = Ty::Bounded { inner: Box::new(Ty::Int(IntW::Fixed { signed: true, bits: n + 1 })), bound: Bound::Int { lo: -(1i128 << (n - 1)), hi: (1i128 << n) - 1, said: Default::default() } };
                    self.narrow_arg(&op, &from_ty, &room, span);
                    let rv = Rvalue::Convert { to: target.clone(), operand: op, useful: false, bits: true };
                    match dest {
                        Some(p) => self.assign(p, rv, span),
                        None => {
                            let ty = target.clone();
                            let _ = self.into_temp(rv, &ty, span);
                        }
                    }
                    return;
                }
                // A C enum is its number, C's `int` (owner, 2026-09-28): that
                // first, and the rest is a conversion from an `i32`.
                if let Ty::Enum { id, .. } = from_ty.base() {
                    // As text it is its case's name (owner, 2026-09-30).
                    if self.arena.items.enums[id.0 as usize].numbers.is_some() && !matches!(self.resolve_ty(target).base(), Ty::Str(_)) {
                        let n = Ty::Int(IntW::Fixed { signed: true, bits: 32 });
                        op = self.into_temp(Rvalue::Convert { to: n.clone(), operand: op, useful: false, bits: false }, &n, span);
                        from_ty = n;
                    }
                }
                let f: Vec<(String, Option<Ty>)> = fails.iter().map(|n| (n.clone(), None)).collect();
                if f.is_empty() {
                    // A conversion that cannot fail is an ordinary
                    // instruction — but a *narrowing* one is still a check
                    // site (`conversion.md`): `to.i8['big']` asks whether
                    // it fits, and only `check` makes the compiler say so.
                    let from = from_ty.clone();
                    self.narrow_arg(&op, &from, &self.resolve_ty(target), span);
                    let rv = Rvalue::Convert { to: target.clone(), operand: op, useful: *useful, bits: false };
                    match dest {
                        Some(p) => self.assign(p, rv, span),
                        None => {
                            let ty = target.clone();
                            let _ = self.into_temp(rv, &ty, span);
                        }
                    }
                    return;
                }
                // A bin or deci into a whole-number type fails `not_whole`, or
                // `out_of_range` when it is too big (owner, 2026-09-30: a) —
                // one failing call, as text into a number is; no site.
                (Callee::Convert(target.clone(), *written), vec![Arg::Value(op)], otherwise.as_deref(), f)
            }
            _ => unreachable!("call_into on {:?}", e.kind),
        };
        // A language operation that cannot fail is a statement, not an edge.
        if fails.is_empty() {
            if let Callee::Op(op) = &callee {
                let op = op.clone();
                // What the operation keeps — the value sent, appended or
                // set — it takes: a fresh value's temporary is moved in, so
                // it is not given back a second time where it was made.
                let keeps = |i: usize| matches!((&op, i), (Op::ChannelSend, 1) | (Op::ListAppend, 1) | (Op::MapSet, 1) | (Op::MapSet, 2));
                let ops: Vec<Operand> = args.iter().enumerate().map(|(i, a)| {
                    let o = self.owned_operand(a, span);
                    match o {
                        Operand::Read(p) if keeps(i) && p.proj.is_empty() && self.locals[p.local.0 as usize].name.is_none() && self.locals[p.local.0 as usize].ty.is_owned() => Operand::Move(p),
                        o => o,
                    }
                }).collect();
                let result_ty = self.resolve_ty(&e.ty);
                // `simd.sum.lane` (owner, 2026-10-07): the whole sum, then
                // narrowed to the lane's type, checked as any landing is.
                if let (Op::SimdSum, Some(Ty::Simd { elem, lanes })) = (&op, args.first().map(|a| self.resolve_ty(&self.arg_mir_ty(a))).as_ref().map(|t| t.base().clone())) {
                    if let (Some(exact), false) = (Ty::simd_sum_exact(&elem, lanes), result_ty.is_exact()) {
                        let t = self.temp(&exact, span);
                        self.push(StmtKind::Op { dest: Some(Place::local(t)), op: Op::SimdSum, args: ops }, span);
                        let o = Operand::Read(Place::local(t));
                        self.narrow_arg(&o, &exact, &result_ty, span);
                        let rv = Rvalue::Convert { to: result_ty.clone(), operand: o, useful: false, bits: false };
                        match dest {
                            Some(p) => self.assign(p, rv, span),
                            None => {
                                let _ = self.into_temp(rv, &result_ty, span);
                            }
                        }
                        return;
                    }
                }
                self.before_op_sites(&op, &args, &result_ty, span);
                self.push(StmtKind::Op { dest: dest.clone(), op: op.clone(), args: ops }, span);
                self.after_op_sites(&op, &args, span);
                return;
            }
        }
        // A failing operation (`map.get`) carries its sites too.
        if let Callee::Op(op) = &callee {
            let op = op.clone();
            let result_ty = self.resolve_ty(&e.ty);
            self.before_op_sites(&op, &args, &result_ty, span);
        }
        let ok = self.new_block();
        let fail_arms = self.fail_blocks_for_call(otherwise, &fails, dest.clone(), ok, span);
        self.terminate_at(Terminator::Call { callee, args, dest, ok, fail: fail_arms }, span);
        self.cur = ok;
    }

    /// The sites an operation carries before it runs.
    fn before_op_sites(&mut self, op: &Op, args: &[Arg], ty: &Ty, span: SpanId) {
        // A text position is 1 or more (owner, 2026-10-17): one worked out
        // as 0 or below while running stops the program.
        if let Op::Std(n) = op {
            if n.starts_with("str.at.") || n.starts_with("str.slice.") {
                for a in args.iter().skip(1) {
                    self.site(SiteKind::Position, vec![arg_as_operand(a)], span);
                }
            }
        }
        // The position `list.remove` takes is within the list, as a read's
        // is (check claude, 2026-10-08: past the end it read freed memory).
        if *op == Op::ListRemove {
            if let (Some(Arg::Lend { place, .. }), Some(at)) = (args.first(), args.get(1)) {
                let len_ty = Ty::u64();
                let t = self.temp(&len_ty, span);
                self.push(StmtKind::Op { dest: Some(Place::local(t)), op: Op::ListLength, args: vec![Operand::Read(place.clone())] }, span);
                self.site(SiteKind::Bounds, vec![arg_as_operand(at), Operand::Read(Place::local(t))], span);
            }
        }
        // A key type with its own `hash` and `op ==`: "equal keys hash the
        // same" is a check site on every use of the map (`maps.md`).
        if matches!(op, Op::MapHas | Op::MapGet | Op::MapSet | Op::MapRemove) {
            if let Some(Arg::Lend { place, .. }) = args.first() {
                let base = self.locals[place.local.0 as usize].ty.clone();
                let mt = self.resolve_ty(&projected_ty(&base, &place.proj, &self.arena.items));
                if let Ty::Map { key, .. } = mt.base() {
                    if key_has_own_hash(&self.arena.items, key) {
                        self.site(SiteKind::KeyHashAgrees, vec![Operand::Read(place.clone())], span);
                    }
                }
            }
        }
        match op {
            // A load or a store touches N elements from the position: the
            // question is whether `i + N - 1` is still inside (`simd.md`).
            Op::SimdLoad | Op::SimdStore => {
                let lanes = match op {
                    Op::SimdLoad => ty.base().lane().and_then(|_| match ty.base() {
                        Ty::Simd { lanes, .. } => Some(*lanes),
                        _ => None,
                    }),
                    _ => args.get(2).map(|a| self.arg_ty_lanes(a)).unwrap_or(None),
                }
                .unwrap_or(0);
                let idx = args.get(1).map(arg_as_operand).unwrap_or(Operand::Const(Const::Int(1)));
                let list = args.first().map(arg_as_operand).unwrap_or(Operand::Const(Const::Nothing));
                self.site(SiteKind::SimdRange { lanes }, vec![idx, list], span);
            }
            // k items from the position, k at most N (owner, 2026-09-27: a).
            Op::SimdLoadSome | Op::SimdStoreSome => {
                let lanes = match op {
                    Op::SimdLoadSome => match ty.base() {
                        Ty::Simd { lanes, .. } => *lanes,
                        _ => 0,
                    },
                    _ => args.get(3).map(|a| self.arg_ty_lanes(a)).unwrap_or(None).unwrap_or(0),
                };
                let idx = args.get(1).map(arg_as_operand).unwrap_or(Operand::Const(Const::Int(1)));
                let list = args.first().map(arg_as_operand).unwrap_or(Operand::Const(Const::Nothing));
                let k = args.get(2).map(arg_as_operand).unwrap_or(Operand::Const(Const::Int(0)));
                self.site(SiteKind::SimdSome { lanes }, vec![idx, list, k], span);
            }
            // A count below 0 is a check site (owner, 2026-09-30).
            Op::ListFill => {
                let count = args.get(1).map(arg_as_operand).unwrap_or(Operand::Const(Const::Int(0)));
                self.site(SiteKind::FillCount, vec![count], span);
                // And the value fits what the list holds, as an append's
                // does: `list.fill[300, 2]` is no `list[i8]`.
                if let (Ty::Seq { elem, .. }, Some(a)) = (self.resolve_ty(ty).base().clone(), args.first()) {
                    let have = self.arg_mir_ty(a);
                    let o = arg_as_operand(a);
                    self.narrow_arg(&o, &have, &elem, span);
                }
            }
            // 1 <= first <= last + 1 <= length + 1 (`lists.md`).
            Op::ListSlice => {
                let Some(Arg::Lend { place, .. }) = args.first() else { return };
                let t = self.temp(&Ty::u64(), span);
                self.push(StmtKind::Op { dest: Some(Place::local(t)), op: Op::ListLength, args: vec![Operand::Read(place.clone())] }, span);
                let first = args.get(1).map(arg_as_operand).unwrap_or(Operand::Const(Const::Int(1)));
                let last = args.get(2).map(arg_as_operand).unwrap_or(Operand::Const(Const::Int(0)));
                self.site(SiteKind::SliceRange, vec![first, last, Operand::Read(Place::local(t))], span);
            }
            // A value going into a collection has to fit what the
            // collection holds, as a store into a variable does.
            Op::ListAppend | Op::MapSet => {
                let Some(coll) = args.first().and_then(|a| match a {
                    Arg::Lend { place, .. } => Some(place.clone()),
                    _ => None,
                }) else {
                    return;
                };
                let base = self.locals[coll.local.0 as usize].ty.clone();
                let coll_ty = projected_ty(&base, &coll.proj, &self.arena.items);
                let (key_ty, val_ty) = match coll_ty.base() {
                    Ty::Seq { elem, .. } => (None, Some((**elem).clone())),
                    Ty::Map { key, val, .. } => (Some((**key).clone()), Some((**val).clone())),
                    _ => (None, None),
                };
                if *op == Op::MapSet {
                    if let (Some(k), Some(a)) = (key_ty, args.get(1)) {
                        let have = self.arg_mir_ty(a);
                        let o = arg_as_operand(a);
                        self.narrow_arg(&o, &have, &k, span);
                    }
                    if let (Some(v), Some(a)) = (val_ty, args.get(2)) {
                        let have = self.arg_mir_ty(a);
                        let o = arg_as_operand(a);
                        self.narrow_arg(&o, &have, &v, span);
                    }
                } else if let (Some(v), Some(a)) = (val_ty, args.get(1)) {
                    let have = self.arg_mir_ty(a);
                    let o = arg_as_operand(a);
                    self.narrow_arg(&o, &have, &v, span);
                }
            }
            _ => {}
        }
    }

    /// The type a MIR argument already has.
    fn arg_mir_ty(&self, a: &Arg) -> Ty {
        let p = match a {
            Arg::Value(Operand::Const(Const::Int(_))) => return Ty::Int(crate::types::IntW::Free(crate::types::InferId(0))),
            Arg::Value(o) => match o.place() {
                Some(p) => p,
                None => return Ty::Nothing,
            },
            Arg::Lend { place, .. } | Arg::Give(place) | Arg::Copy(place) | Arg::Spread { place, .. } => place,
        };
        let base = self.locals[p.local.0 as usize].ty.clone();
        projected_ty(&base, &p.proj, &self.arena.items)
    }

    fn arg_ty_lanes(&self, a: &Arg) -> Option<u32> {
        let p = match a {
            Arg::Value(o) => o.place()?,
            Arg::Lend { place, .. } | Arg::Give(place) | Arg::Copy(place) => place,
            Arg::Spread { place, .. } => place,
        };
        let base = self.locals.get(p.local.0 as usize)?.ty.clone();
        match projected_ty(&base, &p.proj, &self.arena.items).base() {
            Ty::Simd { lanes, .. } => Some(*lanes),
            _ => None,
        }
    }

    /// A change to a bounded collection is checked after it happens.
    fn after_op_sites(&mut self, op: &Op, args: &[Arg], span: SpanId) {
        let changes = matches!(op, Op::ListAppend | Op::ListRemove | Op::MapSet | Op::MapRemove);
        if !changes {
            return;
        }
        if let Some(Arg::Lend { place, .. }) = args.first() {
            let place = place.clone();
            let base = self.locals[place.local.0 as usize].ty.clone();
            let ty = projected_ty(&base, &place.proj, &self.arena.items);
            if let Ty::Bounded { bound: Bound::Len { .. }, .. } = &ty {
                self.site(SiteKind::LengthBound { to: ty }, vec![Operand::Read(place)], span);
            }
        }
    }

    /// The blocks a failing call's failure lands in, one arm per reason.
    fn fail_blocks_for_call(&mut self, otherwise: Option<&hir::Otherwise>, fails: &[(String, Option<Ty>)], dest: Option<Place>, ok: BlockId, span: SpanId) -> Vec<FailArm> {
        let Some(o) = otherwise else { return Vec::new() };
        self.fail_blocks(o, fails, Some((dest, ok)), span).0
    }

    /// Lower an `otherwise` into blocks: one arm per reason the callee lists.
    /// `cont` is where the success path carries on, when there is one (a task's
    /// handler has none — the block it belongs to has already ended).
    fn fail_blocks(&mut self, o: &hir::Otherwise, fails: &[(String, Option<Ty>)], cont: Option<(Option<Place>, BlockId)>, span: SpanId) -> (Vec<FailArm>, BlockId) {
        let saved = self.cur;
        let after = match cont {
            Some((_, ok)) => ok,
            None => self.new_block(),
        };
        let dest = cont.and_then(|(d, _)| d);
        let out = match o {
            hir::Otherwise::Arms(arms) => {
                let mut out = Vec::new();
                for arm in arms {
                    let b = self.new_block();
                    let payload = arm.bind.map(|l| Place::local(LocalId(l.0)));
                    out.push((arm, FailArm { index: arm.index, payload, block: b }));
                }
                for (arm, m) in &out {
                    self.cur = m.block;
                    self.push_scope();
                    if let Some(l) = arm.bind {
                        self.scopes.last_mut().unwrap().push(LocalId(l.0));
                    }
                    for st in &arm.body.stmts {
                        if self.closed() {
                            break;
                        }
                        self.stmt(st);
                    }
                    if let (Some(g), Some(d), false) = (&arm.gives, &dest, self.closed()) {
                        let ty = self.resolve_ty(&g.ty);
                        self.store(d.clone(), g, &ty, false, span);
                    }
                    self.pop_scope(span);
                    self.terminate(Terminator::Goto(after));
                }
                out.into_iter().map(|(_, m)| m).collect()
            }
            hir::Otherwise::PassUp => {
                // The failure ckits unchanged, under *this* function's number
                // for the same reason — the caller's list has the name.
                let mine = self.my_fails();
                let mut out = Vec::new();
                for (i, (name, payload_ty)) in fails.iter().enumerate() {
                    let b = self.new_block();
                    self.cur = b;
                    let index = mine.iter().position(|(n, _)| n == name).unwrap_or(i);
                    let payload = payload_ty.as_ref().map(|t| {
                        let p = self.temp(t, span);
                        Place::local(p)
                    });
                    self.leave_failing(index, payload.clone().map(Operand::Move), span);
                    out.push(FailArm { index: i, payload, block: b });
                }
                out
            }
            _ => {
                // One block for every reason: a value, or a leaving statement.
                let b = self.new_block();
                self.cur = b;
                self.otherwise(o, dest, after, span);
                fails.iter().enumerate().map(|(i, _)| FailArm { index: i, payload: None, block: b }).collect()
            }
        };
        self.cur = saved;
        (out, after)
    }

    fn fn_name(&self) -> String {
        match self.inst {
            Some(i) => self.arena.instances[i.0 as usize].body.as_ref().map(|b| b.name.clone()).unwrap_or_else(|| "a function".into()),
            None => "a task".into(),
        }
    }

    /// The failures this function itself lists.
    fn my_fails(&self) -> Vec<(String, Option<Ty>)> {
        match self.inst {
            Some(i) => self.arena.instances[i.0 as usize].sig.fails.clone(),
            None => Vec::new(),
        }
    }

    /// What one non-arm `otherwise` does, lowered into the current block.
    /// A `match`: switch on the case, each arm in its own scope. Used as a
    /// value (owner, 2026-09-29), each arm that gives one stores it in
    /// `dest` before the join.
    fn lower_match(&mut self, value: &hir::Expr, arms: &[hir::MatchArm], dest: Option<Place>, span: SpanId) {
        let v = self.eval(value, span);
        let join = self.new_block();
        let mut cases = Vec::new();
        for arm in arms {
            let b = self.new_block();
            cases.push((arm.case, b));
        }
        self.terminate(Terminator::Switch { value: v.clone(), cases: cases.clone() });
        for (arm, (_, b)) in arms.iter().zip(&cases) {
            self.cur = *b;
            self.push_scope();
            if let (Some(bind), Some(place)) = (arm.bind, v.place()) {
                let mut src = place.clone();
                src.proj.push(Proj::Field(arm.case));
                let rv = match self.locals[bind.0 as usize].kind {
                    LocalKind::Lent { mutable } => Rvalue::Lend { place: src, mutable },
                    _ => Rvalue::Use(Operand::Read(src)),
                };
                self.assign(Place::local(LocalId(bind.0)), rv, span);
                self.scopes.last_mut().unwrap().push(LocalId(bind.0));
            }
            for st in &arm.body.stmts {
                if self.closed() {
                    break;
                }
                self.stmt(st);
            }
            // Into the destination's type, checked like any store: an arm's
            // exact answer lands there (check claude, seed 102: -1 went
            // into a `u32` through a `match` unchecked).
            if let (Some(g), Some(d), false) = (&arm.gives, &dest, self.closed()) {
                let base = self.locals[d.local.0 as usize].ty.clone();
                let to = self.resolve_ty(&projected_ty(&base, &d.proj, &self.arena.items));
                self.store(d.clone(), g, &to, true, span);
            }
            self.pop_scope(span);
            self.terminate(Terminator::Goto(join));
        }
        self.cur = join;
    }

    fn otherwise(&mut self, o: &hir::Otherwise, dest: Option<Place>, ok: BlockId, span: SpanId) {
        match o {
            hir::Otherwise::Value(e) => {
                let v = self.eval(e, span);
                if let Some(p) = dest {
                    self.assign(p, Rvalue::Use(v), span);
                }
                self.terminate(Terminator::Goto(ok));
            }
            hir::Otherwise::Leave(s) => {
                self.stmt(s);
                if !self.closed() {
                    self.terminate(Terminator::Goto(ok));
                }
            }
            hir::Otherwise::PassUp | hir::Otherwise::Arms(_) => unreachable!("handled in fail_blocks"),
        }
    }
}

fn placeholder(body: &hir::Body) -> Function {
    Function {
        dropped: false,
        nocheat: false,
        nocheat_of: None,
        nocheat_blocks: Vec::new(),
        nocheat_entries: Vec::new(),
        picked: Vec::new(),
        markers: Vec::new(),
        name: body.name.clone(),
        inst: None,
        params: Vec::new(),
        locals: Vec::new(),
        blocks: vec![BasicBlock { stmts: Vec::new(), term: Terminator::Unreachable, term_span: SpanId(0) }],
        ret: Ty::Nothing,
        ret_local: None,
        ret_lent: None,
        fails: Vec::new(),
        sites: Vec::new(),
        spans: Vec::new(),
        is_main: body.is_main,
        is_test: body.is_test,
        is_task: false,
        variadic: false,
        captures: Vec::new(),
        ranges: Default::default(),
        operand_ranges: Default::default(),
        fors: Vec::new(),
    }
}

fn arg_as_operand(a: &Arg) -> Operand {
    match a {
        Arg::Value(o) => o.clone(),
        Arg::Lend { place, .. } | Arg::Copy(place) => Operand::Read(place.clone()),
        Arg::Give(place) => Operand::Move(place.clone()),
        Arg::Spread { place, .. } => Operand::Read(place.clone()),
    }
}

impl<'a> FnBuilder<'a> {
    /// An argument to a language operation, with a `copy` made real.
    ///
    /// A collection owns what is put in it (`lists.md`: "a list owns its
    /// elements: `append` takes a fresh value, a `copy`, or a `give` —
    /// never a lend"), and `copy` is "a second, independent value; an
    /// allocation" (`ownership-syntax.md`). Read as a plain operand, a
    /// `copy` of a list or a text would put the *same* buffer in the
    /// collection and leave the original owning it too — and both would
    /// give it back. The duplicate is made here, where the IR can say so.
    fn owned_operand(&mut self, a: &Arg, span: SpanId) -> Operand {
        if let Arg::Copy(place) = a {
            let base = self.locals[place.local.0 as usize].ty.clone();
            let ty = projected_ty(&base, &place.proj, &self.arena.items);
            if ty.is_owned() {
                let t = self.into_temp(Rvalue::Duplicate(place.clone()), &ty, span);
                return match t {
                    Operand::Read(p) => Operand::Move(p),
                    other => other,
                };
            }
        }
        arg_as_operand(a)
    }

    /// The type a HIR argument has before it goes anywhere.
    fn arg_ty(&self, a: &hir::Arg) -> Ty {
        match a {
            hir::Arg::Value(e) => self.resolve_ty(&e.ty),
            hir::Arg::Lend { place, .. } | hir::Arg::Give(place) | hir::Arg::Copy(place) => self.resolve_ty(&place.ty),
            hir::Arg::Spread { expr, .. } => self.arg_ty(expr),
        }
    }
}

fn rvalue_operand(rv: &Rvalue) -> Option<Operand> {
    match rv {
        Rvalue::Use(o) => Some(o.clone()),
        Rvalue::Binary { .. } | Rvalue::Unary { .. } | Rvalue::Convert { .. } => None,
        _ => None,
    }
}

/// Does storing a `from` into a `to` need a check?
pub fn needs_narrow_check(from: &Ty, to: &Ty) -> bool {
    // Into a bound: always, unless the source's bound is inside it.
    if let Ty::Bounded { bound: to_b, inner } = to {
        if let Bound::Int { lo, hi, .. } = to_b {
            if let Ty::Bounded { bound: Bound::Int { lo: flo, hi: fhi, .. }, .. } = from {
                return !(flo >= lo && fhi <= hi);
            }
            let _ = inner;
            return true;
        }
        if let Bound::Len { lo, hi, unit } = to_b {
            if let Ty::Bounded { bound: Bound::Len { lo: flo, hi: fhi, unit: funit }, .. } = from {
                return !(funit == unit && flo >= lo && fhi <= hi);
            }
            return true;
        }
        return true;
    }
    match (from.base(), to.base()) {
        (Ty::Int(a), Ty::Int(b)) => match (a, b) {
            (IntW::Fixed { signed: s1, bits: b1 }, IntW::Fixed { signed: s2, bits: b2 }) => !crate::resolve::int_fits(*s1, *b1, *s2, *b2),
            // A width Precog has not chosen: the value decides, and the store
            // is where it has to fit.
            _ => true,
        },
        (Ty::Bin(_), Ty::Bin(_)) => true,
        // A whole number that can round past the largest bin or brain16:
        // a wide one (owner, 2026-10-03), or an `i32` into an f16.
        (Ty::Int(IntW::Fixed { signed, bits }), Ty::Bin(crate::types::BinW::Fixed(b))) => {
            let (lo, hi) = crate::bigint::type_range(*signed, *bits);
            let limit = crate::bigint::bin_round_limit(*b);
            hi >= limit || lo.neg() >= limit
        }
        (Ty::Int(IntW::Fixed { signed, bits }), Ty::Brain) => {
            let (lo, hi) = crate::bigint::type_range(*signed, *bits);
            let limit = crate::bigint::bin_round_limit(crate::types::BRAIN_BITS);
            hi >= limit || lo.neg() >= limit
        }
        // Past the largest brain16 (owner, 2026-10-02), or a brain16 past
        // the largest f16.
        (Ty::Bin(_) | Ty::Deci(_), Ty::Brain) => true,
        (Ty::Brain, Ty::Bin(crate::types::BinW::Fixed(16))) => true,
        _ => false,
    }
}

fn builtin_op(b: &Builtin) -> Op {
    match b {
        Builtin::MustReport => Op::MustReport,
        Builtin::MustStopFork => Op::MustStopFork,
        Builtin::MustStopDone => Op::MustStopDone,
        Builtin::ListLength => Op::ListLength,
        Builtin::ListAppend => Op::ListAppend,
        Builtin::ListFill => Op::ListFill,
        Builtin::ListSlice | Builtin::ArraySlice => Op::ListSlice,
        Builtin::ToList => Op::ToList,
        Builtin::ToArray => Op::ToArray,
        Builtin::ListRemove => Op::ListRemove,
        Builtin::ArrayLength => Op::ArrayLength,
        Builtin::InclBytes => Op::InclBytes,
        Builtin::ConstView => Op::ConstView,
        Builtin::MapLength => Op::MapLength,
        Builtin::MapHas => Op::MapHas,
        Builtin::MapGet => Op::MapGet,
        Builtin::MapSet => Op::MapSet,
        Builtin::MapRemove => Op::MapRemove,
        Builtin::MapKeys => Op::MapKeys,
        Builtin::MapValues => Op::MapValues,
        Builtin::MapPairs => Op::MapPairs,
        Builtin::StrBytes => Op::StrBytes,
        Builtin::StrPoints => Op::StrPoints,
        Builtin::StrGraphemes => Op::StrGraphemes,
        Builtin::StrEach(u) => Op::StrEach(*u),
        Builtin::CharCode => Op::CharCode,
        Builtin::MachineHas(f) => Op::MachineHas(*f),
        Builtin::CharFromCode(u) => Op::CharFromCode(*u),
        Builtin::PtrRead => Op::PtrRead,
        Builtin::CStrRead => Op::CStrRead,
        Builtin::CStrMake => Op::CStrMake,
        Builtin::PtrWrite => Op::PtrWrite,
        Builtin::PtrAt => Op::PtrAt,
        Builtin::PtrIsNull => Op::PtrIsNull,
        Builtin::PtrNull => Op::PtrNull,
        Builtin::ChannelSend => Op::ChannelSend,
        Builtin::ChannelTake => Op::ChannelTake,
        Builtin::SimdFill => Op::SimdFill,
        Builtin::SimdLoad => Op::SimdLoad,
        Builtin::SimdStore => Op::SimdStore,
        Builtin::SimdAll => Op::SimdAll,
        Builtin::SimdAny => Op::SimdAny,
        Builtin::SimdMax => Op::SimdMax,
        Builtin::SimdMin => Op::SimdMin,
        Builtin::SimdSum | Builtin::SimdSumLane => Op::SimdSum,
        Builtin::SimdSumWrap => Op::SimdSumWrap,
        Builtin::SimdShuffle(order) => Op::SimdShuffle(order.clone()),
        Builtin::SimdLoadSome => Op::SimdLoadSome,
        Builtin::SimdStoreSome => Op::SimdStoreSome,
        Builtin::Std(n) => Op::Std(n.clone()),
    }
}

/// The type a projection reaches.
pub fn projected_ty(base: &Ty, proj: &[Proj], items: &crate::resolve::Items) -> Ty {
    let mut t = base.clone();
    for p in proj {
        t = match (p, t.base().clone()) {
            (Proj::Field(i), Ty::Struct { id, args }) => items.structs[id.0 as usize].fields[*i].ty.subst(&args),
            // On an enum, a field *is* a case: `match` binds the payload
            // of the arm it took (`enums.md`).
            (Proj::Field(i), Ty::Enum { id, args }) => items.enums[id.0 as usize].cases[*i].payload.clone().map(|t| t.subst(&args)).unwrap_or(Ty::Nothing),
            (Proj::Index(_), Ty::Seq { elem, .. }) => (*elem).clone(),
            // One lane of a vector is its element type: without this a lane
            // was `nothing`, and a `u8` lane of 200 was widened by sign
            // to 2^64 - 56 (check claude).
            (Proj::Index(_), Ty::Simd { elem, .. }) => (*elem).clone(),
            (Proj::BundleElem(i), Ty::Bundle(ts)) => ts.get(*i).cloned().unwrap_or(Ty::Nothing),
            (Proj::Deref, Ty::Ptr { elem, .. }) => (*elem).clone(),
            _ => Ty::Nothing,
        };
    }
    t
}

impl SpanId {
    /// The more specific of two spans: an expression's own, when it has one.
    fn max_of(self, _other: SpanId) -> SpanId {
        self
    }
}

impl ParamKind {
    pub fn is_lent(self) -> bool {
        matches!(self, ParamKind::Lent | ParamKind::LentMut)
    }
}

/// Does this type bring its own `hash`? The interface is
/// `fn hash [lent 'v': <T>] -> u64` (`maps.md`).
pub fn key_has_own_hash(items: &crate::resolve::Items, t: &Ty) -> bool {
    let name = crate::types::TySpelling { ty: t, structs: &items.structs, enums: &items.enums }.to_string();
    items.fns.iter().any(|f| {
        f.name == "hash"
            && f.params.len() == 1
            && f.params[0].ty.as_ref().map(|t| crate::types::TySpelling { ty: t, structs: &items.structs, enums: &items.enums }.to_string() == name).unwrap_or(false)
    })
}
