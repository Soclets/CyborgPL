//! A check on the graph itself, run after every lowering. Not the
//! program's correctness — the *compiler's*: an edge to a block that does
//! not exist, a local nobody declared, a reachable block that falls off
//! the end. A bug here would make Precog prove something about a graph
//! that is not the program, which is the one failure the tiers cannot
//! catch, so it is worth finding at the moment it is built.

use std::collections::HashSet;

use super::*;

pub fn program(p: &Program) -> Result<(), Vec<String>> {
    let mut problems = Vec::new();
    for (i, f) in p.functions.iter().enumerate() {
        if let Err(mut e) = function(f, p.functions.len()) {
            for x in e.iter_mut() {
                *x = format!("f{} ({}): {}", i, f.name, x);
            }
            problems.append(&mut e);
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

pub fn function(f: &Function, n_functions: usize) -> Result<(), Vec<String>> {
    let mut p = Vec::new();
    let nb = f.blocks.len();
    let nl = f.locals.len();
    let ns = f.sites.len();

    let check_place = |pl: &Place, where_: &str, p: &mut Vec<String>| {
        if pl.local.0 as usize >= nl {
            p.push(format!("{}: local _{} is not declared", where_, pl.local.0));
        }
    };

    for (bi, b) in f.blocks.iter().enumerate() {
        let w = format!("b{}", bi);
        for s in &b.stmts {
            if s.span.0 as usize >= f.spans.len() {
                p.push(format!("{}: a statement points at span {} and there are {}", w, s.span.0, f.spans.len()));
            }
            match &s.kind {
                StmtKind::Assign { place, rvalue } => {
                    check_place(place, &w, &mut p);
                    for pl in rvalue_places(rvalue) {
                        check_place(&pl, &w, &mut p);
                    }
                    if let Rvalue::FnRef(i) | Rvalue::Closure { body: i, .. } = rvalue {
                        if i.0 as usize >= n_functions {
                            p.push(format!("{}: names f{}, and there are {} functions", w, i.0, n_functions));
                        }
                    }
                }
                StmtKind::Op { dest, args, .. } => {
                    if let Some(d) = dest {
                        check_place(d, &w, &mut p);
                    }
                    for a in args {
                        if let Some(pl) = a.place() {
                            check_place(pl, &w, &mut p);
                        }
                    }
                }
                StmtKind::Site(id) => {
                    if id.0 as usize >= ns {
                        p.push(format!("{}: site s{} does not exist", w, id.0));
                    }
                }
                StmtKind::Drop(pl) | StmtKind::EndLend(pl) => {
                    check_place(pl, &w, &mut p);
                    if !pl.is_local() {
                        p.push(format!("{}: a drop names part of a value ({:?}); only a whole value is freed", w, pl.proj));
                    }
                }
                StmtKind::Task { body, captures, .. } => {
                    if body.0 as usize >= n_functions {
                        p.push(format!("{}: starts f{}, and there are {} functions", w, body.0, n_functions));
                    }
                    for (_, pl) in captures {
                        check_place(pl, &w, &mut p);
                    }
                }
                StmtKind::Asm { ins, outs, lends, .. } => {
                    for (pl, _) in ins.iter().chain(outs) {
                        check_place(pl, &w, &mut p);
                    }
                    for (pl, _, _) in lends {
                        check_place(pl, &w, &mut p);
                    }
                }
                StmtKind::Nop => {}
            }
        }
        for t in successors(&b.term) {
            if t.0 as usize >= nb {
                p.push(format!("{}: an edge to b{}, and there are {} blocks", w, t.0, nb));
            }
        }
        if let Terminator::Call { callee: Callee::Fn(i), .. } = &b.term {
            if i.0 as usize >= n_functions {
                p.push(format!("{}: calls f{}, and there are {} functions", w, i.0, n_functions));
            }
        }
        if let Terminator::Trap(id) = &b.term {
            if id.0 as usize >= ns {
                p.push(format!("{}: traps at site s{}, which does not exist", w, id.0));
            }
        }
        if let Terminator::Fail { index, .. } = &b.term {
            if *index >= f.fails.len() {
                p.push(format!("{}: fails with reason {}, and this function lists {}", w, index, f.fails.len()));
            }
        }
    }

    // A block anyone can reach must say where it goes; a dropped function
    // has no body to go anywhere.
    let reachable = if f.dropped { Vec::new() } else { reachable(f) };
    for bi in &reachable {
        if matches!(f.blocks[*bi].term, Terminator::Unreachable) {
            p.push(format!("b{} can be reached and falls off the end", bi));
        }
    }
    for id in &f.params {
        if id.0 as usize >= nl {
            p.push(format!("a parameter names local _{}, which is not declared", id.0));
        }
    }
    if f.ret != Ty::Nothing && f.ret_local.is_none() {
        p.push("returns a value and has no slot to put it in".into());
    }

    if p.is_empty() {
        Ok(())
    } else {
        Err(p)
    }
}

pub fn successors(t: &Terminator) -> Vec<BlockId> {
    match t {
        Terminator::Goto(b) => vec![*b],
        Terminator::If { then, else_, .. } => vec![*then, *else_],
        Terminator::Switch { cases, .. } => cases.iter().map(|(_, b)| *b).collect(),
        Terminator::Call { ok, fail, .. } => {
            let mut v = vec![*ok];
            v.extend(fail.iter().map(|f| f.block));
            v
        }
        Terminator::Whichever { arms, closed } => {
            let mut v: Vec<BlockId> = arms.iter().map(|a| a.block).collect();
            v.extend(closed);
            v
        }
        Terminator::Join { ok, .. } => vec![*ok],
        Terminator::TaskOutcome { ok, fail, .. } => {
            let mut v = vec![*ok];
            v.extend(fail.iter().map(|f| f.block));
            v
        }
        Terminator::Return | Terminator::Fail { .. } | Terminator::Trap(_) | Terminator::Unreachable => Vec::new(),
    }
}

pub fn reachable(f: &Function) -> Vec<usize> {
    let mut seen = HashSet::new();
    let mut stack = vec![0usize];
    while let Some(b) = stack.pop() {
        if b >= f.blocks.len() || !seen.insert(b) {
            continue;
        }
        for s in successors(&f.blocks[b].term) {
            stack.push(s.0 as usize);
        }
    }
    let mut v: Vec<usize> = seen.into_iter().collect();
    v.sort_unstable();
    v
}

fn rvalue_places(r: &Rvalue) -> Vec<Place> {
    match r {
        Rvalue::Use(o) => o.place().cloned().into_iter().collect(),
        Rvalue::Duplicate(p) | Rvalue::Lend { place: p, .. } => vec![p.clone()],
        Rvalue::Unary { operand, .. } => operand.place().cloned().into_iter().collect(),
        Rvalue::Binary { lhs, rhs, .. } => lhs.place().into_iter().chain(rhs.place()).cloned().collect(),
        Rvalue::Convert { operand, .. } => operand.place().cloned().into_iter().collect(),
        Rvalue::MakeList(v) => v.iter().filter_map(|o| o.place().cloned()).collect(),
        Rvalue::MakeMap(v) => v.iter().flat_map(|(k, x)| k.place().into_iter().chain(x.place()).cloned().collect::<Vec<_>>()).collect(),
        Rvalue::MakeStruct { fields, .. } => fields.iter().filter_map(|o| o.place().cloned()).collect(),
        Rvalue::MakeEnum { payload, .. } => payload.iter().filter_map(|o| o.place().cloned()).collect(),
        Rvalue::Closure { captures, .. } => captures.iter().map(|(_, p)| p.clone()).collect(),
        Rvalue::MakeChannel { .. } | Rvalue::FnRef(_) | Rvalue::ExternRead(_) => Vec::new(),
    }
}
