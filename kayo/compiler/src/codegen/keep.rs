//! Loops under `no_cheat` that LLVM may still vectorise (owner, 2026-10-01
//! and 2026-10-18: SIMD, inlining and cores are allowed; only skipping work
//! is not. Design claude, 2026-10-20: "A barrier that keeps the rounds and
//! still lets LLVM vectorise is what the rulings ask for").
//!
//! A barrier on every store in a loop keeps each round, and keeps LLVM from
//! vectorising it (bench sum ran 1.5× its cheat build). A loop instead has
//! its barriers at its edges:
//!
//! - **on the way in**, what the loop reads is made unknown, so its trip
//!   count and its values cannot be worked out while building;
//! - **each round**, a value the round makes and the next does not carry on
//!   (a term added, a variable written over) is added into a hidden sink,
//!   which LLVM vectorises as it does any sum, so no round's value can be
//!   dropped as unused;
//! - **on the way out**, what the loop wrote, and the sink, go through a
//!   barrier, so the rounds' work is needed;
//! - and LLVM's `replexitval` is `never` in a build with such a loop, so a
//!   value carried round to round (a sum) is never replaced by its closed
//!   form (`n x (n + 1) / 2`).
//!
//! Code under `no_cheat` outside a loop keeps a barrier on each store.

use std::collections::{HashMap, HashSet};

use crate::mir::*;

/// One function's loops under `no_cheat`.
#[derive(Default)]
pub(super) struct KeepLoops {
    /// Every block in such a loop.
    pub blocks: HashSet<usize>,
    /// Locals whose each round's value goes into the sink: written in such
    /// a loop and not carried on to the next round.
    pub mixed: HashSet<LocalId>,
    /// A loop's way in (its `pre` block): what the loop reads.
    pub pre: HashMap<usize, Vec<LocalId>>,
    /// A loop's way out (its `exit` block): what the loop wrote.
    pub exit: HashMap<usize, Vec<LocalId>>,
}

/// One use of a local inside a loop.
#[derive(Clone, Copy)]
enum Use {
    /// Read to work out the local this statement writes.
    Feeds(LocalId),
    /// Read to decide a branch.
    Branch,
    Other,
}

/// The local a statement writes whole, if any.
fn stmt_writes(s: &Statement) -> Option<LocalId> {
    match &s.kind {
        StmtKind::Assign { place, .. } if place.proj.is_empty() => Some(place.local),
        StmtKind::Op { dest: Some(d), .. } if d.proj.is_empty() => Some(d.local),
        _ => None,
    }
}

/// Every function's loops under `no_cheat`: a loop whose head is a block
/// under `no_cheat` (`for !cheat`, a `no_cheat` statement, a `no_cheat`
/// copy of a function).
pub(super) fn keep_loops(p: &Program, nocheat: &HashSet<(usize, usize)>) -> HashMap<usize, KeepLoops> {
    let mut out = HashMap::new();
    for (fi, f) in p.functions.iter().enumerate() {
        let steps: HashSet<LocalId> = f.fors.iter().filter_map(|fl| fl.step).collect();
        let mut k = KeepLoops::default();
        for fl in &f.fors {
            if !nocheat.contains(&(fi, fl.head.0 as usize)) {
                continue;
            }
            let blocks = crate::par::loop_blocks(f, fl);
            // What reads what: an edge from each local read to the local a
            // statement or call writes.
            let mut edges: HashMap<LocalId, HashSet<LocalId>> = HashMap::new();
            let mut reads: Vec<LocalId> = Vec::new();
            let mut writes: Vec<LocalId> = Vec::new();
            // Every use of a local in the loop: feeding the local a statement
            // writes, deciding a branch, or anything else.
            let mut uses: HashMap<LocalId, Vec<Use>> = HashMap::new();
            for &b in &blocks {
                let bb = &f.blocks[b];
                for s in &bb.stmts {
                    let r = crate::mir::channels::stmt_reads(s);
                    let w = stmt_writes(s);
                    for x in &r {
                        uses.entry(*x).or_default().push(w.map_or(Use::Other, Use::Feeds));
                    }
                    if let Some(w) = w {
                        for x in &r {
                            edges.entry(*x).or_default().insert(w);
                        }
                        writes.push(w);
                    }
                    reads.extend(r);
                }
                match &bb.term {
                    Terminator::If { cond, .. } => {
                        let mut r = Vec::new();
                        crate::mir::channels::operand_locals(cond, &mut r);
                        for x in r {
                            uses.entry(x).or_default().push(Use::Branch);
                        }
                    }
                    t => {
                        for x in crate::mir::channels::term_reads(t) {
                            uses.entry(x).or_default().push(Use::Other);
                        }
                    }
                }
                let r = crate::mir::channels::term_reads(&bb.term);
                if let Terminator::Call { dest: Some(d), .. } = &bb.term {
                    if d.proj.is_empty() {
                        for x in &r {
                            edges.entry(*x).or_default().insert(d.local);
                        }
                        writes.push(d.local);
                    }
                }
                reads.extend(r);
            }
            // A local is carried when what it is written from leads back
            // to it: a round's value the next round starts from.
            let carried = |w: LocalId| -> bool {
                let mut seen = HashSet::new();
                let mut work: Vec<LocalId> = edges.get(&w).map(|s| s.iter().copied().collect()).unwrap_or_default();
                while let Some(x) = work.pop() {
                    if x == w {
                        return true;
                    }
                    if seen.insert(x) {
                        work.extend(edges.get(&x).into_iter().flatten().copied());
                    }
                }
                false
            };
            // How the loop counts, not its work (the room left before the
            // stop, whether to go round, the next count): a value whose every
            // use in the loop decides a branch, sets the counter, or feeds
            // another such value. Kept out of the sink, so LLVM still sees a
            // plain count. A value the loop never uses is what it makes.
            let mut counting: HashSet<LocalId> = writes.iter().copied().filter(|w| uses.get(w).is_some_and(|u| !u.is_empty())).collect();
            loop {
                let before = counting.len();
                let keep: Vec<LocalId> = counting.iter().copied().filter(|l| {
                    uses[l].iter().all(|u| match u {
                        Use::Branch => true,
                        Use::Feeds(w) => *w == fl.counter || counting.contains(w),
                        Use::Other => false,
                    })
                }).collect();
                counting = keep.into_iter().collect();
                if counting.len() == before {
                    break;
                }
            }
            counting.insert(fl.counter);
            for &w in &writes {
                if !steps.contains(&w) && !counting.contains(&w) && !carried(w) {
                    k.mixed.insert(w);
                }
            }
            let mut ins: Vec<LocalId> = reads.into_iter().filter(|l| !steps.contains(l)).collect();
            ins.sort_by_key(|l| l.0);
            ins.dedup();
            let mut outs: Vec<LocalId> = writes.into_iter().filter(|l| !steps.contains(l)).collect();
            outs.sort_by_key(|l| l.0);
            outs.dedup();
            k.pre.entry(fl.pre.0 as usize).or_default().extend(ins);
            k.exit.entry(fl.exit.0 as usize).or_default().extend(outs);
            k.blocks.extend(blocks);
        }
        if !k.blocks.is_empty() {
            out.insert(fi, k);
        }
    }
    out
}
