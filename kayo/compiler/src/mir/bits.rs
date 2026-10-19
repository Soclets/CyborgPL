//! A `list[bool]` held as bits (owner, 2026-10-18: "Pack a `list[bool]` into
//! bits"): eight to a byte, eight times less memory to walk, the same work
//! done — so it holds under `no_cheat` too.
//!
//! Only where nothing can tell: a list a function makes with `list.fill`,
//! reads and writes by position, asks the length of, and drops. Handed to a
//! call, lent, copied, appended to, walked, sliced, captured, given back or
//! read whole in any other way, it stays one byte per element, since every
//! other operation reads the bytes.

use std::collections::HashSet;

use super::*;
use crate::types::{SeqKind, Ty};

/// Every `(function, local)` held as bits.
pub fn bit_lists(p: &Program) -> HashSet<(usize, u32)> {
    let mut out = HashSet::new();
    // `KAYO_BITS=0` keeps every list a byte an element, to compare.
    if std::env::var("KAYO_BITS").is_ok_and(|v| v == "0") {
        return out;
    }
    for (fi, f) in p.functions.iter().enumerate() {
        if f.dropped {
            continue;
        }
        for (li, l) in f.locals.iter().enumerate() {
            let is_bool_list = matches!(l.ty.base(), Ty::Seq { elem, kind: SeqKind::List } if elem.base() == &Ty::Bool);
            let li = li as u32;
            if !is_bool_list || l.kind != LocalKind::Owned || f.params.iter().any(|p| p.0 == li) || f.ret_local.is_some_and(|r| r.0 == li) || f.captures.iter().any(|c| c.0 == li) {
                continue;
            }
            if only_by_position(f, li) {
                out.insert((fi, li));
            }
        }
    }
    out
}

/// Whether every mention of `l` in `f` is one a bit can stand for.
fn only_by_position(f: &Function, l: u32) -> bool {
    let mut filled = false;
    for b in &f.blocks {
        for s in &b.stmts {
            match &s.kind {
                // Made by `list.fill`, the length asked, dropped.
                StmtKind::Op { dest: Some(d), op: Op::ListFill, args } if d.local.0 == l && d.proj.is_empty() => {
                    if !args.iter().all(|o| fine_operand(o, l)) {
                        return false;
                    }
                    filled = true;
                }
                StmtKind::Op { op: Op::ListLength, args, dest } if matches!(args.as_slice(), [o] if o.place().is_some_and(|p| p.local.0 == l && p.proj.is_empty())) => {
                    if dest.as_ref().is_some_and(|d| !fine_place(d, l)) {
                        return false;
                    }
                }
                StmtKind::Drop(p) if p.local.0 == l && p.proj.is_empty() => {}
                StmtKind::Assign { place, rvalue } => {
                    if !fine_place(place, l) || !fine_rvalue(rvalue, l) {
                        return false;
                    }
                }
                StmtKind::Op { dest, args, .. } => {
                    if dest.as_ref().is_some_and(|d| !fine_place(d, l)) || !args.iter().all(|o| fine_operand(o, l)) {
                        return false;
                    }
                }
                StmtKind::Drop(p) | StmtKind::EndLend(p) => {
                    if !fine_place(p, l) {
                        return false;
                    }
                }
                StmtKind::Task { captures, .. } => {
                    if captures.iter().any(|(_, p)| mentions(p, l)) {
                        return false;
                    }
                }
                StmtKind::Asm { ins, outs, lends, .. } => {
                    if ins.iter().any(|(p, _)| mentions(p, l)) || outs.iter().any(|(p, _)| mentions(p, l)) || lends.iter().any(|(p, _, _)| mentions(p, l)) {
                        return false;
                    }
                }
                StmtKind::Site(_) | StmtKind::Nop => {}
            }
        }
        let fine = match &b.term {
            Terminator::If { cond, .. } => fine_operand(cond, l),
            Terminator::Switch { value, .. } => fine_operand(value, l),
            Terminator::Call { callee, args, dest, fail, .. } => {
                let callee_fine = match callee {
                    Callee::Value(o) => fine_operand(o, l),
                    _ => true,
                };
                callee_fine
                    && args.iter().all(|a| match a {
                        Arg::Value(o) => fine_operand(o, l),
                        Arg::Lend { place, .. } | Arg::Give(place) | Arg::Copy(place) | Arg::Spread { place, .. } => !mentions(place, l),
                    })
                    // A call's answer and a failure's payload are stored
                    // whole, as bytes: never into a bit.
                    && dest.as_ref().is_none_or(|d| !mentions(d, l))
                    && !fail.iter().any(|a| a.payload.as_ref().is_some_and(|p| mentions(p, l)))
            }
            Terminator::TaskOutcome { fail, .. } => !fail.iter().any(|a| a.payload.as_ref().is_some_and(|p| mentions(p, l))),
            Terminator::Fail { payload: Some(o), .. } => fine_operand(o, l),
            Terminator::Whichever { arms, .. } => arms.iter().all(|a| fine_operand(&a.channel, l) && !mentions(&a.dest, l)),
            _ => true,
        };
        if !fine {
            return false;
        }
    }
    filled
}

/// A place naming `l`, at all.
fn mentions(p: &Place, l: u32) -> bool {
    p.local.0 == l || p.proj.iter().any(|pr| matches!(pr, Proj::Index(o) if o.place().is_some_and(|q| mentions(q, l))))
}

/// A place that is not `l` itself: `l` by position, or something else.
fn fine_place(p: &Place, l: u32) -> bool {
    if p.local.0 == l {
        return matches!(p.proj.as_slice(), [Proj::Index(o)] if fine_operand(o, l));
    }
    p.proj.iter().all(|pr| match pr {
        Proj::Index(o) => fine_operand(o, l),
        _ => true,
    })
}

fn fine_operand(o: &Operand, l: u32) -> bool {
    o.place().is_none_or(|p| fine_place(p, l))
}

fn fine_rvalue(rv: &Rvalue, l: u32) -> bool {
    match rv {
        Rvalue::Use(o) | Rvalue::Unary { operand: o, .. } | Rvalue::Convert { operand: o, .. } => fine_operand(o, l),
        Rvalue::Binary { lhs, rhs, .. } => fine_operand(lhs, l) && fine_operand(rhs, l),
        // Copying or lending it reads the whole list's bytes.
        Rvalue::Duplicate(p) | Rvalue::Lend { place: p, .. } => !mentions(p, l),
        Rvalue::MakeList(os) => os.iter().all(|o| fine_operand(o, l)),
        Rvalue::MakeMap(kv) => kv.iter().all(|(k, v)| fine_operand(k, l) && fine_operand(v, l)),
        Rvalue::MakeStruct { fields, .. } => fields.iter().all(|o| fine_operand(o, l)),
        Rvalue::MakeEnum { payload, .. } => payload.as_ref().is_none_or(|o| fine_operand(o, l)),
        Rvalue::Closure { captures, .. } => !captures.iter().any(|(_, p)| mentions(p, l)),
        Rvalue::MakeChannel { .. } | Rvalue::FnRef(_) | Rvalue::ExternRead(_) => true,
    }
}
