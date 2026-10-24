//! A writer never goes away holding text (owner, 2026-10-23: "both, bro";
//! `design/strings.md`: "kayo refuses a writer that goes away unflushed").
//!
//! Walked once ownership has run, so a writer given away is no longer this
//! function's to drop. A print into a writer, or lending it mutably to a
//! function (which may print into it), may leave text in it; a flush
//! leaves none; handing it on hands on the duty. A drop of one that may
//! hold text on any way to it is refused.

use std::collections::HashSet;

use crate::diag::Diagnostic;
use crate::source::SourceMap;
use crate::types::Ty;

use super::*;
use super::verify::successors;

/// The prints that leave text in their buffer.
fn prints(name: &str) -> bool {
    name.starts_with("print.") && !matches!(name, "print.flush" | "print.writer" | "print.writer.stderr")
}

pub fn check(p: &Program, sources: &SourceMap) -> Vec<String> {
    let mut out = Vec::new();
    for f in &p.functions {
        let writers: HashSet<LocalId> = (0..f.locals.len()).map(|i| LocalId(i as u32)).filter(|l| matches!(f.locals[l.0 as usize].ty.base(), Ty::Writer) && matches!(f.locals[l.0 as usize].kind, LocalKind::Owned)).collect();
        if writers.is_empty() {
            continue;
        }
        walk(f, &writers, sources, &mut out);
    }
    out
}

/// The writer a print or flush works on: its first argument.
fn first_writer(args: &[Operand], writers: &HashSet<LocalId>) -> Option<LocalId> {
    match args.first() {
        Some(Operand::Read(pl) | Operand::Move(pl)) if pl.proj.is_empty() && writers.contains(&pl.local) => Some(pl.local),
        _ => None,
    }
}

fn walk(f: &Function, writers: &HashSet<LocalId>, sources: &SourceMap, out: &mut Vec<String>) {
    let n = f.blocks.len();
    let mut entry: Vec<Option<HashSet<LocalId>>> = vec![None; n];
    entry[0] = Some(HashSet::new());
    let mut work = vec![0usize];
    let mut reported: HashSet<LocalId> = HashSet::new();
    while let Some(b) = work.pop() {
        let mut held = entry[b].clone().unwrap_or_default();
        let bb = &f.blocks[b];
        for s in &bb.stmts {
            match &s.kind {
                StmtKind::Op { op: Op::Std(name), args, .. } => {
                    if let Some(w) = first_writer(args, writers) {
                        if prints(name) {
                            held.insert(w);
                        } else if name == "print.flush" {
                            held.remove(&w);
                        }
                    }
                }
                StmtKind::Assign { rvalue: Rvalue::Use(Operand::Move(pl)), .. } if pl.proj.is_empty() => {
                    held.remove(&pl.local);
                }
                StmtKind::Drop(pl) if pl.proj.is_empty() && held.contains(&pl.local) && reported.insert(pl.local) => {
                    let name = f.locals[pl.local.0 as usize].name.clone().unwrap_or_else(|| "out".into());
                    let d = Diagnostic::error(format!("${} goes away still holding text: `std::print.flush(lend.mut ${});` before its block ends (owner, 2026-10-23)", name, name)).named("writer_unflushed")
                        .at(f.span(f.locals[pl.local.0 as usize].span))
                        .fix(None, format!("std::print.flush(lend.mut ${});", name));
                    out.push(d.render(sources));
                }
                _ => {}
            }
        }
        match &bb.term {
            Terminator::Call { callee, args, .. } => {
                let ops: Vec<Operand> = args.iter().map(|a| match a {
                    Arg::Value(o) => o.clone(),
                    Arg::Lend { place, .. } | Arg::Copy(place) | Arg::Spread { place, .. } => Operand::Read(place.clone()),
                    Arg::Give(place) => Operand::Move(place.clone()),
                }).collect();
                match callee {
                    Callee::Op(Op::Std(name)) => {
                        if let Some(w) = first_writer(&ops, writers) {
                            if prints(name) {
                                held.insert(w);
                            } else if name == "print.flush" {
                                held.remove(&w);
                            }
                        }
                    }
                    _ => {
                        for a in args {
                            match a {
                                // It may print into it.
                                Arg::Lend { place, mutable: true } if place.proj.is_empty() && writers.contains(&place.local) => {
                                    held.insert(place.local);
                                }
                                Arg::Give(place) if place.proj.is_empty() => {
                                    held.remove(&place.local);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Terminator::Return => {}
            _ => {}
        }
        for s in successors(&bb.term) {
            let s = s.0 as usize;
            let grown = match &mut entry[s] {
                None => {
                    entry[s] = Some(held.clone());
                    true
                }
                Some(e) => {
                    let before = e.len();
                    e.extend(held.iter().copied());
                    e.len() != before
                }
            };
            if grown {
                work.push(s);
            }
        }
    }
}
