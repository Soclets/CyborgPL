//! Precog's half of `closed` (owner, 2026-09-23, c): the task that made a
//! channel stops counting as someone who might send on it at the point where
//! it provably never touches it again — not at the end of its block.
//!
//! The runtime rule is "a `take` on an empty channel fails `closed` when
//! every task still holding it is itself waiting to take from it". The
//! maker holds its channel until it drops it, which is after the block's
//! tasks have all ended. So in the plainest producer/consumer shape — MAIN
//! makes a channel, lends it to a producer and a consumer, and waits at the
//! block's end — MAIN is a holder that is waiting on the *join*, never on a
//! take, and the consumer's last `take` waits for ever where the oracle says
//! `closed`. The decision names that case and its answer: where Precog
//! proves the last sender done, the close is emitted there.
//!
//! What is proven here is the simplest true thing: after this point, the
//! making function never names the channel again — not directly, and not
//! through a borrow of it held in another name. At that point it gives up
//! its hold (`Op::ChannelRelease`). A channel ever *given* away is left
//! alone: whoever received it holds it now, and lets go when they drop it.
//! A name assigned more than once is left alone too, since it may be a
//! second channel. Both refusals only mean the runtime counting stays as it
//! was — safe, never wrong.

use std::collections::{HashMap, HashSet};

use super::verify::successors;
use super::*;

pub fn release_early(p: &mut Program) {
    for f in &mut p.functions {
        let candidates: Vec<LocalId> = (0..f.locals.len())
            .map(|i| LocalId(i as u32))
            .filter(|l| {
                let d = &f.locals[l.0 as usize];
                matches!(d.kind, LocalKind::Owned) && matches!(d.ty.base(), crate::types::Ty::Channel(_))
            })
            .collect();
        for l in candidates {
            release_one(f, l);
        }
    }
}

fn place_locals(p: &Place, out: &mut Vec<LocalId>) {
    out.push(p.local);
    for pr in &p.proj {
        if let Proj::Index(o) = pr {
            operand_locals(o, out);
        }
    }
}

fn operand_locals(o: &Operand, out: &mut Vec<LocalId>) {
    if let Some(p) = o.place() {
        place_locals(p, out);
    }
}

fn arg_locals(a: &Arg, out: &mut Vec<LocalId>) {
    match a {
        Arg::Value(o) => operand_locals(o, out),
        Arg::Lend { place, .. } | Arg::Give(place) | Arg::Copy(place) | Arg::Spread { place, .. } => place_locals(place, out),
    }
}

fn rvalue_locals(rv: &Rvalue, out: &mut Vec<LocalId>) {
    match rv {
        Rvalue::Use(o) | Rvalue::Unary { operand: o, .. } | Rvalue::Convert { operand: o, .. } => operand_locals(o, out),
        Rvalue::Duplicate(p) | Rvalue::Lend { place: p, .. } => place_locals(p, out),
        Rvalue::Binary { lhs, rhs, .. } => {
            operand_locals(lhs, out);
            operand_locals(rhs, out);
        }
        Rvalue::MakeList(os) | Rvalue::MakeStruct { fields: os, .. } => os.iter().for_each(|o| operand_locals(o, out)),
        Rvalue::MakeMap(ps) => ps.iter().for_each(|(k, v)| {
            operand_locals(k, out);
            operand_locals(v, out);
        }),
        Rvalue::MakeEnum { payload, .. } => {
            if let Some(o) = payload {
                operand_locals(o, out);
            }
        }
        Rvalue::Closure { captures, .. } => captures.iter().for_each(|(_, p)| place_locals(p, out)),
        Rvalue::MakeChannel { .. } | Rvalue::FnRef(_) | Rvalue::ExternRead(_) => {}
    }
}

/// Every local a statement reads — not what it drops or stops lending.
fn stmt_reads(s: &Statement) -> Vec<LocalId> {
    let mut out = Vec::new();
    match &s.kind {
        StmtKind::Assign { place, rvalue } => {
            // Writing into part of a value reads the rest of it.
            if !place.proj.is_empty() {
                place_locals(place, &mut out);
            }
            rvalue_locals(rvalue, &mut out);
        }
        StmtKind::Op { dest, args, .. } => {
            args.iter().for_each(|o| operand_locals(o, &mut out));
            if let Some(d) = dest {
                if !d.proj.is_empty() {
                    place_locals(d, &mut out);
                }
            }
        }
        StmtKind::Site(_) => {}
        StmtKind::Task { captures, .. } => captures.iter().for_each(|(_, p)| place_locals(p, &mut out)),
        StmtKind::Asm { ins, outs, lends, .. } => {
            ins.iter().for_each(|(p, _)| place_locals(p, &mut out));
            lends.iter().for_each(|(p, _, _)| place_locals(p, &mut out));
            outs.iter().for_each(|(p, _)| place_locals(p, &mut out));
        }
        StmtKind::Drop(_) | StmtKind::EndLend(_) | StmtKind::Nop => {}
    }
    out
}

