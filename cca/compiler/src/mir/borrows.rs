//! While something borrows a value, the value stays as it is.
//!
//! Borrow checking is Rust-style (`core-features.md`): a borrow lasts until
//! its last use, not until its name goes out of scope, and while it lasts
//! what it borrows is not given away, changed, or lent `mut` — nor, for a
//! `lent.mut` borrow, used at all. The ownership walk follows moves; this
//! follows borrows, which is a liveness question, so it is its own pass.
//!
//! A borrow is a lent local: made by `lend`, by another borrow, or handed
//! back by a function (`-> lent@x T`, owner 2026-09-29), in which case it
//! borrows the arguments of the parameters its group names. A `for` over
//! `lend 'xs'` is one too, so giving `'xs'` away inside the loop — which
//! freed the list the loop was still walking — is refused here.

use std::collections::{HashMap, HashSet};

use crate::diag::Diagnostic;
use crate::source::SourceMap;

use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Acc {
    Read,
    Move,
    Write,
    Lend,
    LendMut,
}

/// Every place a statement touches, and how.
fn operand_acc(o: &Operand, out: &mut Vec<(Place, Acc)>) {
    match o {
        Operand::Read(p) => place_acc(p, Acc::Read, out),
        Operand::Move(p) => place_acc(p, Acc::Move, out),
        Operand::Const(_) => {}
    }
}

fn place_acc(p: &Place, how: Acc, out: &mut Vec<(Place, Acc)>) {
    for pr in &p.proj {
        if let Proj::Index(o) = pr {
            operand_acc(o, out);
        }
    }
    out.push((p.clone(), how));
}

fn rvalue_acc(rv: &Rvalue, out: &mut Vec<(Place, Acc)>) {
    match rv {
        Rvalue::Use(o) | Rvalue::Unary { operand: o, .. } | Rvalue::Convert { operand: o, .. } => operand_acc(o, out),
        Rvalue::Duplicate(p) => place_acc(p, Acc::Read, out),
        Rvalue::Lend { place, mutable } => place_acc(place, if *mutable { Acc::LendMut } else { Acc::Lend }, out),
        Rvalue::Binary { lhs, rhs, .. } => {
            operand_acc(lhs, out);
            operand_acc(rhs, out);
        }
        Rvalue::MakeList(os) | Rvalue::MakeStruct { fields: os, .. } => os.iter().for_each(|o| operand_acc(o, out)),
        Rvalue::MakeMap(ps) => ps.iter().for_each(|(k, v)| {
            operand_acc(k, out);
            operand_acc(v, out);
        }),
        Rvalue::MakeEnum { payload: Some(o), .. } => operand_acc(o, out),
        Rvalue::Closure { captures, .. } => captures.iter().for_each(|(v, p)| {
            let how = match v {
                crate::ast::Verb::Give => Acc::Move,
                crate::ast::Verb::LendMut => Acc::LendMut,
                crate::ast::Verb::Lend => Acc::Lend,
                _ => Acc::Read,
            };
            place_acc(p, how, out)
        }),
        _ => {}
    }
}

fn arg_acc(a: &Arg, out: &mut Vec<(Place, Acc)>) {
    match a {
        Arg::Value(o) => operand_acc(o, out),
        Arg::Lend { place, mutable } => place_acc(place, if *mutable { Acc::LendMut } else { Acc::Lend }, out),
        Arg::Give(p) => place_acc(p, Acc::Move, out),
        Arg::Copy(p) | Arg::Spread { place: p, .. } => place_acc(p, Acc::Read, out),
    }
}

/// What a statement touches, and the local it binds (a whole-local write).
fn stmt_acc(s: &StmtKind) -> (Vec<(Place, Acc)>, Option<LocalId>) {
    let mut out = Vec::new();
    let mut def = None;
    match s {
        StmtKind::Assign { place, rvalue } => {
            rvalue_acc(rvalue, &mut out);
            if place.proj.is_empty() {
                def = Some(place.local);
            } else {
                place_acc(place, Acc::Write, &mut out);
            }
        }
        StmtKind::Op { dest, args, op } => {
            // An operation that changes what it is given — an append, a
            // removal, a store — changes its first argument (check claude,
            // 2026-09-29: an append slipped past a borrow still in use).
            let changes = matches!(op, Op::ListAppend | Op::ListRemove | Op::MapSet | Op::MapRemove | Op::SimdStore | Op::SimdStoreSome);
            for (i, o) in args.iter().enumerate() {
                match o {
                    Operand::Read(p) | Operand::Move(p) if changes && i == 0 => place_acc(p, Acc::Write, &mut out),
                    o => operand_acc(o, &mut out),
                }
            }
            if let Some(d) = dest {
                if d.proj.is_empty() {
                    def = Some(d.local);
                } else {
                    place_acc(d, Acc::Write, &mut out);
                }
            }
        }
        StmtKind::Task { captures, .. } => captures.iter().for_each(|(v, p)| {
            let how = match v {
                crate::ast::Verb::Give => Acc::Move,
                crate::ast::Verb::LendMut => Acc::LendMut,
                crate::ast::Verb::Lend => Acc::Lend,
                _ => Acc::Read,
            };
            place_acc(p, how, &mut out)
        }),
        StmtKind::Asm { ins, outs, lends, .. } => {
            ins.iter().for_each(|(p, _)| place_acc(p, Acc::Read, &mut out));
            lends.iter().for_each(|(p, m, _)| place_acc(p, if *m { Acc::LendMut } else { Acc::Lend }, &mut out));
            outs.iter().for_each(|(p, _)| place_acc(p, Acc::Write, &mut out));
        }
        StmtKind::Drop(_) | StmtKind::EndLend(_) | StmtKind::Site(_) | StmtKind::Nop => {}
    }
    (out, def)
}

