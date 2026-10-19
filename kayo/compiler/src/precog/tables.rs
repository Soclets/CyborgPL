//! Precog's tables (owner, 2026-10-17; limits 2026-10-19, `design/precog.md`
//! § Tables): a function Precog proves pure, whose arguments can take few
//! enough values, has every answer worked out while building, and becomes a
//! lookup in a table of them ("compile time is free").
//!
//! How big a table may be is the programmer's: `table_bytes N` in front of
//! the `fn`, else the target's `table_bytes` in `CyJet.cyr`, else 65,536.
//! A table bigger than its limit is not made, and nothing else changes.
//!
//! Each answer comes from Precog running the function on that one argument
//! tuple, as layer 1 runs a program whose inputs are known; the runs are dry,
//! so they leave the program's verdicts, folds and ranges as they were. A
//! table is made only when every answer comes out as one exact value and
//! every check in the function, and in what it calls, is proven: an input a
//! call can bring then never fails, and an entry no call can reach is never
//! read. A `no_cheat` copy is never a table (owner, 2026-10-18): its work
//! really runs.

use std::collections::HashSet;

use super::*;

/// The default limit, in bytes (design claude's number, `precog.md`).
pub const DEFAULT_TABLE_BYTES: u64 = 65_536;

/// Argument tuples worked out at most, whatever the limit: past this the
/// work outgrows what a table saves.
const MAX_ENTRIES: u64 = 65_536;

/// A function made a table.
#[derive(Debug, Clone)]
pub struct Table {
    pub function: usize,
    /// Each parameter's lowest and highest value; the entries run through
    /// the first parameter slowest.
    pub ranges: Vec<(i128, i128)>,
    pub entries: Vec<Const>,
    /// Bytes an entry takes, and the whole table.
    pub entry_bytes: u64,
    pub bytes: u64,
    /// The limit, and where it came from.
    pub limit: u64,
    pub limit_from: String,
    /// It and every function it calls.
    pub closure: Vec<usize>,
}

/// The tables whose function, and everything it calls, has every check
/// proven: an input a call brings then never fails.
pub(super) fn keep_proven(p: &Program, (made, mut not): (Vec<Table>, Vec<NotTable>), proven: &HashSet<(usize, u32)>) -> (Vec<Table>, Vec<NotTable>) {
    let all_proven = |g: usize| (0..p.functions[g].sites.len()).all(|si| proven.contains(&(g, si as u32)));
    let mut kept = Vec::new();
    for t in made {
        if t.closure.iter().all(|g| all_proven(*g)) {
            kept.push(t);
        } else {
            not.push(NotTable { function: t.function, why: "a check in it, or in what it calls, is not proven".to_string() });
        }
    }
    (kept, not)
}

/// A function that could have been a table and was not, and why.
#[derive(Debug, Clone)]
pub struct NotTable {
    pub function: usize,
    pub why: String,
}

impl Precog {
    /// Every table the program gets, and the near misses worth reporting.
    pub(super) fn make_tables(&mut self, target_limit: Option<u64>, target: &str) -> (Vec<Table>, Vec<NotTable>) {
        let mut made = Vec::new();
        let mut not = Vec::new();
        if std::env::var("KAYO_TABLES").is_ok_and(|v| v == "0") {
            return (made, not);
        }
        let n = self.prog().functions.len();
        for f in 0..n {
            let func = &self.prog().functions[f];
            if func.dropped || func.nocheat || func.is_main || func.is_test || func.is_task || func.variadic || !func.fails.is_empty() || func.params.is_empty() {
                continue;
            }
            let Some(entry_bytes) = entry_size(&func.ret) else { continue };
            // Each parameter a plain whole number with a range.
            let mut ranges = Vec::new();
            for p in &func.params {
                let l = &func.locals[p.0 as usize];
                if !matches!(l.kind, LocalKind::Plain | LocalKind::Owned) || !matches!(l.ty.base(), Ty::Int(_) | Ty::Bounded { .. }) {
                    break;
                }
                match Domain::of_ty(&l.ty, &self.prog().items) {
                    Domain::Ints(s) => match s.bounds() {
                        Some(b) => ranges.push(b),
                        None => break,
                    },
                    _ => break,
                }
            }
            if ranges.len() != func.params.len() {
                continue;
            }
            let Some(closure) = pure_closure(self.prog(), f) else { continue };
            let decl_limit = func.inst.and_then(|i| self.prog().instances.get(i.0 as usize)).and_then(|i| self.prog().items.fns.get(i.key.fn_id.0 as usize)).and_then(|d| d.table_bytes);
            let (limit, limit_from) = match (decl_limit, target_limit) {
                (Some(b), _) => (b, "its own `table_bytes`".to_string()),
                (None, Some(b)) => (b, format!("`{}`'s `table_bytes`", target)),
                (None, None) => (DEFAULT_TABLE_BYTES, "the default, 65,536".to_string()),
            };
            // Counted without wrapping: an `int128`'s range has more values
            // than an `i128` holds, and wrapped it counted none.
            let count = ranges.iter().try_fold(1u128, |acc, (lo, hi)| hi.checked_sub(*lo).and_then(|d| d.checked_add(1)).filter(|n| *n > 0).and_then(|n| acc.checked_mul(n as u128)));
            let Some(count) = count.filter(|c| *c >= 1 && *c <= u64::MAX as u128).map(|c| c as u64) else { continue };
            let bytes = count.saturating_mul(entry_bytes);
            if limit == 0 {
                not.push(NotTable { function: f, why: format!("`table_bytes 0`: never a table ({} entries would have been {} bytes)", count, bytes) });
                continue;
            }
            if bytes > limit || count > MAX_ENTRIES {
                // Only a near miss is worth a line: a function on whole
                // `int64`s was never going to be one.
                if count <= MAX_ENTRIES * 16 {
                    not.push(NotTable { function: f, why: format!("{} entries would be {} bytes, over {} ({} bytes)", count, bytes, limit_from, limit) });
                }
                continue;
            }
            match self.answers(f, &ranges) {
                Ok(entries) => made.push(Table { function: f, ranges, entries, entry_bytes, bytes, limit, limit_from, closure }),
                Err(why) => not.push(NotTable { function: f, why }),
            }
        }
        // What the dry runs cached is not the program's.
        for k in std::mem::take(&mut self.dry_cache) {
            self.cache.remove(&k);
        }
        (made, not)
    }