fn term_reads(t: &Terminator) -> Vec<LocalId> {
    let mut out = Vec::new();
    match t {
        Terminator::If { cond, .. } => operand_locals(cond, &mut out),
        Terminator::Switch { value, .. } => operand_locals(value, &mut out),
        Terminator::Call { callee, args, .. } => {
            if let Callee::Value(o) = callee {
                operand_locals(o, &mut out);
            }
            args.iter().for_each(|a| arg_locals(a, &mut out));
        }
        Terminator::Whichever { arms, .. } => arms.iter().for_each(|a| operand_locals(&a.channel, &mut out)),
        Terminator::Fail { payload: Some(o), .. } => operand_locals(o, &mut out),
        _ => {}
    }
    out
}

/// Does anything in this function hand the value in `l` away?
fn ever_given(f: &Function, l: LocalId) -> bool {
    let moved = |o: &Operand| matches!(o, Operand::Move(p) if p.local == l);
    for b in &f.blocks {
        for s in &b.stmts {
            match &s.kind {
                StmtKind::Assign { rvalue, .. } => {
                    let mut hit = false;
                    match rvalue {
                        Rvalue::Use(o) | Rvalue::Unary { operand: o, .. } | Rvalue::Convert { operand: o, .. } => hit |= moved(o),
                        Rvalue::Binary { lhs, rhs, .. } => hit |= moved(lhs) || moved(rhs),
                        Rvalue::MakeList(os) | Rvalue::MakeStruct { fields: os, .. } => hit |= os.iter().any(moved),
                        Rvalue::MakeMap(ps) => hit |= ps.iter().any(|(k, v)| moved(k) || moved(v)),
                        Rvalue::MakeEnum { payload: Some(o), .. } => hit |= moved(o),
                        Rvalue::Closure { captures, .. } => hit |= captures.iter().any(|(v, p)| *v == crate::ast::Verb::Give && p.local == l),
                        _ => {}
                    }
                    if hit {
                        return true;
                    }
                }
                StmtKind::Op { args, .. } => {
                    if args.iter().any(moved) {
                        return true;
                    }
                }
                StmtKind::Task { captures, .. } => {
                    if captures.iter().any(|(v, p)| *v == crate::ast::Verb::Give && p.local == l) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        match &b.term {
            Terminator::Call { args, .. } => {
                if args.iter().any(|a| matches!(a, Arg::Give(p) if p.local == l) || matches!(a, Arg::Value(o) if moved(o))) {
                    return true;
                }
            }
            Terminator::Fail { payload: Some(o), .. } if moved(o) => return true,
            _ => {}
        }
    }
    false
}

fn release_one(f: &mut Function, l: LocalId) {
    if ever_given(f, l) {
        return;
    }
    // Where the channel is put into its name: once, or not at all.
    let mut defs = Vec::new();
    for (bi, b) in f.blocks.iter().enumerate() {
        for (si, s) in b.stmts.iter().enumerate() {
            let target = match &s.kind {
                StmtKind::Assign { place, .. } => Some(place),
                StmtKind::Op { dest: Some(d), .. } => Some(d),
                _ => None,
            };
            if let Some(p) = target {
                if p.local == l && p.proj.is_empty() {
                    defs.push((bi, si));
                }
            }
        }
    }
    if defs.len() != 1 {
        return;
    }
    let def = defs[0];
    // Every name that holds a borrow of the channel counts as the channel.
    let mut names: HashSet<LocalId> = HashSet::new();
    names.insert(l);
    loop {
        let before = names.len();
        for b in &f.blocks {
            for s in &b.stmts {
                if let StmtKind::Assign { place, rvalue } = &s.kind {
                    let from = match rvalue {
                        Rvalue::Lend { place: p, .. } => Some(p.local),
                        Rvalue::Use(Operand::Read(p)) | Rvalue::Use(Operand::Move(p)) => Some(p.local),
                        _ => None,
                    };
                    if let Some(src) = from {
                        if names.contains(&src) && place.proj.is_empty() {
                            names.insert(place.local);
                        }
                    }
                }
            }
        }
        if names.len() == before {
            break;
        }
    }
    let n = f.blocks.len();
    // Per block: the last statement index that reads the channel (or n_stmts
    // for the terminator), and whether it reads it before the definition.
    let mut last_use: Vec<Option<usize>> = vec![None; n];
    let mut used_before_def: Vec<bool> = vec![false; n];
    for (bi, b) in f.blocks.iter().enumerate() {
        let mut seen_def = false;
        for (si, s) in b.stmts.iter().enumerate() {
            if (bi, si) == def {
                seen_def = true;
            }
            if stmt_reads(s).iter().any(|x| names.contains(x)) {
                last_use[bi] = Some(si);
                if !seen_def {
                    used_before_def[bi] = true;
                }
            }
        }
        if term_reads(&b.term).iter().any(|x| names.contains(x)) {
            last_use[bi] = Some(b.stmts.len());
            if !seen_def {
                used_before_def[bi] = true;
            }
        }
    }
    let succ: Vec<Vec<usize>> = f.blocks.iter().map(|b| successors(&b.term).iter().map(|s| s.0 as usize).collect()).collect();
    let mut preds: Vec<usize> = vec![0; n];
    for s in &succ {
        for &t in s {
            if t < n {
                preds[t] += 1;
            }
        }
    }
    // Live: the channel is read again later on some path from here.
    let mut live_in = vec![false; n];
    let mut live_out = vec![false; n];
    loop {
        let mut changed = false;
        for b in (0..n).rev() {
            let out = succ[b].iter().any(|&s| s < n && live_in[s]);
            let defines_here = def.0 == b;
            let inn = used_before_def[b] || (out && !defines_here) || (last_use[b].is_some() && !defines_here);
            if out != live_out[b] || inn != live_in[b] {
                live_out[b] = out;
                live_in[b] = inn;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let release = |span: SpanId| Statement { kind: StmtKind::Op { dest: None, op: Op::ChannelRelease, args: vec![Operand::Read(Place::local(l))] }, span };
    // Inside a block: after the last read, when nothing later needs it.
    let mut after_stmt: Vec<(usize, usize, SpanId)> = Vec::new();
    // On an edge: the block ends needing it no more on the way to `to`.
    let mut on_edge: Vec<(usize, usize)> = Vec::new();
    for b in 0..n {
        let reached = live_out[b] || last_use[b].is_some();
        if !reached {
            continue;
        }
        match last_use[b] {
            Some(i) if !live_out[b] => {
                if i < f.blocks[b].stmts.len() {
                    after_stmt.push((b, i, f.blocks[b].stmts[i].span));
                } else {
                    for &s in &succ[b] {
                        on_edge.push((b, s));
                    }
                }
            }
            _ => {
                if live_out[b] {
                    for &s in &succ[b] {
                        if s < n && !live_in[s] {
                            on_edge.push((b, s));
                        }
                    }
                }
            }
        }
    }
    after_stmt.sort_by(|a, b| b.cmp(a));
    for (b, i, span) in after_stmt {
        f.blocks[b].stmts.insert(i + 1, release(span));
    }
    let mut split: HashMap<(usize, usize), usize> = HashMap::new();
    for (b, s) in on_edge {
        let span = f.blocks[b].term_span;
        if preds[s] == 1 {
            f.blocks[s].stmts.insert(0, release(span));
            continue;
        }
        let nb = *split.entry((b, s)).or_insert_with(|| {
            f.blocks.push(BasicBlock { stmts: vec![release(span)], term: Terminator::Goto(BlockId(s as u32)), term_span: span });
            f.blocks.len() - 1
        });
        retarget(&mut f.blocks[b].term, s, nb);
    }
}

/// Point every edge of `t` that went to `from` at `to`.
fn retarget(t: &mut Terminator, from: usize, to: usize) {
    let fix = |b: &mut BlockId| {
        if b.0 as usize == from {
            *b = BlockId(to as u32);
        }
    };
    match t {
        Terminator::Goto(b) => fix(b),
        Terminator::If { then, else_, .. } => {
            fix(then);
            fix(else_);
        }
        Terminator::Switch { cases, .. } => cases.iter_mut().for_each(|(_, b)| fix(b)),
        Terminator::Call { ok, fail, .. } | Terminator::TaskOutcome { ok, fail, .. } => {
            fix(ok);
            fail.iter_mut().for_each(|a| fix(&mut a.block));
        }
        Terminator::Whichever { arms, closed } => {
            arms.iter_mut().for_each(|a| fix(&mut a.block));
            if let Some(c) = closed {
                fix(c);
            }
        }
        Terminator::Join { ok, .. } => fix(ok),
        _ => {}
    }
}
