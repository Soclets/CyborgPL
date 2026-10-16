//! Parallel `for` (`parallelism.md`; owner, 2026-09-30: "do para for").
//!
//! Which lowered `for` loops may run in slices on the cores, and what each
//! slice keeps to itself. The proof is ownership's: every iteration writes
//! only what is its own — locals it makes, the element at its own position,
//! a fold into one variable — and reads nothing another iteration writes.
//! A loop that prints, reads input or calls C stays in order by itself
//! unless it says `para`; a conflict Precog can see stays an error even
//! then. Whether a loop is *worth* its threads is decided when it runs,
//! from its trips and the work of each (`WORTH`), never from the machine.

use std::collections::{HashMap, HashSet};

use crate::ast::BinOp;
use crate::mir::*;
use crate::types::{IntW, Ty};

/// Trips times the statements of one trip at which slicing pays for the
/// threads it wakes (provisional: a measurement in `tune.cplr` will win
/// where there is one, `parallelism.md`).
pub const WORTH: u64 = 64_000;
/// At most this many slices, so at most this many partials per fold.
pub const MAX_SLICES: u64 = 64;

/// A loop that runs in slices.
#[derive(Debug, Clone)]
pub struct ParLoop {
    pub fl: ForLoop,
    /// Its blocks: the head and everything a trip goes through.
    pub blocks: Vec<usize>,
    /// Locals each slice has its own of; every other local it touches is
    /// the parent's, read in place.
    pub private: HashSet<u32>,
    pub folds: Vec<Fold>,
    /// Statements in one trip, the estimate `WORTH` is measured against.
    pub work: u64,
}