fn term_acc(t: &Terminator, ret: Option<LocalId>) -> (Vec<(Place, Acc)>, Option<LocalId>) {
    let mut out = Vec::new();
    let mut def = None;
    match t {
        Terminator::If { cond, .. } => operand_acc(cond, &mut out),
        Terminator::Switch { value, .. } => operand_acc(value, &mut out),
        Terminator::Call { callee, args, dest, .. } => {
            if let Callee::Value(o) = callee {
                operand_acc(o, &mut out);
            }
            args.iter().for_each(|a| arg_acc(a, &mut out));
            if let Some(d) = dest {
                if d.proj.is_empty() {
                    def = Some(d.local);
                } else {
                    place_acc(d, Acc::Write, &mut out);
                }
            }
        }
        Terminator::Whichever { arms, .. } => arms.iter().for_each(|a| operand_acc(&a.channel, &mut out)),
        Terminator::Return => {
            if let Some(r) = ret {
                out.push((Place::local(r), Acc::Read));
            }
        }
        Terminator::Fail { payload: Some(o), .. } => operand_acc(o, &mut out),
        _ => {}
    }
    (out, def)
}

/// Where borrow `l` is next used after step `i` of block `b` (the steps are
/// the statements, then the terminator): the nearest along the graph.
fn next_use(f: &Function, b: usize, i: usize, l: LocalId) -> Option<SpanId> {
    let uses = |acc: &[(Place, Acc)]| acc.iter().any(|(p, _)| p.local == l);
    let block_steps = |bi: usize| -> Vec<(Vec<(Place, Acc)>, SpanId)> {
        let blk = &f.blocks[bi];
        let mut v: Vec<(Vec<(Place, Acc)>, SpanId)> = blk.stmts.iter().map(|s| (stmt_acc(&s.kind).0, s.span)).collect();
        v.push((term_acc(&blk.term, f.ret_local).0, blk.term_span));
        v
    };
    for (acc, sp) in block_steps(b).into_iter().skip(i + 1) {
        if uses(&acc) {
            return Some(sp);
        }
    }
    let mut seen: HashSet<usize> = HashSet::new();
    let mut queue: std::collections::VecDeque<usize> = super::verify::successors(&f.blocks[b].term).into_iter().map(|s| s.0 as usize).collect();
    while let Some(bi) = queue.pop_front() {
        if !seen.insert(bi) {
            continue;
        }
        for (acc, sp) in block_steps(bi) {
            if uses(&acc) {
                return Some(sp);
            }
        }
        queue.extend(super::verify::successors(&f.blocks[bi].term).into_iter().map(|s| s.0 as usize));
    }
    None
}

pub fn check(p: &Program, sources: &SourceMap) -> Vec<String> {
    let mut problems = Vec::new();
    for fi in 0..p.functions.len() {
        check_fn(p, fi, sources, &mut problems);
    }
    problems
}