    /// Every answer, in order, each worked out by running the function on
    /// that tuple; or why not.
    fn answers(&mut self, f: usize, ranges: &[(i128, i128)]) -> Result<Vec<Const>, String> {
        let params = self.prog().functions[f].params.clone();
        let was = (self.dry, self.gave_up);
        self.dry = true;
        let mut out = Vec::new();
        let total: u64 = ranges.iter().map(|(lo, hi)| (hi - lo + 1) as u64).product();
        let mut result = Ok(());
        for idx in 0..total {
            // The tuple at this index: the last parameter fastest.
            let mut tuple = vec![0i128; ranges.len()];
            let mut rest = idx;
            for k in (0..ranges.len()).rev() {
                let size = (ranges[k].1 - ranges[k].0 + 1) as u64;
                tuple[k] = ranges[k].0 + (rest % size) as i128;
                rest /= size;
            }
            let mut env = self.entry_env(f);
            for (p, v) in params.iter().zip(&tuple) {
                env.insert(*p, Domain::Ints(ISet::one(*v)));
            }
            self.gave_up = false;
            let d = self.run_function(f, env, 0);
            match (known_const(&d), self.gave_up) {
                (Some(c), false) => out.push(c),
                _ => {
                    result = Err(format!("its answer for {} is not one exact value Precog can work out", tuple.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")));
                    break;
                }
            }
        }
        let result = result.map(|_| out);
        (self.dry, self.gave_up) = was;
        result
    }
}

/// The bytes one answer takes in a table: a whole number at its width, a
/// `bool` a byte.
fn entry_size(ret: &Ty) -> Option<u64> {
    match ret.base() {
        Ty::Bool => Some(1),
        Ty::Int(crate::types::IntW::Fixed { bits, .. }) if *bits <= 64 => Some((*bits as u64).div_ceil(8).next_power_of_two()),
        // Wider answers are not tabled.
        Ty::Int(crate::types::IntW::Fixed { .. }) => None,
        Ty::Int(_) => Some(8),
        Ty::Bounded { inner, .. } => entry_size(inner),
        _ => None,
    }
}

/// `f` and every function it calls, all the way down, when none of them
/// does anything but compute on plain values: no C, no held function, no
/// task, no assembly, no operation on memory or the outside, and no local
/// that owns memory. `None` when one does.
fn pure_closure(p: &Program, f: usize) -> Option<Vec<usize>> {
    let mut seen = vec![f];
    let mut work = vec![f];
    while let Some(g) = work.pop() {
        let func = &p.functions[g];
        if func.nocheat || func.dropped {
            return None;
        }
        for l in &func.locals {
            match l.ty.base() {
                Ty::Int(_) | Ty::Bool | Ty::Bounded { .. } | Ty::Nothing => {}
                _ => return None,
            }
        }
        for b in &func.blocks {
            for s in &b.stmts {
                match &s.kind {
                    StmtKind::Assign { rvalue, .. } => match rvalue {
                        Rvalue::Use(_) | Rvalue::Unary { .. } | Rvalue::Binary { .. } | Rvalue::Convert { .. } => {}
                        _ => return None,
                    },
                    StmtKind::Site(_) | StmtKind::Nop | StmtKind::Drop(_) | StmtKind::EndLend(_) => {}
                    StmtKind::Op { .. } | StmtKind::Task { .. } | StmtKind::Asm { .. } => return None,
                }
            }
            match &b.term {
                Terminator::Call { callee: Callee::Fn(i), .. } => {
                    let i = i.0 as usize;
                    if !seen.contains(&i) {
                        seen.push(i);
                        work.push(i);
                    }
                }
                Terminator::Call { .. } | Terminator::Whichever { .. } | Terminator::Join { .. } | Terminator::TaskOutcome { .. } | Terminator::Fail { .. } => return None,
                _ => {}
            }
        }
    }
    Some(seen)
}