/// `change 'acc' = 'acc' + x;` (or `x`) in every trip: each slice folds
/// its own trips from nothing — 0 for a sum, 1 for a product — and the
/// parent folds the partials into what 'acc' held (owner, 2026-09-30:
/// reductions, for every type). Integers fold in 128 bits and are checked
/// once at the end; a bin or deci folds in its own type, and its total may
/// round differently from run to run (the owner's ruling).
#[derive(Debug, Clone)]
pub struct Fold {
    pub acc: LocalId,
    pub op: BinOp,
    pub kind: FoldKind,
    /// Where `t = acc + x` is, (block, statement), and the `x`.
    pub adds: Vec<(usize, usize, Operand)>,
    /// What a slice leaves out: the store back into 'acc', and the sites
    /// on the add and the store.
    pub skip: HashSet<(usize, usize)>,
    /// Every term is 0 or more, or every term 0 or less (Precog's ranges):
    /// the running total then only moves one way, and the loop stops at a
    /// store that does not fit exactly when the total does not.
    pub one_signed: bool,
    /// Every site on an integer fold is proven, so the total fits: the
    /// slices can add (or multiply) at the variable's own width, wrapping,
    /// since arithmetic modulo 2^width lands on the true total when it fits.
    pub wrap: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FoldKind {
    /// A whole number, folded in 128 bits.
    Int,
    /// A bin, folded in its own type.
    Bin,
    /// A deci of these bits, folded by the runtime's arithmetic.
    Deci(u16),
}

pub struct Plan {
    /// Keyed by (function, the block that jumps into the loop's head).
    pub loops: HashMap<(usize, usize), ParLoop>,
    /// A `for para` whose iterations conflict: (function, where, why).
    pub errors: Vec<(usize, SpanId, String)>,
    /// Every other loop left in order, and why (`KAYO_PAR=1` prints them).
    pub declined: Vec<(usize, SpanId, String)>,
}

#[derive(Clone, Copy, Default, PartialEq, Debug)]
struct Effects {
    /// Prints, reads input, or reaches C: in order unless `para`.
    outside: bool,
    /// Tasks, channels, `must`: never in slices.
    never: bool,
}

impl Effects {
    fn join(&mut self, o: Effects) -> bool {
        let before = *self;
        self.outside |= o.outside;
        self.never |= o.never;
        *self != before
    }
}

fn op_effects(op: &Op) -> Effects {
    match op {
        Op::Std(_) | Op::PtrRead | Op::PtrWrite => Effects { outside: true, never: false },
        Op::ChannelSend | Op::ChannelTake | Op::ChannelRelease | Op::MustReport | Op::MustStopFork | Op::MustStopDone => Effects { outside: false, never: true },
        _ => Effects::default(),
    }
}

/// What each function does beyond its own memory, through everything it
/// calls.
fn effects(p: &Program) -> Vec<Effects> {
    let mut own = vec![Effects::default(); p.functions.len()];
    let mut calls: Vec<Vec<usize>> = vec![Vec::new(); p.functions.len()];
    for (i, f) in p.functions.iter().enumerate() {
        for b in &f.blocks {
            for s in &b.stmts {
                match &s.kind {
                    StmtKind::Task { .. } => own[i].never = true,
                    StmtKind::Asm { .. } => own[i].outside = true,
                    StmtKind::Op { op, .. } => {
                        own[i].join(op_effects(op));
                    }
                    StmtKind::Assign { rvalue: Rvalue::ExternRead(_), .. } => own[i].outside = true,
                    _ => {}
                }
            }
            match &b.term {
                Terminator::Call { callee, .. } => match callee {
                    Callee::Extern(_) => own[i].outside = true,
                    Callee::Value(_) => own[i].never = true,
                    Callee::Op(op) => {
                        own[i].join(op_effects(op));
                    }
                    Callee::Fn(idx) => calls[i].push(idx.0 as usize),
                    Callee::Convert(..) => {}
                },
                Terminator::Whichever { .. } | Terminator::Join { .. } | Terminator::TaskOutcome { .. } => own[i].never = true,
                _ => {}
            }
        }
    }
    let mut changed = true;
    while changed {
        changed = false;
        for i in 0..own.len() {
            for &c in &calls[i].clone() {
                let e = own[c];
                if own[i].join(e) {
                    changed = true;
                }
            }
        }
    }
    own
}

// ------------------------------------------------------------ reads and writes

fn operand_uses(o: &Operand, out: &mut Vec<u32>) {
    if let Some(p) = o.place() {
        place_uses(p, out);
    }
}

fn place_uses(p: &Place, out: &mut Vec<u32>) {
    out.push(p.local.0);
    for pr in &p.proj {
        if let Proj::Index(o) = pr {
            operand_uses(o, out);
        }
    }
}

fn rvalue_uses(rv: &Rvalue, out: &mut Vec<u32>) {
    match rv {
        Rvalue::Use(o) | Rvalue::Unary { operand: o, .. } | Rvalue::Convert { operand: o, .. } => operand_uses(o, out),
        Rvalue::Duplicate(p) | Rvalue::Lend { place: p, .. } => place_uses(p, out),
        Rvalue::Binary { lhs, rhs, .. } => {
            operand_uses(lhs, out);
            operand_uses(rhs, out);
        }
        Rvalue::MakeList(os) => os.iter().for_each(|o| operand_uses(o, out)),
        Rvalue::MakeMap(kv) => kv.iter().for_each(|(k, v)| {
            operand_uses(k, out);
            operand_uses(v, out);
        }),
        Rvalue::MakeStruct { fields, .. } => fields.iter().for_each(|o| operand_uses(o, out)),
        Rvalue::MakeEnum { payload, .. } => {
            if let Some(o) = payload {
                operand_uses(o, out);
            }
        }
        Rvalue::Closure { captures, .. } => captures.iter().for_each(|(_, p)| place_uses(p, out)),
        Rvalue::MakeChannel { .. } | Rvalue::FnRef(_) | Rvalue::ExternRead(_) => {}
    }
}

fn arg_place(a: &Arg) -> Option<&Place> {
    match a {
        Arg::Value(o) => o.place(),
        Arg::Lend { place, .. } | Arg::Give(place) | Arg::Copy(place) | Arg::Spread { place, .. } => Some(place),
    }
}

/// The locals a statement reads, and the one it gives a whole new value.
fn stmt_uses_defs(s: &Statement) -> (Vec<u32>, Option<u32>) {
    let mut u = Vec::new();
    let mut d = None;
    match &s.kind {
        StmtKind::Assign { place, rvalue } => {
            rvalue_uses(rvalue, &mut u);
            if place.proj.is_empty() {
                d = Some(place.local.0);
            } else {
                place_uses(place, &mut u);
            }
        }
        StmtKind::Op { dest, args, .. } => {
            args.iter().for_each(|o| operand_uses(o, &mut u));
            if let Some(p) = dest {
                if p.proj.is_empty() {
                    d = Some(p.local.0);
                } else {
                    place_uses(p, &mut u);
                }
            }
        }
        StmtKind::Drop(p) | StmtKind::EndLend(p) => place_uses(p, &mut u),
        StmtKind::Task { captures, .. } => captures.iter().for_each(|(_, p)| place_uses(p, &mut u)),
        StmtKind::Asm { ins, outs, lends, .. } => {
            ins.iter().for_each(|(p, _)| place_uses(p, &mut u));
            outs.iter().for_each(|(p, _)| place_uses(p, &mut u));
            lends.iter().for_each(|(p, _, _)| place_uses(p, &mut u));
        }
        StmtKind::Site(_) | StmtKind::Nop => {}
    }
    (u, d)
}

fn term_uses_defs(t: &Terminator) -> (Vec<u32>, Option<u32>) {
    let mut u = Vec::new();
    let mut d = None;
    match t {
        Terminator::If { cond, .. } => operand_uses(cond, &mut u),
        Terminator::Switch { value, .. } => operand_uses(value, &mut u),
        Terminator::Call { callee, args, dest, .. } => {
            if let Callee::Value(o) = callee {
                operand_uses(o, &mut u);
            }
            for a in args {
                if let Some(p) = arg_place(a) {
                    place_uses(p, &mut u);
                }
            }
            if let Some(p) = dest {
                if p.proj.is_empty() {
                    d = Some(p.local.0);
                } else {
                    place_uses(p, &mut u);
                }
            }
        }
        Terminator::Fail { payload: Some(o), .. } => operand_uses(o, &mut u),
        Terminator::Whichever { arms, .. } => arms.iter().for_each(|a| operand_uses(&a.channel, &mut u)),
        _ => {}
    }
    (u, d)
}

fn successors(t: &Terminator) -> Vec<usize> {
    crate::mir::verify::successors(t).into_iter().map(|b| b.0 as usize).collect()
}

/// Which locals are read before being written, from each block on.
fn live_in(f: &Function, only: Option<&HashSet<usize>>) -> Vec<HashSet<u32>> {
    let n = f.blocks.len();
    let mut gen: Vec<HashSet<u32>> = vec![HashSet::new(); n];
    let mut kill: Vec<HashSet<u32>> = vec![HashSet::new(); n];
    for (b, blk) in f.blocks.iter().enumerate() {
        let mut steps: Vec<(Vec<u32>, Option<u32>)> = blk.stmts.iter().map(stmt_uses_defs).collect();
        steps.push(term_uses_defs(&blk.term));
        for (uses, def) in steps {
            for u in uses {
                if !kill[b].contains(&u) {
                    gen[b].insert(u);
                }
            }
            if let Some(d) = def {
                kill[b].insert(d);
            }
        }
    }
    let mut live: Vec<HashSet<u32>> = gen.clone();
    let mut changed = true;
    while changed {
        changed = false;
        for b in (0..n).rev() {
            if only.is_some_and(|o| !o.contains(&b)) {
                continue;
            }
            let mut out: HashSet<u32> = HashSet::new();
            for s in successors(&f.blocks[b].term) {
                if only.is_some_and(|o| !o.contains(&s)) {
                    continue;
                }
                out.extend(live[s].iter().copied());
            }
            let mut inn = gen[b].clone();
            inn.extend(out.difference(&kill[b]).copied());
            if inn.len() != live[b].len() {
                live[b] = inn;
                changed = true;
            }
        }
    }
    live
}

// ------------------------------------------------------------ the plan

enum Why {
    /// Prints, input, C: in order by itself, `para` overrides.
    Order,
    /// Two iterations would touch the same thing, or a trip can leave the
    /// loop: never in slices, and an error under `para`.
    Conflict(String),
}

pub fn plan(p: &Program) -> Plan {
    let eff = effects(p);
    let mut out = Plan { loops: HashMap::new(), errors: Vec::new(), declined: Vec::new() };
    for (fi, f) in p.functions.iter().enumerate() {
        if f.fors.is_empty() {
            continue;
        }
        let live_fn = live_in(f, None);
        for fl in &f.fors {
            if fl.seq {
                continue;
            }
            match analyse(p, f, fl, &eff, &live_fn) {
                Ok(pl) => {
                    out.loops.insert((fi, fl.pre.0 as usize), pl);
                }
                Err((Why::Order, Some(pl))) if fl.para => {
                    out.loops.insert((fi, fl.pre.0 as usize), pl);
                }
                Err((Why::Conflict(why), _)) if fl.para => out.errors.push((fi, fl.span, why)),
                Err((Why::Conflict(why), _)) => out.declined.push((fi, fl.span, why)),
                Err((Why::Order, _)) => out.declined.push((fi, fl.span, "it prints, reads input or calls C, so it stays in order (`para` would override)".into())),
            }
        }
    }
    out
}

fn name(f: &Function, l: u32) -> String {
    match &f.locals[l as usize].name {
        Some(n) => format!("'{}'", n),
        None => "a value".into(),
    }
}

fn analyse(p: &Program, f: &Function, fl: &ForLoop, eff: &[Effects], live_fn: &[HashSet<u32>]) -> Result<ParLoop, (Why, Option<ParLoop>)> {
    let conflict = |s: String| Err((Why::Conflict(s), None));
    if fl.perm {
        return conflict("its variable outlives it (`perm`), and a slice cannot say which value is the last".into());
    }
    let head = fl.head.0 as usize;
    let exit = fl.exit.0 as usize;
    let lb = loop_blocks(f, fl);
    let seen: HashSet<usize> = lb.iter().copied().collect();
    let exits_from: HashSet<usize> = fl.exits_from.iter().map(|b| b.0 as usize).collect();
    // A trip that leaves any other way — `break`, `return`, `fail` — is one
    // the others would have to know about.
    for &b in &lb {
        match &f.blocks[b].term {
            Terminator::Return | Terminator::Fail { .. } => return conflict("a trip can leave it (`return` or `fail`)".into()),
            t => {
                for s in successors(t) {
                    if s == exit && !exits_from.contains(&b) {
                        return conflict("a trip can leave it (`break`)".into());
                    }
                }
            }
        }
        // A trip that runs back into the loop head from some other loop,
        // or is reached from outside it: not a loop CCA lowered.
    }
    for (b, blk) in f.blocks.iter().enumerate() {
        if seen.contains(&b) || b == fl.pre.0 as usize {
            continue;
        }
        if successors(&blk.term).iter().any(|s| *s != head && seen.contains(s)) {
            return conflict("it is entered from more than one place".into());
        }
    }

    let counter = fl.counter.0;
    // What each local's writes and reads in the loop are.
    let mut defs: HashMap<u32, Vec<(usize, usize)>> = HashMap::new();
    let mut effects_here = Effects::default();
    for &b in &lb {
        let blk = &f.blocks[b];
        for (si, s) in blk.stmts.iter().enumerate() {
            let (_, d) = stmt_uses_defs(s);
            if let Some(d) = d {
                defs.entry(d).or_default().push((b, si));
            }
            match &s.kind {
                StmtKind::Task { .. } => return conflict("it starts tasks".into()),
                StmtKind::Asm { .. } => effects_here.outside = true,
                StmtKind::Op { op, .. } => {
                    effects_here.join(op_effects(op));
                }
                StmtKind::Assign { rvalue: Rvalue::ExternRead(_), .. } => effects_here.outside = true,
                _ => {}
            }
        }
        let (_, d) = term_uses_defs(&blk.term);
        if let Some(d) = d {
            defs.entry(d).or_default().push((b, blk.stmts.len()));
        }
        match &blk.term {
            Terminator::Call { callee, .. } => match callee {
                Callee::Extern(_) => effects_here.outside = true,
                Callee::Value(_) => return conflict("it calls a function held as a value, which KAYO\\n cannot see into".into()),
                Callee::Op(op) => {
                    effects_here.join(op_effects(op));
                }
                Callee::Fn(idx) => {
                    effects_here.join(eff[idx.0 as usize]);
                }
                Callee::Convert(..) => {}
            },
            Terminator::Whichever { .. } | Terminator::Join { .. } | Terminator::TaskOutcome { .. } => return conflict("it waits on tasks or channels".into()),
            _ => {}
        }
    }
    if effects_here.never {
        return conflict("it uses tasks, channels or `must`, which run in one order".into());
    }

    // Which locals an iteration keeps to itself: written in the loop, not
    // read before it writes them (not live at the head, within the loop),
    // and not read after the loop.
    let lset: HashSet<usize> = lb.iter().copied().collect();
    let live_loop = live_in(f, Some(&lset));
    let carried: &HashSet<u32> = &live_loop[head];
    let after: &HashSet<u32> = &live_fn[exit];
    let mut private: HashSet<u32> = HashSet::new();
    let mut folds_wanted: Vec<u32> = Vec::new();
    for (&l, _) in &defs {
        if l == counter || l == fl.stop.0 {
            continue;
        }
        if !carried.contains(&l) && !after.contains(&l) {
            private.insert(l);
        } else {
            folds_wanted.push(l);
        }
    }
    private.insert(counter);
    private.insert(fl.stop.0);

    // Locals that are the counter plus a constant: a position an iteration
    // reads or writes, as its offset from the counter. Iterations write
    // one list at one offset, and read it only there: `'xs'['k']` from
    // `'xs'['k' - 1]` is the iteration before's (reading it was planned
    // parallel before offsets were kept).
    let mut offset: HashMap<u32, i128> = [(counter, 0i128)].into_iter().collect();
    loop {
        let mut grew = false;
        for (&l, ds) in &defs {
            if offset.contains_key(&l) || ds.len() != 1 {
                continue;
            }
            let (b, si) = ds[0];
            let Some(s) = f.blocks[b].stmts.get(si) else { continue };
            let StmtKind::Assign { rvalue, .. } = &s.kind else { continue };
            let of = |o: &Operand| match o {
                Operand::Read(pl) if pl.proj.is_empty() => offset.get(&pl.local.0).copied(),
                _ => None,
            };
            let k = |o: &Operand| match o {
                Operand::Const(Const::Int(c)) => Some(*c),
                _ => None,
            };
            let got = match rvalue {
                Rvalue::Use(o) | Rvalue::Convert { operand: o, .. } => of(o),
                Rvalue::Binary { op: BinOp::Add, lhs, rhs } => of(lhs).zip(k(rhs)).or_else(|| k(lhs).zip(of(rhs)).map(|(c, d)| (d, c))).map(|(d, c)| d + c),
                Rvalue::Binary { op: BinOp::Sub, lhs, rhs } => of(lhs).zip(k(rhs)).map(|(d, c)| d - c),
                _ => None,
            };
            if let Some(o) = got {
                offset.insert(l, o);
                grew = true;
            }
        }
        if !grew {
            break;
        }
    }
    let at_offset = |pl: &Place| -> Option<i128> {
        match pl.proj.first() {
            Some(Proj::Index(Operand::Read(ix))) if ix.proj.is_empty() => offset.get(&ix.local.0).copied(),
            _ => None,
        }
    };

    // A borrowed local stands for what it borrows: the root it was lent from.
    let root = |l: u32| -> u32 {
        let mut cur = l;
        for _ in 0..16 {
            if !matches!(f.locals[cur as usize].kind, LocalKind::Lent { .. }) {
                break;
            }
            let mut src = None;
            let mut count = 0;
            for blk in &f.blocks {
                for s in &blk.stmts {
                    if let StmtKind::Assign { place, rvalue } = &s.kind {
                        if place.proj.is_empty() && place.local.0 == cur {
                            count += 1;
                            src = match rvalue {
                                Rvalue::Lend { place: p, .. } | Rvalue::Use(Operand::Read(p)) if p.proj.is_empty() => Some(p.local.0),
                                _ => None,
                            };
                        }
                    }
                }
            }
            match (count, src) {
                (1, Some(s)) => cur = s,
                _ => break,
            }
        }
        cur
    };

    // Writes to what is not the iteration's own: element writes at its own
    // position are fine; anything else is a conflict.
    let mut element_written: HashMap<u32, i128> = HashMap::new();
    let mut two_offsets: Option<u32> = None;
    let mut write_to = |pl: &Place, whole_ok: bool| -> Result<(), String> {
        let r = root(pl.local.0);
        let is_private_lend = private.contains(&pl.local.0) && matches!(f.locals[pl.local.0 as usize].kind, LocalKind::Lent { .. });
        if private.contains(&pl.local.0) && !is_private_lend {
            return Ok(());
        }
        if is_private_lend {
            // A lend made in the trip: fine when it lends the trip's own
            // element (`for 'x' in lend.mut 'xs'`) or something private.
            let bound = defs.get(&pl.local.0).and_then(|ds| ds.first()).and_then(|(b, si)| f.blocks[*b].stmts.get(*si));
            if let Some(Statement { kind: StmtKind::Assign { rvalue: Rvalue::Lend { place: src, .. }, .. }, .. }) = bound {
                if private.contains(&src.local.0) && !matches!(f.locals[src.local.0 as usize].kind, LocalKind::Lent { .. }) {
                    return Ok(());
                }
                if let Some(o) = at_offset(src) {
                    let r = root(src.local.0);
                    if *element_written.entry(r).or_insert(o) != o {
                        two_offsets = Some(r);
                    }
                    return Ok(());
                }
            }
            return Err(format!("{} lends something every iteration shares, and writes through it", name(f, pl.local.0)));
        }
        if let (false, Some(o)) = (whole_ok, at_offset(pl)) {
            if *element_written.entry(r).or_insert(o) != o {
                two_offsets = Some(r);
            }
            return Ok(());
        }
        Err(format!("every iteration writes {}", name(f, r)))
    };
    for &b in &lb {
        let blk = &f.blocks[b];
        for s in &blk.stmts {
            let r = match &s.kind {
                StmtKind::Assign { place, rvalue } => {
                    let lent = matches!(f.locals[place.local.0 as usize].kind, LocalKind::Lent { .. });
                    let binds = place.proj.is_empty()
                        && lent
                        && match rvalue {
                            Rvalue::Lend { .. } => true,
                            Rvalue::Use(Operand::Read(p)) => p.proj.is_empty() && matches!(f.locals[p.local.0 as usize].kind, LocalKind::Lent { .. }),
                            _ => false,
                        };
                    if !place.proj.is_empty() {
                        write_to(place, false)
                    } else if lent && !binds {
                        write_to(place, true)
                    } else if let Rvalue::Lend { place: src, mutable: true } = rvalue {
                        write_to(src, false).map_err(|_| format!("every iteration lends {} to change it", name(f, root(src.local.0))))
                    } else {
                        Ok(())
                    }
                }
                StmtKind::Op { op, args, dest } => {
                    let changes = matches!(op, Op::ListAppend | Op::ListRemove | Op::MapSet | Op::MapRemove | Op::SimdStore | Op::SimdStoreSome | Op::PtrWrite);
                    let mut r = Ok(());
                    if changes {
                        if let Some(pl) = args.first().and_then(|a| a.place()) {
                            let resizes = !matches!(op, Op::SimdStore | Op::SimdStoreSome | Op::PtrWrite);
                            r = write_to(pl, resizes);
                        }
                    }
                    if let Some(d) = dest {
                        if !d.proj.is_empty() && r.is_ok() {
                            r = write_to(d, false);
                        }
                    }
                    r
                }
                StmtKind::Drop(pl) if !private.contains(&pl.local.0) => Err(format!("{} is given back inside it", name(f, pl.local.0))),
                StmtKind::Asm { outs, lends, .. } => {
                    let mut r = Ok(());
                    for (pl, _) in outs {
                        if r.is_ok() {
                            r = write_to(pl, true);
                        }
                    }
                    for (pl, m, _) in lends {
                        if *m && r.is_ok() {
                            r = write_to(pl, true);
                        }
                    }
                    r
                }
                _ => Ok(()),
            };
            if let Err(e) = r {
                return conflict(e);
            }
        }
        if let Terminator::Call { args, dest, .. } = &blk.term {
            for a in args {
                let r = match a {
                    Arg::Lend { place, mutable: true } => write_to(place, false),
                    Arg::Give(place) => write_to(place, true),
                    _ => Ok(()),
                };
                if let Err(e) = r {
                    return conflict(e);
                }
            }
            if let Some(d) = dest {
                if !d.proj.is_empty() {
                    if let Err(e) = write_to(d, false) {
                        return conflict(e);
                    }
                }
            }
        }
    }
    if let Some(r) = two_offsets {
        return conflict(format!("iterations write {} at two different positions each", name(f, r)));
    }
    // A list some iteration writes an element of is read by each only at
    // its own position (and for its length).
    for &b in &lb {
        let blk = &f.blocks[b];
        let check = |pl: &Place| -> Result<(), String> {
            let r = root(pl.local.0);
            if let Some(&w) = element_written.get(&r) {
                if at_offset(pl) != Some(w) {
                    return Err(format!("an iteration reads {} away from its own position while others write it", name(f, r)));
                }
            }
            Ok(())
        };
        let mut places: Vec<Place> = Vec::new();
        let push_operand = |o: &Operand, v: &mut Vec<Place>| {
            if let Some(pl) = o.place() {
                v.push(pl.clone());
            }
        };
        for s in &blk.stmts {
            match &s.kind {
                StmtKind::Assign { rvalue, place } => {
                    match rvalue {
                        Rvalue::Use(o) | Rvalue::Unary { operand: o, .. } | Rvalue::Convert { operand: o, .. } => push_operand(o, &mut places),
                        Rvalue::Binary { lhs, rhs, .. } => {
                            push_operand(lhs, &mut places);
                            push_operand(rhs, &mut places);
                        }
                        Rvalue::Duplicate(pl) | Rvalue::Lend { place: pl, .. } => places.push(pl.clone()),
                        Rvalue::MakeList(os) => os.iter().for_each(|o| push_operand(o, &mut places)),
                        Rvalue::MakeStruct { fields, .. } => fields.iter().for_each(|o| push_operand(o, &mut places)),
                        Rvalue::MakeEnum { payload: Some(o), .. } => push_operand(o, &mut places),
                        _ => {}
                    }
                    let _ = place;
                }
                StmtKind::Op { op, args, .. } => {
                    let length = matches!(op, Op::ListLength | Op::ArrayLength);
                    if !length {
                        args.iter().for_each(|o| push_operand(o, &mut places));
                    }
                }
                _ => {}
            }
        }
        if let Terminator::Call { args, .. } = &blk.term {
            for a in args {
                if let Some(pl) = arg_place(a) {
                    places.push(pl.clone());
                }
            }
        }
        for pl in &places {
            // A whole lend of a written list, or a read at another place.
            let whole = pl.proj.is_empty();
            let r = root(pl.local.0);
            if element_written.contains_key(&r) && whole && !private.contains(&pl.local.0) {
                // The list itself handed somewhere: only its length is safe,
                // and that is `ListLength`, left out above.
                return conflict(format!("{} is handed on whole while iterations write its elements", name(f, r)));
            }
            if let Err(e) = check(pl) {
                return conflict(e);
            }
        }
    }

    // What is kept between trips must be a fold.
    let mut folds = Vec::new();
    for l in folds_wanted {
        match fold(f, &lb, &defs, l) {
            Some(mut fd) => {
                // The sequential total has to be one no check stops: every
                // site on the fold proven, so the slices' sum is that total.
                // Or the only one left is the store into 'acc', and the
                // total only moves one way: then the check the parent makes
                // once on the total stops the program exactly when the loop
                // would have, at the same site.
                let open: Vec<&SiteKind> = fd.skip.iter().filter_map(|(b, si)| match &f.blocks[*b].stmts[*si].kind {
                    StmtKind::Site(id) if !matches!(f.site(*id).verdict, Verdict::Proven { .. } | Verdict::Trusted) => Some(&f.site(*id).kind),
                    _ => None,
                }).collect();
                // A sum of integers may leave its store checked when its
                // terms keep one sign; a product, a bin or a deci needs
                // every site proven, so the loop in order never stops.
                let settled = open.is_empty() || (fd.kind == FoldKind::Int && fd.op == BinOp::Add && fd.one_signed && open.iter().all(|k| matches!(k, SiteKind::Narrow { .. })));
                if !settled {
                    let what = if fd.op == BinOp::Mul { "product" } else { "sum" };
                    return conflict(format!("the {} into {} is not proven to fit, so slices could not stop where the loop would", what, name(f, l)));
                }
                fd.wrap = fd.kind == FoldKind::Int && open.is_empty();
                folds.push(fd);
            }
            None => {
                return conflict(format!("{} is kept from one iteration to the next (or read after the loop) and is not a sum", name(f, l)));
            }
        }
    }
    for fd in &folds {
        private.insert(fd.acc.0);
    }

    // The work of one trip: its statements, a call ten, an inner loop's
    // blocks sixteen times.
    let inner_heads: HashSet<usize> = f.fors.iter().filter(|o| o.head != fl.head && seen.contains(&(o.head.0 as usize))).map(|o| o.head.0 as usize).collect();
    let mut inner: HashSet<usize> = HashSet::new();
    for o in f.fors.iter().filter(|o| inner_heads.contains(&(o.head.0 as usize))) {
        let mut q = vec![o.head.0 as usize];
        while let Some(b) = q.pop() {
            if !inner.insert(b) {
                continue;
            }
            for s in successors(&f.blocks[b].term) {
                if s != o.exit.0 as usize && seen.contains(&s) {
                    q.push(s);
                }
            }
        }
    }
    let mut work: u64 = 0;
    for &b in &lb {
        let blk = &f.blocks[b];
        let mut w = blk.stmts.iter().filter(|s| matches!(s.kind, StmtKind::Assign { .. } | StmtKind::Op { .. })).count() as u64;
        if matches!(blk.term, Terminator::Call { .. }) {
            w += 10;
        }
        if inner.contains(&b) {
            w *= 16;
        }
        work += w;
    }
    let pl = ParLoop { fl: fl.clone(), blocks: lb, private, folds, work: work.max(1) };
    if effects_here.outside {
        return Err((Why::Order, Some(pl)));
    }
    let _ = p;
    Ok(pl)
}

/// `acc = t` after `t = acc + x` (or `x`) in the same block — or the add
/// written straight into 'acc', as a deci's is — and 'acc' read nowhere
/// else in the loop: a fold each slice can take on its own. Whole numbers,
/// bins and decis of a plain type (a range type's bounds would need their
/// own check at the end, not yet built).
fn fold(f: &Function, lb: &[usize], defs: &HashMap<u32, Vec<(usize, usize)>>, acc: u32) -> Option<Fold> {
    let ty = &f.locals[acc as usize].ty;
    if ty.bound().is_some() {
        return None;
    }
    let kind = match ty.base() {
        Ty::Int(IntW::Fixed { .. }) if !ty.is_exact() => FoldKind::Int,
        Ty::Bin(crate::types::BinW::Fixed(_)) => FoldKind::Bin,
        Ty::Deci(bits) => FoldKind::Deci(*bits),
        _ => return None,
    };
    let mut adds = Vec::new();
    let mut skip: HashSet<(usize, usize)> = HashSet::new();
    let mut temps: HashSet<u32> = HashSet::new();
    let mut the_op: Option<BinOp> = None;
    // The sign every term keeps: Some(true) never below 0, Some(false)
    // never above, None either way or unknown.
    let mut sign: Option<Option<bool>> = None;
    let is_acc = |o: &Operand| matches!(o, Operand::Read(pl) if pl.proj.is_empty() && pl.local.0 == acc);
    for &(b, si) in defs.get(&acc)? {
        let blk = &f.blocks[b];
        let Some(Statement { kind: StmtKind::Assign { rvalue, .. }, .. }) = blk.stmts.get(si) else { return None };
        // Where the add is: this statement, or the temporary it stores.
        let ai = match rvalue {
            Rvalue::Binary { .. } => si,
            Rvalue::Use(Operand::Read(tp)) if tp.proj.is_empty() && f.locals[tp.local.0 as usize].name.is_none() => {
                let t = tp.local.0;
                let tdefs = defs.get(&t)?;
                if tdefs.len() != 1 || tdefs[0].0 != b || tdefs[0].1 >= si {
                    return None;
                }
                skip.insert((b, si));
                temps.insert(t);
                tdefs[0].1
            }
            _ => return None,
        };
        let Some(Statement { kind: StmtKind::Assign { rvalue: Rvalue::Binary { op, lhs, rhs }, .. }, .. }) = blk.stmts.get(ai) else { return None };
        if !matches!(op, BinOp::Add | BinOp::Mul) || the_op.as_ref().is_some_and(|o| o != op) {
            return None;
        }
        the_op = Some(op.clone());
        let (x, side) = if is_acc(lhs) && !uses(rhs, acc) {
            (rhs.clone(), 1)
        } else if is_acc(rhs) && !uses(lhs, acc) {
            (lhs.clone(), 0)
        } else {
            return None;
        };
        let this = f.operand_ranges.get(&(b as u32, ai as u32)).map(|r| r[side]).and_then(|(lo, hi)| if lo >= 0 { Some(true) } else if hi <= 0 { Some(false) } else { None });
        sign = Some(match sign {
            None => this,
            Some(prev) if prev == this => prev,
            Some(_) => None,
        });
        adds.push((b, ai, x));
    }
    // 'acc' and each temporary read nowhere else; the sites that name them go too.
    for &b in lb {
        for (si, s) in f.blocks[b].stmts.iter().enumerate() {
            if skip.contains(&(b, si)) || adds.iter().any(|(ab, ai, _)| *ab == b && *ai == si) {
                continue;
            }
            if let StmtKind::Site(id) = &s.kind {
                let mut u = Vec::new();
                f.site(*id).operands.iter().for_each(|o| operand_uses(o, &mut u));
                if u.iter().any(|l| *l == acc || temps.contains(l)) {
                    skip.insert((b, si));
                }
                continue;
            }
            let (u, _) = stmt_uses_defs(s);
            if u.iter().any(|l| *l == acc || temps.contains(l)) {
                return None;
            }
        }
        let (u, _) = term_uses_defs(&f.blocks[b].term);
        if u.iter().any(|l| *l == acc || temps.contains(l)) {
            return None;
        }
    }
    Some(Fold { acc: LocalId(acc), op: the_op?, kind, adds, skip, one_signed: matches!(sign, Some(Some(_))), wrap: false })
}

fn uses(o: &Operand, l: u32) -> bool {
    let mut u = Vec::new();
    operand_uses(o, &mut u);
    u.contains(&l)
}

/// A `for`'s blocks: from its head, never through its exit.
pub fn loop_blocks(f: &Function, fl: &ForLoop) -> Vec<usize> {
    let head = fl.head.0 as usize;
    let exit = fl.exit.0 as usize;
    let mut lb: Vec<usize> = vec![head];
    let mut seen: HashSet<usize> = [head].into_iter().collect();
    let mut i = 0;
    while i < lb.len() {
        let b = lb[i];
        i += 1;
        for s in successors(&f.blocks[b].term) {
            if s != exit && seen.insert(s) {
                lb.push(s);
            }
        }
    }
    lb
}