fn check_fn(p: &Program, fi: usize, sources: &SourceMap, problems: &mut Vec<String>) {
    let f = &p.functions[fi];
    let is_lent = |l: LocalId| matches!(f.locals[l.0 as usize].kind, LocalKind::Lent { .. }) && !f.params.contains(&l);
    // What each borrow borrows: the owners' locals, through other borrows.
    let mut src: HashMap<LocalId, HashSet<LocalId>> = HashMap::new();
    let mut direct: Vec<(LocalId, LocalId)> = Vec::new();
    for b in &f.blocks {
        for s in &b.stmts {
            if let StmtKind::Assign { place, rvalue } = &s.kind {
                if place.proj.is_empty() && is_lent(place.local) {
                    match rvalue {
                        Rvalue::Lend { place: from, .. } => direct.push((place.local, from.local)),
                        Rvalue::Use(o) => {
                            if let Some(from) = o.place() {
                                direct.push((place.local, from.local));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        if let Terminator::Call { callee: Callee::Fn(g), args, dest: Some(d), .. } = &b.term {
            if d.proj.is_empty() && is_lent(d.local) {
                if let Some((_, deps)) = &p.functions[g.0 as usize].ret_lent {
                    for i in deps {
                        if let Some(Arg::Lend { place, .. }) = args.get(*i) {
                            direct.push((d.local, place.local));
                        }
                    }
                }
            }
        }
    }
    for (l, from) in &direct {
        src.entry(*l).or_default().insert(*from);
    }
    loop {
        let mut grew = false;
        for (l, from) in &direct {
            let extra: Vec<LocalId> = src.get(from).map(|s| s.iter().copied().collect()).unwrap_or_default();
            let set = src.entry(*l).or_default();
            for x in extra {
                grew |= set.insert(x);
            }
        }
        if !grew {
            break;
        }
    }
    if src.is_empty() {
        return;
    }
    let borrows: Vec<LocalId> = src.keys().copied().collect();
    // Liveness of the borrows, backwards to a fixpoint.
    let n = f.blocks.len();
    let mut live_in: Vec<HashSet<LocalId>> = vec![HashSet::new(); n];
    let succ: Vec<Vec<BlockId>> = f.blocks.iter().map(|b| super::verify::successors(&b.term)).collect();
    let step = |live: &mut HashSet<LocalId>, acc: &[(Place, Acc)], def: Option<LocalId>| {
        if let Some(d) = def {
            live.remove(&d);
        }
        for (pl, _) in acc {
            if src.contains_key(&pl.local) {
                live.insert(pl.local);
            }
        }
    };
    loop {
        let mut changed = false;
        for bi in (0..n).rev() {
            let mut live: HashSet<LocalId> = succ[bi].iter().flat_map(|s| live_in[s.0 as usize].iter().copied()).collect();
            let (ta, td) = term_acc(&f.blocks[bi].term, f.ret_local);
            step(&mut live, &ta, td);
            for s in f.blocks[bi].stmts.iter().rev() {
                let (a, d) = stmt_acc(&s.kind);
                step(&mut live, &a, d);
            }
            if live != live_in[bi] {
                live_in[bi] = live;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    // Now forwards through each block, with what is live after each step.
    let mut reported: HashSet<(LocalId, SpanId)> = HashSet::new();
    for bi in 0..n {
        let mut after: Vec<HashSet<LocalId>> = Vec::new();
        let mut live: HashSet<LocalId> = succ[bi].iter().flat_map(|s| live_in[s.0 as usize].iter().copied()).collect();
        let (ta, td) = term_acc(&f.blocks[bi].term, f.ret_local);
        after.push(live.clone());
        step(&mut live, &ta, td);
        for s in f.blocks[bi].stmts.iter().rev() {
            after.push(live.clone());
            let (a, d) = stmt_acc(&s.kind);
            step(&mut live, &a, d);
        }
        after.reverse();
        // after[i] is live after statement i; the last is after the terminator.
        let mut steps: Vec<(Vec<(Place, Acc)>, Option<LocalId>, SpanId)> = f.blocks[bi].stmts.iter().map(|s| {
            let (a, d) = stmt_acc(&s.kind);
            (a, d, s.span)
        }).collect();
        let (ta, td) = term_acc(&f.blocks[bi].term, f.ret_local);
        steps.push((ta, td, f.blocks[bi].term_span));
        for (i, (acc, def, span)) in steps.iter().enumerate() {
            for (pl, how) in acc {
                for l in &borrows {
                    if Some(*l) == *def || *l == pl.local || !after[i].contains(l) || !src[l].contains(&pl.local) {
                        continue;
                    }
                    let exclusive = matches!(f.locals[l.0 as usize].kind, LocalKind::Lent { mutable: true });
                    let what = match how {
                        Acc::Move => "given away",
                        Acc::Write => "changed",
                        Acc::LendMut => "lent `mut`",
                        Acc::Read | Acc::Lend if exclusive => "used",
                        _ => continue,
                    };
                    let Some(owner) = f.locals[pl.local.0 as usize].name.clone() else { continue };
                    if !reported.insert((pl.local, *span)) {
                        continue;
                    }
                    // Reported at the change, naming the later use (owner,
                    // 2026-09-29: a), as Rust does.
                    let later = next_use(f, bi, i, *l).map(|sp| {
                        let s = f.span(sp);
                        sources.get(s.file).line_col(s.start).0
                    });
                    let kind = if exclusive { "`lent.mut` " } else { "" };
                    let tail = match (f.locals[l.0 as usize].name.clone(), later) {
                        (Some(n), Some(line)) => format!("'{}' still borrows it {}and is used on line {}", n, kind, line),
                        (Some(n), None) => format!("'{}' still borrows it {}and is used after this", n, kind),
                        (None, Some(line)) => format!("the loop on line {} is still going through it", line),
                        (None, None) => "a borrow of it is still in use".to_string(),
                    };
                    let mut d = Diagnostic::error(format!("'{}' cannot be {} here: {}", owner, what, tail)).at(f.span(*span));
                    d = d.why("while a borrow is in use, what it borrows stays as it is (Rust-style borrowing, `core-features.md`)");
                    if f.name != "MAIN" {
                        d = d.place(format!("in `{}`", f.name));
                    }
                    problems.push(d.render(sources));
                }
            }
        }
    }
}
