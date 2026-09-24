//! Precog — the whole program, run at compile time over sets of values.
//!
//! `design/precog.md`. Layer 1 executes what is known; layer 2 tracks what
//! is not as a domain. They are one pass here, because they are one idea:
//! a domain holding exactly one value *is* a known value, and following a
//! loop trip by trip *is* executing it. The difference is only how much is
//! left when the sets get too big to walk.
//!
//! At every check site the question is the same: is every value in the
//! result's set legal? Yes → **proven**, and the check is not emitted. No
//! value legal → a **certain failure**, and the program never builds,
//! because it would fail there every time it ran. Otherwise → **unproven**,
//! which layer 3 (the solver) would take, and which the tiers handle until
//! it exists.
//!
//! Every limit here is a step count, never a clock
//! (`reproducible-builds.md`): the same source gives the same answer on
//! every machine, on every day.

pub mod report;
pub mod smt;

use std::collections::HashMap;

use crate::ast::{BinOp, Tier, UnOp};
use crate::mir::domain::{Class, Cmp, Domain, ISet, Lengths};
use crate::mir::*;
use crate::types::Ty;

/// Provisional, from Kespar (`precog.md`, Open — to be measured):
/// how many times one loop is followed trip by trip before widening.
pub const TRIPS_PER_LOOP: u32 = 100_000;
/// Provisional: trips across the whole program.
pub const TRIPS_PER_PROGRAM: u64 = 2_000_000;
/// Provisional: how deep a call chain is followed per call site.
pub const CALL_DEPTH: u32 = 8;
/// Provisional: the most argument tuples a recursive function is proven
/// over by induction, one analysis of its body each.
pub const INDUCTION_CAP: usize = 4096;
/// Provisional: one step per instruction interpreted.
pub const STEP_BUDGET: u64 = 100_000_000;

/// What the analysis concluded about one site, and why, in the words the
/// report and any message will use.
#[derive(Debug, Clone)]
pub struct SiteResult {
    pub function: usize,
    pub site: SiteId,
    pub verdict: Verdict,
    /// "layer 1: ran it", "layer 2: 1 to 100" — what convinced it.
    pub note: String,
    /// The value the check asks about, as layers 1 and 2 saw it, if the
    /// kind has one; and every operand's.
    pub value: Option<Domain>,
    pub operands: Vec<Domain>,
}

pub struct Analysis {
    pub sites: Vec<SiteResult>,
    /// What the solver portfolio was on this machine, and what it said.
    pub portfolio: String,
    /// Sites layer 3 settled that layer 2 could not.
    pub solved: usize,
    /// The solvers disagreed here: a bug in a solver or in the query both
    /// of them received, never in the program.
    pub disagreements: Vec<String>,
    /// `nocheck` sites that Precog's ordinary work happened to show fail on
    /// every run (owner, 2026-09-22). A note, never a refusal: Precog does
    /// not go looking, it only refuses to stay silent about what it could
    /// not avoid noticing.
    pub certain_under_nocheck: Vec<(usize, SiteId, String)>,
    /// Sites that fail on every path that reaches them: the program never
    /// builds, whatever the tier.
    pub certain: Vec<usize>,
    pub steps: u64,
    /// The whole program ran at compile time: nothing came from outside.
    pub whole_program: bool,
    /// How many instructions layer 1 replaced by their answer.
    pub folded: usize,
    /// Sites reached on every run of the program (owner, 2026-09-23): only
    /// there is a certain failure a compile error.
    pub always_reached: std::collections::HashSet<(usize, u32)>,
}

/// One local's value, as far as this path knows.
type Env = HashMap<LocalId, Domain>;

pub fn analyse(p: &mut Program) -> Analysis {
    analyse_opts(p, true)
}

/// `fold: false` leaves every instruction in the binary, even one layer 1
/// worked out: a proof-checking build (`--assert-proofs`) has to compute
/// what Precog claimed, not be handed the claim.
pub fn analyse_opts(p: &mut Program, fold: bool) -> Analysis {
    let mut a = Precog { p: p as *mut Program, steps: 0, trips: 0, results: HashMap::new(), claims: HashMap::new(), lies: Vec::new(), folds: HashMap::new(), ranges: HashMap::new(), loop_entry: HashMap::new(), loop_bound: HashMap::new(), acc_step: HashMap::new(), induction: HashMap::new(), induction_missing: HashMap::new(), induction_wide: false, dry: false, dry_cache: Vec::new(), gave_up: false, cache: HashMap::new(), reads_outside: false, block_facts: HashMap::new(), edges: std::collections::HashSet::new() };
    // A program is walked from `MAIN`, and every function it reaches is
    // analysed **per call site**, with that call's argument domains
    // (`precog.md`): `square['i']` with `'i'` in 1..1000 proves its own
    // multiply, and nobody writes a contract on `square`. A Limb with no
    // `MAIN` is a library — there is no call site to start from, so each
    // function is taken on its own terms instead.
    match p.main {
        Some(m) => {
            let entry = a.entry_env(m.0 as usize);
            a.run_function(m.0 as usize, entry, 0);
        }
        None => {
            for f in 0..p.functions.len() {
                let entry = a.entry_env(f);
                a.run_function(f, entry, 0);
            }
        }
    }
    // Fold what layer 1 settled, before anything reads the graph again —
    // and only when it settled *everything*. A recursion deeper than the
    // limit means `'n' < 2` looked false on every trip the analysis took
    // and true on the one it never reached, and folding that would be a
    // program that never stops.
    // The ranges go to codegen on the same terms as the folds: only when
    // layer 1 settled everything, and never in a build checking the proofs.
    if !a.gave_up && fold {
        for ((f, b, si), r) in &a.ranges {
            if let (Some(r), Some(func)) = (r, p.functions.get_mut(*f)) {
                func.ranges.insert((*b as u32, *si as u32), *r);
            }
        }
    }
    let mut folded = 0usize;
    let folds: Vec<((usize, usize, usize), Option<Const>)> = if a.gave_up || !fold { Vec::new() } else { a.folds.iter().map(|(k, v)| (*k, v.clone())).collect() };
    for ((f, b, si), known) in &folds {
        let Some(c) = known else { continue };
        // The check sites guarding an instruction are the ones just before
        // it. Folding one that is not proven would take its check with it:
        // layer 1's answer is the mathematician's, worked out in 128 bits,
        // and the instruction may not be able to give it. `(-28014) +
        // (-23953)` at `int16` fails every time, and the ordinary build
        // printed the folded answer while `--assert-proofs` stopped (check
        // claude, seed 25).
        let guarded_by_open_site = p.functions.get(*f).and_then(|func| func.blocks.get(*b)).map_or(false, |blk| {
            blk.stmts[..*si].iter().rev().map_while(|s| match &s.kind {
                StmtKind::Site(id) => Some(*id),
                _ => None,
            }).any(|id| !matches!(a.results.get(&(*f, id.0)), Some(Verdict::Proven { .. })))
        });
        if guarded_by_open_site {
            continue;
        }
        if let Some(st) = p.functions.get_mut(*f).and_then(|f| f.blocks.get_mut(*b)).and_then(|blk| blk.stmts.get_mut(*si)) {
            if let StmtKind::Assign { place, rvalue } = &st.kind {
                if !matches!(rvalue, Rvalue::Use(Operand::Const(_))) {
                    st.kind = StmtKind::Assign { place: place.clone(), rvalue: Rvalue::Use(Operand::Const(c.clone())) };
                    folded += 1;
                }
            }
        }
    }
    let mut sites: Vec<SiteResult> = a
        .results
        .iter()
        .map(|((f, s), v)| {
            let (value, operands) = a.claims.get(&(*f, *s)).cloned().unwrap_or((None, Vec::new()));
            SiteResult { function: *f, site: SiteId(*s), verdict: v.clone(), note: note_of(v), value, operands }
        })
        .collect();
    sites.sort_by_key(|r| (r.function, r.site.0));
    let certain: Vec<usize> = sites.iter().enumerate().filter(|(_, r)| matches!(&r.verdict, Verdict::Fails { counterexample } if counterexample.starts_with("every"))).map(|(i, _)| i).collect();
    let mut certain_under_nocheck = a.lies;
    certain_under_nocheck.sort_by_key(|(f, s, _)| (*f, s.0));
    certain_under_nocheck.dedup_by_key(|(f, s, _)| (*f, s.0));
    // Layer 3: what is left after layer 2 is asked exactly.
    // Layer 2's joined block states, as facts layer 3 may lean on — only if
    // layer 2 followed every call and every trip it needed to. If it gave
    // up anywhere, a state it never saw could be a real one.
    let facts: Facts = if a.gave_up { HashMap::new() } else { a.block_facts.clone() };
    let portfolio = smt::Portfolio::find();
    let mut solved = 0usize;
    let mut disagreements = Vec::new();
    if !portfolio.members.is_empty() {
        for r in sites.iter_mut() {
            let ask = match &r.verdict {
                Verdict::Proven { .. } | Verdict::Trusted => false,
                Verdict::Fails { counterexample } => !counterexample.starts_with("every"),
                Verdict::Unproven => true,
            };
            if !ask {
                continue;
            }
            let Some(q) = smt::encode(p, r.function, r.site, &facts) else {
                // A site whose values bit-vectors do not describe — what a
                // task computes, a horizontal sum across lanes. Layer 3
                // ran; it had nothing it could ask.
                r.note = "layer 3 has no way to ask about this one yet".into();
                continue;
            };
            let (answer, each) = portfolio.ask(&q);
            if smt::disagreed(&each) {
                // Run the counterexample and see which side was right
                // (`precog.md`, owner: c). Until the Precog oracle exists,
                // the safe answer is taken and the disagreement reported.
                disagreements.push(describe_disagreement(p, r, &each, &q.text));
                r.verdict = Verdict::Unproven;
                r.note = "the solvers disagreed; the safe answer was taken".into();
                continue;
            }
            match answer {
                smt::Answer::Unsat if !q.truncated => {
                    r.verdict = Verdict::Proven { how: "layer 3" };
                    r.note = "layer 3: no input reaches it with a bad value".into();
                    solved += 1;
                }
                smt::Answer::Unsat => {
                    // Proven for the trips the unrolling reached, which is
                    // not the same as proven.
                    r.note = "layer 3 proved the first trips of a loop, which is not the whole loop".into();
                }
                smt::Answer::Sat(model) => {
                    let when = if model.is_empty() { "the solver found a way".to_string() } else { model };
                    r.verdict = Verdict::Fails { counterexample: when.clone() };
                    r.note = format!("layer 3: {}", when);
                    solved += 1;
                }
                smt::Answer::Unknown => {
                    r.note = "layer 3 asked, and the solvers reached their step budget without an answer".into();
                }
            }
        }
    }
    for r in &sites {
        let f = &mut p.functions[r.function];
        f.sites[r.site.0 as usize].verdict = r.verdict.clone();
    }
    let always_reached = if a.gave_up { std::collections::HashSet::new() } else { reached_every_run(p, &a.edges) };
    Analysis { sites, certain, certain_under_nocheck, steps: a.steps, whole_program: !a.reads_outside, folded, portfolio: portfolio.describe(), solved, disagreements, always_reached }
}

struct Precog {
    /// The graph. Held raw so a call can read another function while this
    /// one is being walked; nothing is written through it.
    p: *mut Program,
    steps: u64,
    trips: u64,
    /// One verdict per site, whatever it took to get there: a site reached
    /// from several call sites has to survive all of them, so the worse
    /// answer is the one kept.
    results: HashMap<(usize, u32), Verdict>,
    /// What each site's check is about — the value it asks of — as layers 1
    /// and 2 saw it, joined over every time it was judged; and the
    /// operands'. For the Precog oracle (`cca precog --json`).
    claims: HashMap<(usize, u32), (Option<Domain>, Vec<Domain>)>,
    /// Exact instructions' value ranges, `None` once one visit could not say.
    ranges: HashMap<(usize, usize, usize), Option<(i128, i128)>>,
    /// Counted loops: the state each head was entered with from outside
    /// (joined), the largest value its bound took, and the range of what
    /// each accumulator adds per trip (joined over every visit).
    loop_entry: HashMap<(usize, usize), Env>,
    loop_bound: HashMap<(usize, usize), i128>,
    acc_step: HashMap<(usize, usize, usize), ISet>,
    /// Recursive functions summed up by induction on their arguments: for
    /// each function, the return found for each argument tuple, and whether
    /// its table is being filled right now (so a recursive call reads it).
    induction: HashMap<usize, (HashMap<Vec<i128>, Domain>, bool)>,
    /// Tuples a recursive call reached that were not in its table, and
    /// whether one reached more than can be listed.
    induction_missing: HashMap<usize, std::collections::BTreeSet<Vec<i128>>>,
    induction_wide: bool,
    /// A dry run finds which tuples a recursion reaches, and records
    /// nothing: no verdict, fold, range, fact, edge or cached answer.
    dry: bool,
    dry_cache: Vec<(usize, String)>,
    lies: Vec<(usize, SiteId, String)>,
    /// What layer 1 worked out for an instruction, where it worked out the
    /// same answer every time it went past. `None` poisons it: the
    /// instruction was reached with different values, so it stays.
    folds: HashMap<(usize, usize, usize), Option<Const>>,
    /// The analysis stopped short somewhere — a call deeper than the limit,
    /// a budget spent. Nothing may be folded after that: a value that looked
    /// settled might only have looked that way because the contexts that
    /// would have disagreed were never walked.
    gave_up: bool,
    /// What a function returns, for one set of argument domains.
    cache: HashMap<(usize, String), Domain>,
    reads_outside: bool,
    /// Everything layer 2 ever saw arrive at each block, joined: every trip
    /// of every loop, every call site. A sound invariant for the block —
    /// provided nothing was skipped, which `gave_up` says.
    block_facts: HashMap<(usize, usize), Env>,
    /// Every edge layer 2 found some run could take: (function, from, to).
    edges: std::collections::HashSet<(usize, usize, usize)>,
}

/// What layer 3 may assume at the head of a loop, per function and block.
pub type Facts = HashMap<(usize, usize), HashMap<LocalId, Domain>>;

impl Precog {
    fn prog(&self) -> &Program {
        unsafe { &*self.p }
    }

    fn entry_env(&mut self, f: usize) -> Env {
        let items = &self.prog().items;
        let func = &self.prog().functions[f];
        let mut env = Env::new();
        for (i, l) in func.locals.iter().enumerate() {
            // A parameter is whatever its type allows until a call says more.
            env.insert(LocalId(i as u32), Domain::of_ty(&l.ty, &items));
        }
        env
    }

    /// Walk one function's graph to a fixpoint, recording every site.
    /// Cache an answer; in a dry run, remember to forget it afterwards.
    fn remember(&mut self, i: usize, key: String, d: Domain) {
        if self.dry {
            self.dry_cache.push((i, key.clone()));
        }
        self.cache.insert((i, key), d);
    }

    /// Every tuple of values the parameters can take, when they are whole
    /// numbers with few enough of them, smallest first. For the table, each
    /// parameter runs from its type's lowest value up to the highest this
    /// call can pass (`from_type_floor`): a call with 10 also needs 9, 8, …
    /// on the way down, and those are what induction starts from.
    fn arg_tuples(&self, params: &[LocalId], env: &Env) -> Option<Vec<Vec<i128>>> {
        self.arg_tuples_opts(params, env, None)
    }

    fn arg_tuples_opts(&self, params: &[LocalId], env: &Env, from_type_floor: Option<usize>) -> Option<Vec<Vec<i128>>> {
        let mut out: Vec<Vec<i128>> = vec![Vec::new()];
        for p in params {
            let Some(Domain::Ints(s)) = env.get(p) else { return None };
            let vals = match (from_type_floor, s.bounds(), int_limits(&self.local_ty(from_type_floor.unwrap_or(0), *p), &self.prog().items)) {
                (Some(_), Some((_, hi)), Some((lo, _))) if hi >= lo && hi - lo < INDUCTION_CAP as i128 => (lo..=hi).collect(),
                _ => s.values()?,
            };
            if out.len().saturating_mul(vals.len()) > INDUCTION_CAP {
                return None;
            }
            out = out.into_iter().flat_map(|t| vals.iter().map(move |v| {
                let mut t = t.clone();
                t.push(*v);
                t
            })).collect();
        }
        out.sort();
        Some(out)
    }

    /// While `i`'s table is being filled, a call to it is the union of the
    /// table's answers for every tuple its arguments can be. A tuple not in
    /// the table yet — the call is not to a smaller argument — is anything
    /// the return type holds: sound, only less precise.
    fn filling(&mut self, i: usize, params: &[LocalId], env: &Env) -> Option<Domain> {
        let active = matches!(self.induction.get(&i), Some((_, true)));
        if !active {
            return None;
        }
        let ret_ty = self.prog().functions[i].ret.clone();
        let whole = Domain::of_ty(&ret_ty, &self.prog().items);
        let Some(tuples) = self.arg_tuples(params, env) else {
            self.induction_wide = true;
            return Some(whole);
        };
        let mut out = Domain::Empty;
        let mut missing = Vec::new();
        for t in &tuples {
            match self.induction[&i].0.get(t) {
                Some(d) => out = out.union(d),
                None => missing.push(t.clone()),
            }
        }
        if !missing.is_empty() {
            self.induction_missing.entry(i).or_default().extend(missing);
            return Some(whole);
        }
        Some(out)
    }

    /// Induction on the arguments (owner, 2026-09-23, "teach Precog to prove
    /// bounds on recursive functions by induction"). For a function that
    /// calls itself, with whole-number parameters of few values, the body is
    /// analysed once per argument tuple, smallest first; a recursive call
    /// inside is answered from the answers for the tuples before it. Every
    /// run of the body has one of these tuples, and its calls were to tuples
    /// already done (or were given the return type's whole range), so every
    /// site's verdict, merged over the tuples, holds for every call. `fib`
    /// on `int64[0, 92]` proves `fib['n' - 1] + fib['n' - 2]` fits.
    fn by_induction(&mut self, i: usize, params: &[LocalId], env: &Env, depth: u32) -> Option<Domain> {
        let func = &self.prog().functions[i];
        let calls_itself = func.blocks.iter().any(|b| matches!(&b.term, Terminator::Call { callee: Callee::Fn(idx), .. } if idx.0 as usize == i));
        if !calls_itself || params.is_empty() || matches!(self.induction.get(&i), Some((_, true))) {
            return None;
        }
        // Every tuple the recursion can reach must have its body analysed —
        // a body analysed only for 10 folded `'n' < 2` to false and never
        // stopped. So first a dry run finds them: analyse, collect the
        // tuples recursive calls reached that are not in the table, add
        // them, until none are new. Then the real run, smallest first, so a
        // call on a smaller tuple reads its answer. Too many: the old way.
        let asked = self.arg_tuples(params, env)?;
        let mut needed: std::collections::BTreeSet<Vec<i128>> = asked.iter().cloned().collect();
        if let Some(from_floor) = self.arg_tuples_opts(params, env, Some(i)) {
            needed.extend(from_floor);
        }
        let run_all = |this: &mut Self, needed: &std::collections::BTreeSet<Vec<i128>>| {
            for t in needed {
                if this.induction[&i].0.contains_key(t) {
                    continue;
                }
                let mut e = env.clone();
                for (p, v) in params.iter().zip(t) {
                    e.insert(*p, Domain::Ints(ISet::one(*v)));
                }
                let d = this.run_function(i, e, depth + 1);
                this.induction.get_mut(&i).unwrap().0.insert(t.clone(), d);
            }
        };
        let was_dry = self.dry;
        self.induction.insert(i, (HashMap::new(), true));
        self.dry = true;
        let mut complete = false;
        while needed.len() <= INDUCTION_CAP {
            self.induction_missing.remove(&i);
            self.induction_wide = false;
            run_all(self, &needed);
            if self.induction_wide {
                break;
            }
            let new: Vec<Vec<i128>> = self.induction_missing.get(&i).map(|m| m.iter().filter(|t| !needed.contains(*t)).cloned().collect()).unwrap_or_default();
            if new.is_empty() {
                complete = true;
                break;
            }
            needed.extend(new);
        }
        self.dry = was_dry;
        if !was_dry {
            // What the dry run cached was worked out without its verdicts
            // recorded: the real run works it out again.
            for k in std::mem::take(&mut self.dry_cache) {
                self.cache.remove(&k);
            }
        }
        if !complete {
            self.induction.remove(&i);
            return None;
        }
        self.induction.insert(i, (HashMap::new(), true));
        run_all(self, &needed);
        self.induction.get_mut(&i).unwrap().1 = false;
        let table = &self.induction[&i].0;
        let mut out = Domain::Empty;
        for t in &asked {
            if let Some(d) = table.get(t) {
                out = out.union(d);
            }
        }
        if matches!(out, Domain::Empty) {
            let ret_ty = self.prog().functions[i].ret.clone();
            out = Domain::of_ty(&ret_ty, &self.prog().items);
        }
        Some(out)
    }

    /// How many times a counted loop's body can run: from the counter's
    /// lowest value on entry to the bound's highest, a step at a time.
    fn trips_of(&self, f: usize, head: usize, c: &Counted) -> Option<i128> {
        let entry = self.loop_entry.get(&(f, head))?;
        let Some(Domain::Ints(c0)) = entry.get(&c.counter) else { return None };
        let (lo, _) = c0.bounds()?;
        let hi = *self.loop_bound.get(&(f, head))?;
        let last = if c.strict { hi.saturating_sub(1) } else { hi };
        if last < lo {
            return Some(0);
        }
        Some((last.saturating_sub(lo)) / c.step + 1)
    }

    fn run_function(&mut self, f: usize, entry: Env, depth: u32) -> Domain {
        if depth > CALL_DEPTH {
            self.gave_up = true;
            let ret = self.prog().functions[f].ret.clone();
            return Domain::of_ty(&ret, &self.prog().items);
        }
        let n_blocks = self.prog().functions[f].blocks.len();
        let back = back_edges(&self.prog().functions[f]);
        // What each back edge's loop is made of: a fresh trip starts the
        // whole body again, not only its head, or the body's blocks would
        // pile up every trip's values and the loop would be joined after all.
        let loops: HashMap<(usize, usize), Vec<usize>> = back.iter().map(|(b, h)| ((*b, *h), natural_loop(&self.prog().functions[f], *h, *b))).collect();
        let mut states: Vec<Option<Env>> = vec![None; n_blocks];
        let mut visits: Vec<u32> = vec![0; n_blocks];
        states[0] = Some(entry);
        let mut work = vec![0usize];
        let mut returned = Domain::Empty;
        let mut sites_seen: HashMap<u32, Verdict> = HashMap::new();
        // A trip ends when *every* path through it has come round, not when
        // the first one does. A back edge therefore does not start the next
        // trip itself: what it brings is joined into the loop's pending
        // trip, and the trip starts once no block of that loop is waiting
        // any more. Starting it at the first arrival wiped the states of the
        // paths still on their way — `if 'b' { put_i[0]; }` inside a loop
        // was never analysed when the `else` side got round first, and a
        // value layer 1 saw only on the other call was folded into both.
        let mut pending_trip: HashMap<usize, Env> = HashMap::new();
        let steps = thresholds(&self.prog().functions[f]);
        let heads: std::collections::HashSet<usize> = back.iter().map(|(_, h)| *h).collect();
        let mut head_body: HashMap<usize, Vec<usize>> = HashMap::new();
        for ((_, h), body) in &loops {
            head_body.entry(*h).or_default().extend(body.iter().copied());
        }
        // Counted loops, and where each accumulator's step is computed.
        let mut counted: HashMap<usize, Counted> = HashMap::new();
        for (h, body) in &head_body {
            let latches: Vec<usize> = back.iter().filter(|(_, hh)| hh == h).map(|(fr, _)| *fr).collect();
            // Every block of a loop nested in this one.
            let inner: std::collections::HashSet<usize> = head_body.iter().filter(|(hh, _)| *hh != h && body.contains(hh)).flat_map(|(_, bd)| bd.iter().copied()).collect();
            if let [latch] = latches.as_slice() {
                if let Some(c) = counted_loop(&self.prog().functions[f], *h, *latch, body, &inner) {
                    counted.insert(*h, c);
                }
            }
        }
        // Program order (reverse postorder): the worklist always takes the
        // earliest block, so a loop finishes all its trips before what
        // follows it is walked, once, with the loop's final way out. Taking
        // the newest instead walked everything after a loop again after
        // every trip, until the program's budget ran out.
        let order = {
            let func = &self.prog().functions[f];
            let n = func.blocks.len();
            let mut seen = vec![false; n];
            let mut post: Vec<usize> = Vec::with_capacity(n);
            let mut stack: Vec<(usize, usize)> = vec![(0, 0)];
            seen[0] = true;
            // A successor that stays inside more of the loops around `b` is
            // explored later, so it comes first: an inner loop's body before
            // its way out, which is still inside the outer loop.
            let stays = |b: usize, t: usize| head_body.values().filter(|body| body.contains(&b) && body.contains(&t)).count();
            while let Some((b, k)) = stack.pop() {
                let mut succ = crate::mir::verify::successors(&func.blocks[b].term);
                succ.sort_by_key(|t| stays(b, t.0 as usize));
                if k < succ.len() {
                    stack.push((b, k + 1));
                    let t = succ[k].0 as usize;
                    if t < n && !seen[t] {
                        seen[t] = true;
                        stack.push((t, 0));
                    }
                } else {
                    post.push(b);
                }
            }
            let mut rank = vec![usize::MAX; n];
            for (i, b) in post.iter().rev().enumerate() {
                rank[*b] = i;
            }
            rank
        };
        let acc_sites: std::collections::HashSet<(usize, usize)> = counted.values().flat_map(|c| c.accs.iter().map(|(_, at)| *at)).collect();

        loop {
            if let Some(h) = pending_trip.keys().copied().find(|h| !head_body.get(h).map(|bd| bd.iter().any(|x| x != h && work.contains(x))).unwrap_or(false)) {
                let e = pending_trip.remove(&h).unwrap();
                self.trips += 1;
                let settled = matches!(&states[h], Some(old) if same(old, &e));
                if !settled {
                    if let Some(body) = head_body.get(&h) {
                        for x in body {
                            if *x != h {
                                states[*x] = None;
                            }
                        }
                    }
                    states[h] = Some(e);
                    if !work.contains(&h) {
                        work.push(h);
                    }
                }
            }
            let next = work.iter().enumerate().min_by_key(|(_, b)| order.get(**b).copied().unwrap_or(usize::MAX)).map(|(i, _)| i);
            let Some(b) = next.map(|i| work.swap_remove(i)) else {
                if pending_trip.is_empty() {
                    break;
                }
                continue;
            };
            self.steps += 1;
            if self.steps > STEP_BUDGET {
                self.gave_up = true;
                break;
            }
            let Some(mut env) = states[b].clone() else { continue };
            visits[b] += 1;
            // Remember what arrived here, joined with everything that ever
            // did. A local missing from either side is unknown in the join.
            match self.block_facts.get_mut(&(f, b)) {
                _ if self.dry => {}
                None => {
                    self.block_facts.insert((f, b), env.clone());
                }
                Some(seen) => {
                    seen.retain(|l, d| match env.get(l) {
                        Some(now) => {
                            *d = d.union(now);
                            true
                        }
                        None => false,
                    });
                }
            }
            let widening = visits[b] as u64 > 32 || self.trips > TRIPS_PER_PROGRAM;
            // One block's statements.
            let stmts = self.prog().functions[f].blocks[b].stmts.clone();
            let mut facts: HashMap<LocalId, (Cmp, Operand, Operand)> = HashMap::new();
            // Where a number came from, when it came from asking a value how
            // long it is. A guardrail is written about the *length*
            // (`if list.length[…] == 0 { … }`), so narrowing the number has
            // to reach back to the list, or the next `list.length` of the
            // same list would know nothing.
            let mut lengths: HashMap<LocalId, (Op, Place)> = HashMap::new();
            for (si, s) in stmts.iter().enumerate() {
                self.steps += 1;
                match &s.kind {
                    StmtKind::Assign { place, rvalue } => {
                        let d = self.eval_rvalue(f, rvalue, &env, &mut facts, place);
                        if acc_sites.contains(&(b, si)) {
                            if let Rvalue::Binary { lhs, rhs, .. } = rvalue {
                                // The step is the operand that is not the
                                // accumulator being written two lines on.
                                let x = counted.values().flat_map(|c| c.accs.iter()).find(|(_, at)| *at == (b, si)).map(|(x, _)| *x);
                                let e = if lhs.place().map(|p| Some(p.local) == x).unwrap_or(false) { rhs } else { lhs };
                                let step = match self.read_operand(f, &env, e) {
                                    Domain::Ints(s) => s,
                                    _ => ISet::Range { lo: i128::MIN / 4, hi: i128::MAX / 4 },
                                };
                                let joined = match self.acc_step.get(&(f, b, si)) {
                                    Some(old) => old.union(&step),
                                    None => step,
                                };
                                self.acc_step.insert((f, b, si), joined);
                            }
                        }
                        // Every value an exact instruction's operands and
                        // answer take here, joined over every visit: what
                        // lets codegen work at 64 bits.
                        if let Rvalue::Binary { lhs, rhs, .. } = rvalue {
                            if self.place_ty(f, place).is_exact() {
                                let key = (f, b, si);
                                let hull = [self.read_operand(f, &env, lhs), self.read_operand(f, &env, rhs), d.clone()].iter().try_fold((i128::MAX, i128::MIN), |(lo, hi), x| match x {
                                    Domain::Ints(s) => s.bounds().map(|(a, z)| (lo.min(a), hi.max(z))),
                                    _ => None,
                                });
                                let merged = match (self.ranges.get(&key), hull) {
                                    (Some(None), _) | (_, None) => None,
                                    (Some(Some((a, z))), Some((c, y))) => Some((*a.min(&c), *z.max(&y))),
                                    (None, h) => h,
                                };
                                if !self.dry {
                                    self.ranges.insert(key, merged);
                                }
                            }
                        }
                        // Layer 1: "everything that does not depend on the
                        // outside world is known… the result is folded in".
                        // An instruction that comes out the same every time
                        // it is reached becomes that answer.
                        if foldable(rvalue) && !self.dry {
                            let known = known_const(&d);
                            let key = (f, b, si);
                            match self.folds.get(&key) {
                                None => {
                                    self.folds.insert(key, known);
                                }
                                Some(prev) => {
                                    if *prev != known {
                                        self.folds.insert(key, None);
                                    }
                                }
                            }
                        }
                        self.store(f, &mut env, place, d);
                        lengths.retain(|k, (_, p)| *k != place.local && p.local != place.local);
                    }
                    StmtKind::Op { dest, op, args } => {
                        let d = self.eval_op(f, op, args, &env);
                        if let Some(p) = dest {
                            self.store(f, &mut env, p, d);
                            if p.proj.is_empty() {
                                match (op, args.first().and_then(|a| a.place())) {
                                    (Op::ListLength | Op::ArrayLength | Op::MapLength | Op::StrBytes | Op::StrPoints | Op::StrGraphemes, Some(src)) => {
                                        lengths.insert(p.local, (op.clone(), src.clone()));
                                    }
                                    _ => {
                                        lengths.remove(&p.local);
                                    }
                                }
                            }
                        }
                        // Anything that changes a value makes what was asked
                        // about it a moment ago no longer news.
                        if let Some(src) = args.first().and_then(|a| a.place()) {
                            if matches!(op, Op::ListAppend | Op::ListRemove | Op::MapSet | Op::MapRemove | Op::SimdStore) {
                                lengths.retain(|_, (_, p)| p.local != src.local);
                            }
                        }
                        // An operation that changes a collection forgets what
                        // was known about its length unless the op says.
                        self.apply_op_effect(f, &mut env, op, args);
                    }
                    StmtKind::Site(id) => {
                        let v = self.judge(f, *id, &env);
                        let prev = sites_seen.get(&id.0).cloned();
                        let combined = match prev {
                            None => v,
                            Some(old) => worse(old, v),
                        };
                        sites_seen.insert(id.0, combined);
                    }
                    StmtKind::Drop(_) | StmtKind::EndLend(_) | StmtKind::Nop => {}
                    StmtKind::Task { body, captures, .. } => {
                        // A task runs with what it was given; its own sites are
                        // recorded against its own function.
                        let mut tenv = self.entry_env(body.0 as usize);
                        let tf = &self.prog().functions[body.0 as usize];
                        let params: Vec<LocalId> = tf.captures.clone();
                        for (pl, (_, place)) in params.iter().zip(captures) {
                            let d = self.read(f, &env, place);
                            tenv.insert(*pl, d);
                        }
                        let idx = body.0 as usize;
                        if depth < CALL_DEPTH {
                            self.run_function(idx, tenv, depth + 1);
                        }
                        // A task lent something mutably may change it at any
                        // moment until its block ends.
                        let lent: Vec<Place> = captures.iter().filter(|(v, _)| *v == crate::ast::Verb::LendMut).map(|(_, p)| p.clone()).collect();
                        self.forget_mutable_lends(f, &mut env, lent.into_iter());
                    }
                    StmtKind::Asm { outs, .. } => {
                        // Nothing about assembly can be known.
                        for (pl, _) in outs {
                            let ty = self.place_ty(f, pl);
                            self.store(f, &mut env, pl, Domain::of_ty(&ty, &self.prog().items));
                        }
                    }
                }
            }
            // Where the block goes, and what each successor then knows.
            let term = self.prog().functions[f].blocks[b].term.clone();
            let mut go = |this: &mut Self, to: BlockId, e: Env, states: &mut Vec<Option<Env>>, work: &mut Vec<usize>| {
                let i = to.0 as usize;
                // This edge can be taken: what "reached on every run" is
                // worked out from.
                if !this.dry {
                    this.edges.insert((f, b, i));
                }
                // A **back edge** is the loop going round again. The design
                // follows a counted loop *trip by trip* rather than joining
                // at its head (`precog.md`, layer 2), because a join cannot
                // see that a counter and a sum move together: joining gives
                // `count` every value from 0 upwards, and the sum overflows
                // in the analysis though never in the program. So each trip
                // replaces the head's state, and every trip's verdict is
                // merged — the site has to hold on all of them.
                let looping = back.contains(&(b, i));
                // Entered from outside: the state a counted loop's bound
                // starts from.
                if heads.contains(&i) && !head_body.get(&i).map_or(false, |bd| bd.contains(&b)) {
                    let joined = match this.loop_entry.get(&(f, i)) {
                        Some(old) => join(old, &e, false, unsafe { &(*this.p).items }, |l| this.local_ty(f, l)),
                        None => e.clone(),
                    };
                    this.loop_entry.insert((f, i), joined);
                }
                // A loop is followed up to 100 000 trips before its head
                // widens (owner, 2026-09-23: "keep it at 100k"); a counted
                // loop's bounds apply when it does.
                let trips_bound = counted.get(&i).and_then(|c| this.trips_of(f, i, c));
                let over_budget = visits[i] as u32 >= TRIPS_PER_LOOP || this.trips > TRIPS_PER_PROGRAM;
                let merged = match &states[i] {
                    None => e,
                    Some(old) => {
                        if looping && !over_budget {
                            // Wait for the rest of this trip (above). If the
                            // trip changes nothing, the loop has settled and
                            // going round again would say the same thing.
                            let joined = match pending_trip.remove(&i) {
                                None => e,
                                Some(p) => join(&p, &e, false, unsafe { &(*this.p).items }, |l| this.local_ty(f, l)),
                            };
                            pending_trip.insert(i, joined);
                            return;
                        } else {
                            // Widening happens at a loop's head and nowhere
                            // else: every cycle passes one, so it still ends,
                            // and a block inside the loop keeps what its
                            // condition just established. Widening at any
                            // busy block threw `'i' <== 'n'` away in the
                            // body — `'i'` went back to `int64`'s top — and
                            // lost an interval across nested loops (check
                            // claude, seed 316).
                            let at_head = heads.contains(&i);
                            // Only on the way round: arrivals from outside
                            // are bounded by whatever produces them (a loop
                            // before this one, walked trip by trip, widens
                            // itself), and widening on them threw away a
                            // list's counted length before the next loop saw it.
                            let widen_now = (widening && at_head && looping) || (looping && over_budget);
                            let mut m = join_to(old, &e, widen_now, unsafe { &(*this.p).items }, |l| this.local_ty(f, l), &steps);
                            // A counted loop's accumulators: at most the
                            // entry plus trips times the largest step.
                            if widen_now {
                                if let (Some(c), Some(t)) = (counted.get(&i), trips_bound) {
                                    for (x, at) in &c.accs {
                                        let (Some(Domain::Ints(xe)), Some(step)) = (this.loop_entry.get(&(f, i)).and_then(|en| en.get(x)).cloned(), this.acc_step.get(&(f, at.0, at.1)).cloned()) else { continue };
                                        let (Some((e1, e2)), Some((s1, s2))) = (xe.bounds(), step.bounds()) else { continue };
                                        let lo = e1.saturating_add(t.saturating_mul(s1.min(0)));
                                        let hi = e2.saturating_add(t.saturating_mul(s2.max(0)));
                                        if let Some(Domain::Ints(_)) = m.get(x) {
                                            m.insert(*x, Domain::Ints(ISet::range(lo, hi)));
                                        }
                                    }
                                    for l in &c.grows {
                                        let Some(Domain::Seq { len: len0, .. }) = this.loop_entry.get(&(f, i)).and_then(|en| en.get(l)).cloned() else { continue };
                                        let Some((a, z)) = len0.bounds() else { continue };
                                        if let Some(Domain::Seq { elem, .. }) = m.get(l).cloned() {
                                            m.insert(*l, Domain::Seq { len: ISet::range(a, z.saturating_add(t)), elem });
                                        }
                                    }
                                }
                            }
                            if same(old, &m) {
                                return;
                            }
                            m
                        }
                    }
                };
                states[i] = Some(merged);
                if !work.contains(&i) {
                    work.push(i);
                }
            };
            match &term {
                Terminator::Goto(t) => go(self, *t, env, &mut states, &mut work),
                Terminator::If { cond, then, else_ } => {
                    self.trips += 1;
                    if let Some(c) = counted.get(&b) {
                        let hi = match self.read_operand(f, &env, &c.bound) {
                            Domain::Ints(s) => s.bounds().map(|(_, h)| h).unwrap_or(i128::MAX / 4),
                            _ => i128::MAX / 4,
                        };
                        let prev = self.loop_bound.get(&(f, b)).copied().unwrap_or(i128::MIN);
                        self.loop_bound.insert((f, b), prev.max(hi));
                    }
                    let (can_true, can_false) = self.bool_of(f, cond, &env);
                    if can_true {
                        let e = self.narrow(f, &env, cond, &facts, &lengths, true);
                        go(self, *then, e, &mut states, &mut work);
                    }
                    if can_false {
                        let e = self.narrow(f, &env, cond, &facts, &lengths, false);
                        go(self, *else_, e, &mut states, &mut work);
                    }
                }
                Terminator::Switch { value, cases } => {
                    let d = self.read_operand(f, &env, value);
                    for (case, to) in cases {
                        if let Domain::Enum(possible) = &d {
                            if !possible.iter().any(|(c, _)| c == case) {
                                continue;
                            }
                        }
                        go(self, *to, env.clone(), &mut states, &mut work);
                    }
                }
                Terminator::Call { callee, args, dest, ok, fail } => {
                    let ret = self.call(f, callee, args, &env, depth);
                    // What the call may have changed. A language operation
                    // says exactly how; anything else lent mutably is now
                    // whatever its type allows.
                    let mut after = env.clone();
                    match callee {
                        Callee::Op(op) => {
                            let ops: Vec<Operand> = args.iter().map(arg_operand).collect();
                            self.apply_op_effect(f, &mut after, op, &ops);
                        }
                        _ => self.forget_mutable_lends(f, &mut after, args.iter().filter_map(|a| match a {
                            Arg::Lend { place, mutable: true } => Some(place.clone()),
                            _ => None,
                        })),
                    }
                    // `...give 'xs'` moves every element out and leaves the
                    // list empty ("a give never leaves holes").
                    for a in args {
                        if let Arg::Spread { place, verb: Some(crate::ast::Verb::Give), .. } = a {
                            if let Domain::Seq { elem, .. } = self.read(f, &after, place) {
                                self.store(f, &mut after, place, Domain::Seq { len: ISet::one(0), elem });
                            }
                        }
                    }
                    let mut e = after.clone();
                    if let Some(p) = dest {
                        self.store(f, &mut e, p, ret);
                    }
                    go(self, *ok, e, &mut states, &mut work);
                    for arm in fail {
                        let mut fe = after.clone();
                        if let Some(p) = &arm.payload {
                            let ty = self.place_ty(f, p);
                            self.store(f, &mut fe, p, Domain::of_ty(&ty, &self.prog().items));
                        }
                        go(self, arm.block, fe, &mut states, &mut work);
                    }
                }
                Terminator::Whichever { arms, closed } => {
                    for arm in arms {
                        let mut e = env.clone();
                        let ty = self.place_ty(f, &arm.dest);
                        self.store(f, &mut e, &arm.dest, Domain::of_ty(&ty, &self.prog().items));
                        go(self, arm.block, e, &mut states, &mut work);
                    }
                    if let Some(c) = closed {
                        go(self, *c, env.clone(), &mut states, &mut work);
                    }
                }
                Terminator::Join { ok, .. } => {
                    go(self, *ok, env.clone(), &mut states, &mut work);
                }
                Terminator::TaskOutcome { ok, fail, .. } => {
                    go(self, *ok, env.clone(), &mut states, &mut work);
                    for arm in fail {
                        let mut fe = env.clone();
                        if let Some(p) = &arm.payload {
                            let ty = self.place_ty(f, p);
                            self.store(f, &mut fe, p, Domain::of_ty(&ty, &self.prog().items));
                        }
                        go(self, arm.block, fe, &mut states, &mut work);
                    }
                }
                Terminator::Return => {
                    if let Some(r) = self.prog().functions[f].ret_local {
                        let d = env.get(&r).cloned().unwrap_or(Domain::Any);
                        returned = returned.union(&d);
                    }
                }
                Terminator::Fail { .. } | Terminator::Trap(_) | Terminator::Unreachable => {}
            }

        }
        for (id, v) in sites_seen {
            if self.dry {
                break;
            }
            let key = (f, id);
            let merged = match self.results.get(&key) {
                Some(old) => worse(old.clone(), v),
                None => v,
            };
            self.results.insert(key, merged);
        }
        returned
    }

    // ------------------------------------------------------------ reading

    fn local_ty(&self, f: usize, l: LocalId) -> Ty {
        self.prog().functions[f].locals[l.0 as usize].ty.clone()
    }

    fn place_ty(&self, f: usize, p: &Place) -> Ty {
        let base = self.local_ty(f, p.local);
        crate::mir::build::projected_ty(&base, &p.proj, &self.prog().items)
    }

    fn read(&mut self, f: usize, env: &Env, p: &Place) -> Domain {
        if p.proj.is_empty() {
            return env.get(&p.local).cloned().unwrap_or_else(|| Domain::of_ty(&self.local_ty(f, p.local), &self.prog().items));
        }
        // Reaching into a value: what is known about the part, when the
        // whole says something about it.
        let mut d = env.get(&p.local).cloned().unwrap_or_else(|| Domain::of_ty(&self.local_ty(f, p.local), &self.prog().items));
        for pr in &p.proj {
            d = match (pr, &d) {
                (Proj::Field(i), Domain::Struct(fields)) => fields.get(*i).cloned().unwrap_or(Domain::Any),
                (Proj::Index(_), Domain::Seq { elem, .. }) => (**elem).clone(),
                _ => Domain::Any,
            };
        }
        if matches!(d, Domain::Any) {
            let ty = self.place_ty(f, p);
            return Domain::of_ty(&ty, &self.prog().items);
        }
        d
    }

    fn read_operand(&mut self, f: usize, env: &Env, o: &Operand) -> Domain {
        match o {
            Operand::Const(c) => self.const_domain(f, c),
            Operand::Read(p) | Operand::Move(p) => self.read(f, env, p),
        }
    }

    fn const_domain(&mut self, f: usize, c: &Const) -> Domain {
        match c {
            Const::Int(v) => Domain::Ints(ISet::one(*v)),
            Const::Bin(t) => {
                let v = t.parse::<f64>().ok();
                Domain::Bins { lo: v, hi: v }
            }
            Const::Str(s) => Domain::Text { len: Lengths::of(s), values: Some(vec![s.clone()]), classes: classes_of(s) },
            Const::Char(c) => Domain::Chars { values: Some(vec![c.clone()]) },
            Const::Bool(b) => Domain::Bools(*b, !*b),
            Const::Duration(n) => Domain::Duration(ISet::one(*n)),
            Const::Const(id) => {
                let ty = self.prog().items.consts[id.0 as usize].ty.clone();
                match ty {
                    Some(t) => Domain::of_ty(&t, &self.prog().items),
                    None => Domain::Any,
                }
            }
            Const::SizeOf { .. } => Domain::Ints(ISet::Range { lo: 0, hi: 1 << 20 }),
            Const::Nothing => Domain::Any,
        }
        .clone_with(f)
    }

    fn store(&mut self, f: usize, env: &mut Env, p: &Place, d: Domain) {
        // A bin16 or bin32 is rounded to its width where it lands (oracle
        // 3996169; DECISIONS 2026-09-14: a bin32 0.1 is
        // 0.100000001490116119384765625). Kept at 64 bits, a bin32 0.1
        // folded `'c' > 0.1` to false.
        let d = match (&d, self.place_ty(f, p).base()) {
            (Domain::Bins { lo, hi }, Ty::Bin(crate::types::BinW::Fixed(bits @ (16 | 32)))) => Domain::Bins { lo: lo.map(|x| round_out(x, *bits, false)), hi: hi.map(|x| round_out(x, *bits, true)) },
            _ => d,
        };
        if p.proj.is_empty() {
            env.insert(p.local, d);
            return;
        }
        // Writing into a part changes that part, and nothing else: a list's
        // length survives an element being written, and a struct's other
        // fields survive one field being written.
        let whole = self.read(f, env, &Place::local(p.local));
        let updated = put(&whole, &p.proj, d);
        env.insert(p.local, updated);
    }

    // ------------------------------------------------------------ operations

    fn eval_rvalue(&mut self, f: usize, rv: &Rvalue, env: &Env, facts: &mut HashMap<LocalId, (Cmp, Operand, Operand)>, dest: &Place) -> Domain {
        match rv {
            Rvalue::Use(o) => self.read_operand(f, env, o),
            Rvalue::Duplicate(p) => self.read(f, env, p),
            Rvalue::Lend { place, .. } => self.read(f, env, place),
            Rvalue::Unary { op, operand } => {
                let d = self.read_operand(f, env, operand);
                match (op, &d) {
                    (UnOp::Neg, Domain::Ints(s)) => Domain::Ints(s.neg()),
                    // Negating a bin flips the interval; it never rounds, so
                    // the answer is exact whatever the width.
                    (UnOp::Neg, Domain::Bins { lo, hi }) => Domain::Bins { lo: hi.map(|x| -x), hi: lo.map(|x| -x) },
                    (UnOp::Not, Domain::Bools(t, fa)) => Domain::Bools(*fa, *t),
                    // `not.bit x` is `-x - 1` for a signed value, and
                    // `max - x` for an unsigned one: a range stays a range.
                    // Without it `not.bit 'v2'` could be anything, 0 too,
                    // and a divisor by it was refused (check claude, seed 316).
                    (UnOp::NotBit, Domain::Ints(s)) => {
                        let ty = self.place_ty(f, dest);
                        // Only for a value inside the type: the low bits of a
                        // larger one (an exact answer) can be anything.
                        match (int_limits(&ty, &self.prog().items), s.bounds()) {
                            (Some((lo, hi)), Some((a, z))) if lo < 0 && a >= lo && z <= hi => Domain::Ints(ISet::range(-z - 1, -a - 1)),
                            (Some((0, hi)), Some((a, z))) if a >= 0 && z <= hi => Domain::Ints(ISet::range(hi - z, hi - a)),
                            _ => match s.single() {
                                // One known value: its low bits, flipped.
                                Some(v) => match int_limits(&ty, &self.prog().items) {
                                    Some((lo, hi)) => {
                                        let span = hi - lo + 1;
                                        let flipped = (!v).rem_euclid(span) + lo;
                                        let flipped = if lo < 0 && flipped > hi { flipped - span } else { flipped };
                                        Domain::Ints(ISet::one(flipped))
                                    }
                                    None => Domain::of_ty(&ty, &self.prog().items),
                                },
                                None => Domain::of_ty(&ty, &self.prog().items),
                            },
                        }
                    }
                    _ => Domain::of_ty(&self.place_ty(f, dest), &self.prog().items),
                }
            }
            Rvalue::Binary { op, lhs, rhs } => {
                let a = self.read_operand(f, env, lhs);
                let b = self.read_operand(f, env, rhs);
                if let Some(cmp) = cmp_of(op) {
                    if dest.proj.is_empty() {
                        facts.insert(dest.local, (cmp, lhs.clone(), rhs.clone()));
                    }
                }
                let r = self.binary(f, op, &a, &b, dest);
                // A sum over a list's elements divided by its length: the
                // average stays within the elements' range (`precog.md`,
                // the loop-invariant template "a sum is bounded by trips
                // times the largest element").
                if *op == BinOp::Div {
                    if let (Domain::Ints(s), Some((lo, hi))) = (&r, mean_bound(&self.prog().functions[f], lhs, rhs)) {
                        return Domain::Ints(s.clamp(lo, hi));
                    }
                }
                r
            }
            Rvalue::Convert { to, operand } => {
                let d = self.read_operand(f, env, operand);
                self.convert(to, &d)
            }
            Rvalue::MakeList(items) => {
                let mut elem = Domain::Empty;
                for it in items {
                    let d = self.read_operand(f, env, it);
                    elem = elem.union(&d);
                }
                Domain::Seq { len: ISet::one(items.len() as i128), elem: Box::new(elem) }
            }
            // The keys a map can hold, as a set that may be larger than the
            // truth: a new map's are its literal's (none for `map.empty`).
            Rvalue::MakeMap(pairs) => {
                let (mut key, mut val) = (Domain::Empty, Domain::Empty);
                for (k, v) in pairs {
                    key = key.union(&self.read_operand(f, env, k));
                    val = val.union(&self.read_operand(f, env, v));
                }
                Domain::Map { len: ISet::one(pairs.len() as i128), key: Box::new(key), val: Box::new(val) }
            }
            Rvalue::MakeStruct { fields, .. } => {
                let ds = fields.iter().map(|o| self.read_operand(f, env, o)).collect();
                Domain::Struct(ds)
            }
            Rvalue::MakeEnum { case, payload, .. } => {
                let d = payload.as_ref().map(|o| self.read_operand(f, env, o)).unwrap_or(Domain::Any);
                Domain::Enum(vec![(*case, d)])
            }
            Rvalue::MakeChannel { .. } | Rvalue::FnRef(_) | Rvalue::Closure { .. } => Domain::Any,
            Rvalue::ExternRead(_) => {
                // C's memory: nothing about it can be known.
                self.reads_outside = true;
                Domain::of_ty(&self.place_ty(f, dest), &self.prog().items)
            }
        }
    }

    fn binary(&mut self, f: usize, op: &BinOp, a: &Domain, b: &Domain, dest: &Place) -> Domain {
        let ty = self.place_ty(f, dest);
        if let (Domain::Ints(x), Domain::Ints(y)) = (a, b) {
            let s = match op {
                BinOp::Add => x.add(y),
                BinOp::Sub => x.sub(y),
                BinOp::Mul => x.mul(y),
                BinOp::Div => x.div_floor(y),
                BinOp::Mod => x.mod_floor(y),
                BinOp::Pow => x.pow(y),
                // Wrapping arithmetic is exact modulo the width; the domain
                // is only narrowed when the plain answer already fits.
                BinOp::WrapAdd | BinOp::WrapSub | BinOp::WrapMul | BinOp::WrapPow => {
                    let plain = match op {
                        BinOp::WrapAdd => x.add(y),
                        BinOp::WrapSub => x.sub(y),
                        BinOp::WrapMul => x.mul(y),
                        _ => x.pow(y),
                    };
                    match int_limits(&ty, &self.prog().items) {
                        Some((lo, hi)) if plain.within(lo, hi) => plain,
                        Some((lo, hi)) => ISet::range(lo, hi),
                        None => return Domain::of_ty(&ty, &self.prog().items),
                    }
                }
                // Known values are executed outright (`precog.md`, layer 1):
                // `5 and.bit 0` is 0, not "anything a `uint8` holds".
                BinOp::AndBit | BinOp::OrBit | BinOp::Xor | BinOp::Shl | BinOp::Shr => match x.bitwise(y, op) {
                    Some(s) => s,
                    None => return Domain::of_ty(&ty, &self.prog().items),
                },
                _ => {
                    let cmp = cmp_of(op);
                    return match cmp {
                        Some(c) => {
                            let (t, fa) = x.compare(y, c);
                            Domain::Bools(t, fa)
                        }
                        None => Domain::of_ty(&ty, &self.prog().items),
                    };
                }
            };
            if s.is_empty() {
                // Nothing came back: every pair had no answer (a division by
                // zero every time). The site says so; the value is nothing.
                return Domain::Empty;
            }
            return Domain::Ints(s);
        }
        if let (Domain::Duration(x), Domain::Duration(y)) = (a, b) {
            return match op {
                BinOp::Add => Domain::Duration(x.add(y)),
                BinOp::Sub => Domain::Duration(x.sub(y)),
                _ => match cmp_of(op) {
                    Some(c) => {
                        let (t, fa) = x.compare(y, c);
                        Domain::Bools(t, fa)
                    }
                    None => Domain::of_ty(&ty, &self.prog().items),
                },
            };
        }
        if let (Domain::Bins { .. }, Domain::Bins { .. }) = (a, b) {
            if let Some(c) = cmp_of(op) {
                let (t, fa) = a.compare_bin(b, c);
                return Domain::Bools(t, fa);
            }
            // "Bins are intervals from the start" (`precog.md`). IEEE
            // rounding is monotone, so over a box the answers lie between the
            // corners' answers; each corner is worked out in 64 bits and
            // widened a unit in the last place at the result's own width,
            // which covers the rounding (and a double rounding) either way.
            // A divisor that can be 0, or an answer that can leave the
            // type's finite range, gives the type's range and no narrower.
            // A width not chosen yet is worked at 64 bits, as codegen does.
            let bits = match ty.base() {
                Ty::Bin(crate::types::BinW::Fixed(b)) => Some(*b),
                Ty::Bin(crate::types::BinW::Free(_)) => Some(64),
                _ => None,
            };
            if let (Some(bits), Domain::Bins { lo: Some(a1), hi: Some(a2) }, Domain::Bins { lo: Some(b1), hi: Some(b2) }) = (bits, a, b) {
                if let Some(d) = bin_interval(op, (*a1, *a2), (*b1, *b2), bits) {
                    return d;
                }
            }
            return Domain::of_ty(&ty, &self.prog().items);
        }
        if let (Domain::Bools(t1, f1), Domain::Bools(t2, f2)) = (a, b) {
            return match op {
                BinOp::And => Domain::Bools(*t1 && *t2, *f1 || *f2),
                BinOp::Or => Domain::Bools(*t1 || *t2, *f1 && *f2),
                BinOp::Eq => Domain::Bools((*t1 && *t2) || (*f1 && *f2), (*t1 && *f2) || (*f1 && *t2)),
                BinOp::Ne => Domain::Bools((*t1 && *f2) || (*f1 && *t2), (*t1 && *t2) || (*f1 && *f2)),
                _ => Domain::of_ty(&ty, &self.prog().items),
            };
        }
        // Text: a join's length is the sum of the two, and its value is
        // known when both are.
        if let (Domain::Text { len: l1, values: v1, classes: c1 }, Domain::Text { len: l2, values: v2, classes: c2 }) = (a, b) {
            if *op == BinOp::Join {
                let values = match (v1, v2) {
                    (Some(x), Some(y)) if x.len() * y.len() <= 64 => {
                        let mut out = Vec::new();
                        for p in x {
                            for q in y {
                                out.push(format!("{}{}", p, q));
                            }
                        }
                        out.sort();
                        out.dedup();
                        Some(out)
                    }
                    _ => None,
                };
                let classes: Vec<Class> = c1.iter().copied().filter(|c| c2.contains(c) && *c != Class::NonEmpty).collect();
                return Domain::Text {
                    len: Lengths { bytes: l1.bytes.add(&l2.bytes), points: l1.points.add(&l2.points), graphemes: l1.graphemes.add(&l2.graphemes) },
                    values,
                    classes,
                };
            }
            if let Some(c) = cmp_of(op) {
                if matches!(c, Cmp::Eq | Cmp::Ne) {
                    if let (Some(x), Some(y)) = (v1, v2) {
                        if x.len() == 1 && y.len() == 1 {
                            let equal = x[0] == y[0];
                            return match c {
                                Cmp::Eq => Domain::Bools(equal, !equal),
                                _ => Domain::Bools(!equal, equal),
                            };
                        }
                    }
                    // Lengths that cannot match rule equality out.
                    let can_equal = !l1.bytes.meet(&l2.bytes).is_empty();
                    return match c {
                        Cmp::Eq => Domain::Bools(can_equal, true),
                        _ => Domain::Bools(true, can_equal),
                    };
                }
            }
        }
        if let (Domain::Chars { values: Some(x) }, Domain::Chars { values: Some(y) }) = (a, b) {
            if let Some(c) = cmp_of(op) {
                if matches!(c, Cmp::Eq | Cmp::Ne) && x.len() == 1 && y.len() == 1 {
                    let equal = x[0] == y[0];
                    return match c {
                        Cmp::Eq => Domain::Bools(equal, !equal),
                        _ => Domain::Bools(!equal, equal),
                    };
                }
            }
        }
        Domain::of_ty(&ty, &self.prog().items)
    }

    fn convert(&mut self, to: &Ty, from: &Domain) -> Domain {
        let top = Domain::of_ty(to, &self.prog().items);
        match (from, &top) {
            // Into a wider or equal integer: the same values. Into a narrower
            // one: a narrowing is a check site, and past it the value is one
            // the target holds — or the program stopped there. So it is the
            // values it had, clipped to the target's, not "anything the target
            // allows": a loop counter stepped through 128 bits and narrowed
            // back is still at least 1.
            (Domain::Ints(s), Domain::Ints(limit)) => {
                let (Some(_), Some((l2, h2))) = (s.bounds(), limit.bounds()) else { return top };
                Domain::Ints(s.clamp(l2, h2))
            }
            (Domain::Ints(s), Domain::Text { .. }) => {
                // A number written out: its length follows from its digits.
                let len = match s.bounds() {
                    Some((lo, hi)) => {
                        let d = |v: i128| v.to_string().len() as i128;
                        ISet::range(d(lo).min(d(hi)).min(1), d(lo).max(d(hi)))
                    }
                    None => ISet::Range { lo: 1, hi: 40 },
                };
                Domain::Text { len: Lengths { bytes: len.clone(), points: len.clone(), graphemes: len }, values: None, classes: vec![Class::Ascii, Class::NoNewline, Class::NonEmpty] }
            }
            // A bin into a whole number: past the call it was whole, so it is
            // one of the whole values in its interval. Into the exact type a
            // value past 64 bits lands as one no 64-bit type holds (codegen),
            // which the set has to leave room for.
            (Domain::Bins { lo: Some(lo), hi: Some(hi) }, Domain::Ints(limit)) => {
                let (Some((l2, h2)), true) = (limit.bounds(), lo.ceil() <= hi.floor()) else { return top };
                let (l, h) = (lo.ceil().max(-1e38) as i128, hi.floor().min(1e38) as i128);
                self.whole_into(to, l, h, l2, h2)
            }
            // Text whose values are all known: their numbers.
            (Domain::Text { values: Some(vs), .. }, Domain::Ints(limit)) => {
                let Some((l2, h2)) = limit.bounds() else { return top };
                let nums: Vec<i128> = vs.iter().filter_map(|v| v.parse::<i128>().ok()).collect();
                if nums.is_empty() {
                    // None of them is a number: the call always fails.
                    return Domain::Empty;
                }
                let (l, h) = (*nums.iter().min().unwrap(), *nums.iter().max().unwrap());
                self.whole_into(to, l, h, l2, h2)
            }
            (_, Domain::Ints(_)) if to.is_exact() && !matches!(from, Domain::Ints(_)) => {
                Domain::Ints(ISet::Range { lo: i64::MIN as i128 - 1, hi: u64::MAX as i128 })
            }
            _ => top,
        }
    }

    /// Whole values `l..=h` converted into `to`, whose range is `l2..=h2`.
    /// A range type fails `out_of_range` outside it, so the values are
    /// clipped; the exact type holds past-64-bit values as its marker.
    fn whole_into(&self, to: &Ty, l: i128, h: i128, l2: i128, h2: i128) -> Domain {
        if to.is_exact() {
            let lo = if l < l2 || h < l2 { i64::MIN as i128 - 1 } else { l };
            let hi = if h > h2 { h2 } else { h };
            let lo = if h > h2 { lo.min(i64::MIN as i128 - 1) } else { lo };
            return Domain::Ints(ISet::range(lo.min(hi), hi.max(lo)));
        }
        let (lo, hi) = (l.max(l2), h.min(h2));
        if lo > hi {
            return Domain::Empty;
        }
        Domain::Ints(ISet::range(lo, hi))
    }

    fn eval_op(&mut self, f: usize, op: &Op, args: &[Operand], env: &Env) -> Domain {
        let arg = |this: &mut Self, i: usize| -> Domain { args.get(i).map(|a| this.read_operand(f, env, a)).unwrap_or(Domain::Any) };
        match op {
            Op::ListLength | Op::ArrayLength => match arg(self, 0) {
                Domain::Seq { len, .. } => Domain::Ints(len),
                _ => Domain::Ints(ISet::Range { lo: 0, hi: i64::MAX as i128 }),
            },
            // The same elements, the same count: only whether it can grow.
            Op::ToList | Op::ToArray => match arg(self, 0) {
                d @ Domain::Seq { .. } => d,
                _ => Domain::Seq { len: ISet::Range { lo: 0, hi: i64::MAX as i128 }, elem: Box::new(Domain::Any) },
            },
            // As long as its positions say, and never longer than the list;
            // its elements are the list's.
            Op::ListSlice => {
                let (len, elem) = match arg(self, 0) {
                    Domain::Seq { len, elem } => (len, elem),
                    _ => (ISet::Range { lo: 0, hi: i64::MAX as i128 }, Box::new(Domain::Any)),
                };
                let (a, b) = (arg(self, 1), arg(self, 2));
                let n = match (a, b) {
                    (Domain::Ints(a), Domain::Ints(b)) => b.sub(&a).add(&ISet::one(1)),
                    _ => ISet::Range { lo: 0, hi: i64::MAX as i128 },
                };
                let top = len.bounds().map(|(_, h)| h).unwrap_or(i64::MAX as i128);
                Domain::Seq { len: n.clamp(0, top), elem }
            }
            Op::MapLength => match arg(self, 0) {
                Domain::Map { len, .. } => Domain::Ints(len),
                _ => Domain::Ints(ISet::Range { lo: 0, hi: i64::MAX as i128 }),
            },
            Op::StrBytes => match arg(self, 0) {
                Domain::Text { len, .. } => Domain::Ints(len.bytes),
                _ => Domain::Ints(ISet::Range { lo: 0, hi: i64::MAX as i128 }),
            },
            Op::StrPoints => match arg(self, 0) {
                Domain::Text { len, .. } => Domain::Ints(len.points),
                _ => Domain::Ints(ISet::Range { lo: 0, hi: i64::MAX as i128 }),
            },
            Op::StrGraphemes => match arg(self, 0) {
                Domain::Text { len, .. } => Domain::Ints(len.graphemes),
                _ => Domain::Ints(ISet::Range { lo: 0, hi: i64::MAX as i128 }),
            },
            Op::StrEach(u) => {
                let (len, elem) = match arg(self, 0) {
                    Domain::Text { len, .. } => (len.get(*u).clone(), Domain::Any),
                    _ => (ISet::Range { lo: 0, hi: i64::MAX as i128 }, Domain::Any),
                };
                Domain::Seq { len, elem: Box::new(elem) }
            }
            Op::MapKeys | Op::MapValues | Op::MapPairs => {
                let len = match arg(self, 0) {
                    Domain::Map { len, .. } => len,
                    _ => ISet::Range { lo: 0, hi: i64::MAX as i128 },
                };
                Domain::Seq { len, elem: Box::new(Domain::Any) }
            }
            Op::MapHas | Op::PtrIsNull | Op::SimdAll | Op::SimdAny => Domain::Bools(true, true),
            Op::CharCode => Domain::Ints(ISet::Range { lo: 0, hi: 0x10FFFF }),
            Op::Std(name) => {
                if name.starts_with("input") {
                    self.reads_outside = true;
                }
                Domain::Any
            }
            _ => Domain::Any,
        }
    }

    /// What an operation does to what was known about its arguments.
    fn apply_op_effect(&mut self, f: usize, env: &mut Env, op: &Op, args: &[Operand]) {
        let target = match args.first().and_then(|a| a.place()) {
            Some(p) => p.clone(),
            None => return,
        };
        let before = self.read(f, env, &target);
        let after = match (op, before) {
            (Op::ListAppend, Domain::Seq { len, elem }) => {
                let added = args.get(1).map(|a| self.read_operand(f, env, a)).unwrap_or(Domain::Any);
                Domain::Seq { len: len.add(&ISet::one(1)), elem: Box::new(elem.union(&added)) }
            }
            (Op::ListRemove, Domain::Seq { len, elem }) => Domain::Seq { len: len.sub(&ISet::one(1)).clamp(0, i64::MAX as i128), elem },
            // A key that is one known value, outside every key the map can
            // hold, is a new entry: exactly one more (`map_bound_over`'s
            // third `set`). Otherwise it may replace one: none or one more.
            // Either way the keys and values the map can hold grow by it.
            (Op::MapSet, Domain::Map { len, key, val }) => {
                let k = args.get(1).map(|a| self.read_operand(f, env, a)).unwrap_or(Domain::Any);
                let v = args.get(2).map(|a| self.read_operand(f, env, a)).unwrap_or(Domain::Any);
                let grows = if surely_absent(&key, &k) { ISet::one(1) } else { ISet::range(0, 1) };
                Domain::Map { len: len.add(&grows).clamp(0, i64::MAX as i128), key: Box::new(key.union(&k)), val: Box::new(val.union(&v)) }
            }
            (Op::MapRemove, Domain::Map { len, key, val }) => Domain::Map { len: len.sub(&ISet::range(0, 1)).clamp(0, i64::MAX as i128), key, val },
            // A store writes N elements whose lanes Precog does not follow
            // (`simd.md`, a domain per lane, is not built yet): the length
            // stays, the elements become anything. Keeping them as they
            // were folded an `if` about a stored element to false.
            (Op::SimdStore, Domain::Seq { len, .. }) => Domain::Seq { len, elem: Box::new(Domain::Any) },
            _ => return,
        };
        env.insert(target.local, after);
    }

    fn call(&mut self, f: usize, callee: &Callee, args: &[Arg], env: &Env, depth: u32) -> Domain {
        match callee {
            Callee::Fn(idx) => {
                let i = idx.0 as usize;
                if i >= self.prog().functions.len() {
                    return Domain::Any;
                }
                // Per call site, with these argument domains: the same
                // machinery generics and variadics already use.
                let mut callee_env = self.entry_env(i);
                let params = self.prog().functions[i].params.clone();
                let mut key = String::new();
                // A variadic's last parameter is the bundle the call builds
                // from every argument past the fixed ones — not the first of
                // them. Bound one to one, `count[...'ys', 1]` saw only 'ys'
                // and folded `== 3` to false.
                let variadic = self.prog().functions[i].variadic;
                let fixed = if variadic { params.len().saturating_sub(1) } else { params.len() };
                for (p, a) in params[..fixed].iter().zip(args) {
                    let d = self.arg_domain(f, env, a);
                    // `key`, never `describe`: a description is prose, and
                    // two different arguments that read the same would
                    // share one answer.
                    key.push_str(&d.key());
                    key.push(';');
                    callee_env.insert(*p, d);
                }
                if variadic {
                    if let Some(bp) = params.last().copied() {
                        let bty = self.prog().functions[i].locals[bp.0 as usize].ty.clone();
                        let d = match bty.base() {
                            Ty::Seq { .. } => {
                                let mut len = ISet::one(0);
                                let mut elem: Option<Domain> = None;
                                for a in args.iter().skip(fixed) {
                                    let (n, e) = match a {
                                        Arg::Spread { place, .. } => match self.read(f, env, place) {
                                            Domain::Seq { len, elem } => (len, *elem),
                                            _ => (ISet::Range { lo: 0, hi: i64::MAX as i128 }, Domain::Any),
                                        },
                                        _ => (ISet::one(1), self.arg_domain(f, env, a)),
                                    };
                                    len = len.add(&n);
                                    elem = Some(match elem {
                                        Some(x) => x.union(&e),
                                        None => e,
                                    });
                                }
                                Domain::Seq { len, elem: Box::new(elem.unwrap_or(Domain::Any)) }
                            }
                            _ => Domain::Any,
                        };
                        key.push_str(&d.key());
                        key.push(';');
                        callee_env.insert(bp, d);
                    }
                }
                // A recursive call while this function's table is being
                // filled: answered from the table, for every tuple the
                // arguments can be.
                if let Some(tuples) = self.filling(i, &params, &callee_env) {
                    return tuples;
                }
                if let Some(cached) = self.cache.get(&(i, key.clone())) {
                    return cached.clone();
                }
                // A function that calls itself, on arguments with few
                // enough values: proven by induction instead of followed
                // until the depth runs out.
                if let Some(d) = self.by_induction(i, &params, &callee_env, depth) {
                    self.remember(i, key, d.clone());
                    return d;
                }
                if depth >= CALL_DEPTH {
                    self.gave_up = true;
                    let ret = self.prog().functions[i].ret.clone();
                    return Domain::of_ty(&ret, &self.prog().items);
                }
                // Mark it before walking, so a recursive call finds something.
                let ret_ty = self.prog().functions[i].ret.clone();
                self.remember(i, key.clone(), Domain::of_ty(&ret_ty, &self.prog().items));
                let d = self.run_function(i, callee_env, depth + 1);
                let d = if matches!(d, Domain::Empty) { Domain::of_ty(&ret_ty, &self.prog().items) } else { d };
                self.remember(i, key, d.clone());
                d
            }
            Callee::Extern(id) => {
                // "An `extern` result is anything its C-shaped type holds"
                // (`precog.md`) — or, with a ranged return, the range, which
                // the door already enforced as a failure.
                self.reads_outside = true;
                let ret = self.prog().items.externs[id.0 as usize].ret.clone();
                Domain::of_ty(&ret, &self.prog().items)
            }
            Callee::Value(_) => {
                // A held function: which one it is was not followed here.
                self.gave_up = true;
                Domain::Any
            }
            Callee::Op(op) => {
                let ops: Vec<Operand> = args.iter().map(arg_operand).collect();
                self.eval_op(f, op, &ops, env)
            }
            Callee::Convert(to) => {
                let d = args.first().map(|a| self.arg_domain(f, env, a)).unwrap_or(Domain::Any);
                self.convert(to, &d)
            }
        }
    }

    /// A value lent mutably to code Precog did not follow step by step — a
    /// function's body, a task running alongside, C — may come back as
    /// anything its type allows. Keeping what was known before would make a
    /// check look proven after a call that emptied the list it guards.
    fn forget_mutable_lends(&mut self, f: usize, env: &mut Env, places: impl Iterator<Item = Place>) {
        for place in places {
            let ty = self.place_ty(f, &place);
            let d = Domain::of_ty(&ty, &self.prog().items);
            self.store(f, env, &place, d);
        }
    }

    fn arg_domain(&mut self, f: usize, env: &Env, a: &Arg) -> Domain {
        match a {
            Arg::Value(o) => self.read_operand(f, env, o),
            Arg::Lend { place, .. } | Arg::Give(place) | Arg::Copy(place) => self.read(f, env, place),
            Arg::Spread { place, .. } => self.read(f, env, place),
        }
    }

    // ------------------------------------------------------------ branches

    fn bool_of(&mut self, f: usize, cond: &Operand, env: &Env) -> (bool, bool) {
        match self.read_operand(f, env, cond) {
            Domain::Bools(t, fa) => (t, fa),
            _ => (true, true),
        }
    }

    /// What the program knows inside a branch: the condition assumed.
    fn narrow(&mut self, f: usize, env: &Env, cond: &Operand, facts: &HashMap<LocalId, (Cmp, Operand, Operand)>, lengths: &HashMap<LocalId, (Op, Place)>, taken: bool) -> Env {
        let mut out = env.clone();
        let Some(place) = cond.place() else { return out };
        if !place.proj.is_empty() {
            return out;
        }
        out.insert(place.local, Domain::Bools(taken, !taken));
        let Some((cmp, lhs, rhs)) = facts.get(&place.local) else { return out };
        let cmp = if taken { *cmp } else { cmp.negate() };
        let a = self.read_operand(f, env, lhs);
        let b = self.read_operand(f, env, rhs);
        if let (Domain::Bins { .. }, Domain::Bins { .. }) = (&a, &b) {
            if let Some(pl) = lhs.place().filter(|p| p.proj.is_empty()) {
                out.insert(pl.local, a.assume_bin(&b, cmp));
            }
            if let Some(pr) = rhs.place().filter(|p| p.proj.is_empty()) {
                out.insert(pr.local, b.assume_bin(&a, cmp.flip()));
            }
            return out;
        }
        if let (Domain::Ints(x), Domain::Ints(y)) = (&a, &b) {
            if let Some(pl) = lhs.place().filter(|p| p.proj.is_empty()) {
                let narrowed = x.assume(y, cmp);
                out.insert(pl.local, Domain::Ints(narrowed.clone()));
                self.narrow_source(f, &mut out, pl.local, &narrowed, lengths);
            }
            if let Some(pr) = rhs.place().filter(|p| p.proj.is_empty()) {
                let narrowed = y.assume(x, cmp.flip());
                out.insert(pr.local, Domain::Ints(narrowed.clone()));
                self.narrow_source(f, &mut out, pr.local, &narrowed, lengths);
            }
        }
        out
    }

    /// A number that is a value's length, narrowed: the value it came from
    /// is narrowed too, so the next question about it knows the answer.
    fn narrow_source(&mut self, f: usize, env: &mut Env, num: LocalId, narrowed: &ISet, lengths: &HashMap<LocalId, (Op, Place)>) {
        let Some((op, src)) = lengths.get(&num) else { return };
        let src = src.clone();
        let op = op.clone();
        let whole = self.read(f, env, &src);
        let updated = match (&op, whole) {
            (Op::ListLength | Op::ArrayLength, Domain::Seq { len, elem }) => Domain::Seq { len: len.meet(narrowed), elem },
            (Op::MapLength, Domain::Map { len, key, val }) => Domain::Map { len: len.meet(narrowed), key, val },
            (Op::StrBytes, Domain::Text { mut len, values, classes }) => {
                len.bytes = len.bytes.meet(narrowed);
                Domain::Text { len, values, classes }
            }
            (Op::StrPoints, Domain::Text { mut len, values, classes }) => {
                len.points = len.points.meet(narrowed);
                Domain::Text { len, values, classes }
            }
            (Op::StrGraphemes, Domain::Text { mut len, values, classes }) => {
                len.graphemes = len.graphemes.meet(narrowed);
                Domain::Text { len, values, classes }
            }
            _ => return,
        };
        // Through a field, too: `'s'.scores` narrows inside `'s'`.
        let outer = self.read(f, env, &Place::local(src.local));
        let merged = put(&outer, &src.proj, updated);
        env.insert(src.local, merged);
    }

    // ------------------------------------------------------------ the question

    /// Is every value reaching this site legal?
    fn judge(&mut self, f: usize, id: SiteId, env: &Env) -> Verdict {
        let site = self.prog().functions[f].sites[id.0 as usize].clone();
        let ds: Vec<Domain> = site.operands.iter().map(|o| self.read_operand(f, env, o)).collect();
        let ty_of = |this: &Self, i: usize| -> Option<Ty> { site.operands.get(i).and_then(|o| o.place()).map(|p| this.place_ty(f, p)) };
        let verdict = self.judge_kind(f, &site, &ds, ty_of);
        if !self.dry {
            let value = claimed_value(&site.kind, &ds);
            let entry = self.claims.entry((f, id.0)).or_insert((None, Vec::new()));
            entry.0 = match (entry.0.take(), value) {
                (Some(a), Some(b)) => Some(a.union(&b)),
                (a, b) => a.or(b),
            };
            if entry.1.len() == ds.len() {
                for (old, new) in entry.1.iter_mut().zip(&ds) {
                    *old = old.union(new);
                }
            } else {
                entry.1 = ds.clone();
            }
        }
        if site.tier == Some(Tier::Nocheck) {
            // The check is removed on the writer's word. But the domains
            // were computed anyway, and if they say this fails every time
            // the build says so once and carries on.
            if let Verdict::Fails { counterexample } = &verdict {
                if counterexample.starts_with("every") && !self.dry {
                    self.lies.push((f, id, counterexample.trim_start_matches("every time: ").to_string()));
                }
            }
            return Verdict::Trusted;
        }
        verdict
    }

    fn judge_kind(&mut self, _f: usize, site: &Site, ds: &[Domain], ty_of: impl Fn(&Self, usize) -> Option<Ty>) -> Verdict {
        match &site.kind {
            SiteKind::Overflow(op) => {
                let ty = match site.result_ty.clone().or_else(|| ty_of(self, 0)) {
                    Some(t) => t,
                    None => return Verdict::Unproven,
                };
                let Some((lo, hi)) = int_limits(&ty, &self.prog().items) else { return Verdict::Unproven };
                let (Some(Domain::Ints(a)), b) = (ds.first(), ds.get(1)) else { return Verdict::Unproven };
                let result = match (op, b) {
                    (BinOp::Add, Some(Domain::Ints(y))) => a.add(y),
                    (BinOp::Sub, Some(Domain::Ints(y))) => a.sub(y),
                    (BinOp::Mul, Some(Domain::Ints(y))) => a.mul(y),
                    (BinOp::Pow, Some(Domain::Ints(y))) => a.pow(y),
                    (BinOp::Div, Some(Domain::Ints(y))) => a.div_floor(y),
                    // A negation, or a fold whose shape is not two operands.
                    (BinOp::Sub, None) => a.neg(),
                    _ => return Verdict::Unproven,
                };
                self.fit(&result, lo, hi, &ty)
            }
            SiteKind::DivZero | SiteKind::ModZero => {
                let Some(Domain::Ints(d)) = ds.first() else { return Verdict::Unproven };
                if d.misses(0, 0) {
                    Verdict::Proven { how: "the divisor is never zero" }
                } else if d.single() == Some(0) {
                    Verdict::Fails { counterexample: "every time: the divisor is always zero".into() }
                } else {
                    Verdict::Fails { counterexample: "the divisor can be 0".into() }
                }
            }
            SiteKind::Bounds => {
                let (Some(Domain::Ints(i)), Some(Domain::Ints(len))) = (ds.first(), ds.get(1)) else { return Verdict::Unproven };
                // 1-based: legal is 1 ..= len.
                let (Some((ilo, ihi)), Some((llo, lhi))) = (i.bounds(), len.bounds()) else { return Verdict::Unproven };
                if ilo >= 1 && ihi <= llo {
                    Verdict::Proven { how: "the position is always inside" }
                } else if ihi < 1 || ilo > lhi {
                    Verdict::Fails { counterexample: format!("every time: the position is {} and there {} {} element(s)", describe_ints(i), if lhi == 1 { "is" } else { "are" }, describe_ints(len)) }
                } else {
                    Verdict::Fails { counterexample: format!("the position can be {} when there are {} element(s)", ihi, llo) }
                }
            }
            SiteKind::Narrow { to } => {
                // A duration's site carries both sides, because what has to
                // fit is the difference.
                if *to == Ty::Duration {
                    let (Some(Domain::Duration(a)), Some(Domain::Duration(b))) = (ds.first(), ds.get(1)) else { return Verdict::Unproven };
                    return self.fits_type(&Domain::Duration(a.sub(b)), to);
                }
                // A literal into a plain bin16 or bin32 is decided from its
                // written digits, rounded once as codegen writes it: read at
                // 64 bits first, `…568447.0` landed on bin32's limit and was
                // refused, though it rounds to bin32's largest.
                if let (Some(Operand::Const(Const::Bin(text))), None) = (site.operands.first(), to.bound()) {
                    if let Some(fits) = literal_fits_bin(text, to) {
                        return if fits { Verdict::Proven { how: "layer 2" } } else { Verdict::Fails { counterexample: format!("every time: {} rounds past the largest `{}`", text, self.spell(to)) } };
                    }
                }
                let Some(d) = ds.first() else { return Verdict::Unproven };
                self.fits_type(d, to)
            }
            SiteKind::LengthBound { to } => {
                let Some(d) = ds.first() else { return Verdict::Unproven };
                self.fits_type(d, to)
            }
            SiteKind::ShiftWidth | SiteKind::ShiftCount => {
                // A count outside 0 to width - 1, for `shl` and `shr` alike
                // (owner, 2026-09-23); and for `shl`, an answer that no
                // longer fits the type.
                let ty = match site.result_ty.clone().or_else(|| ty_of(self, 0)) {
                    Some(t) => t,
                    None => return Verdict::Unproven,
                };
                let Some((lo, hi)) = int_limits(&ty, &self.prog().items) else { return Verdict::Unproven };
                let width = match ty.base() {
                    Ty::Int(crate::types::IntW::Fixed { bits, .. }) => *bits as i128,
                    _ => return Verdict::Unproven,
                };
                let (Some(Domain::Ints(a)), Some(Domain::Ints(b))) = (ds.first(), ds.get(1)) else { return Verdict::Unproven };
                let (Some((blo, bhi)), Some(_)) = (b.bounds(), a.bounds()) else { return Verdict::Unproven };
                if blo >= width {
                    return Verdict::Fails { counterexample: format!("every time: a shift of {} moves every bit out of a `{}`, which is {} bits wide", blo, self.spell(&ty), width) };
                }
                if bhi < 0 {
                    return Verdict::Fails { counterexample: format!("every time: a shift of {} — a count is never negative; the other direction is the other word", bhi) };
                }
                if bhi >= width {
                    return Verdict::Fails { counterexample: format!("a shift of {} would move every bit out of a `{}`, which is {} bits wide", bhi, self.spell(&ty), width) };
                }
                if blo < 0 {
                    return Verdict::Fails { counterexample: format!("the count can be {}, and a count is never negative", blo) };
                }
                if matches!(site.kind, SiteKind::ShiftCount) {
                    return Verdict::Proven { how: "layer 2" };
                }
                let shifted = a.mul(&ISet::range(1 << blo.max(0), 1 << bhi.max(0)));
                self.fit(&shifted, lo, hi, &ty)
            }
            SiteKind::NegativeExponent => {
                let Some(Domain::Ints(e)) = ds.first() else { return Verdict::Unproven };
                let Some((lo, hi)) = e.bounds() else { return Verdict::Unproven };
                if lo >= 0 {
                    Verdict::Proven { how: "the exponent is never negative" }
                } else if hi < 0 {
                    Verdict::Fails { counterexample: format!("every time: the exponent is {}", describe_ints(e)) }
                } else {
                    Verdict::Fails { counterexample: format!("the exponent can be {}", lo) }
                }
            }
            SiteKind::SliceRange => {
                let (Some(Domain::Ints(a)), Some(Domain::Ints(b)), Some(Domain::Ints(n))) = (ds.first(), ds.get(1), ds.get(2)) else { return Verdict::Unproven };
                let (Some((alo, ahi)), Some((blo, bhi)), Some((nlo, nhi))) = (a.bounds(), b.bounds(), n.bounds()) else { return Verdict::Unproven };
                if alo >= 1 && ahi <= blo.saturating_add(1) && bhi <= nlo {
                    Verdict::Proven { how: "the slice is always inside" }
                } else if ahi < 1 || blo > nhi {
                    Verdict::Fails { counterexample: format!("every time: the slice from {} to {} runs outside a list of {} element(s)", describe_ints(a), describe_ints(b), describe_ints(n)) }
                } else if alo > bhi.saturating_add(1) {
                    Verdict::Fails { counterexample: format!("every time: the slice from {} to {} runs backwards; an empty slice is `pos n, pos n - 1`", describe_ints(a), describe_ints(b)) }
                } else {
                    Verdict::Fails { counterexample: format!("the slice can be from {} to {} when there are {} element(s)", alo.min(ahi), bhi, nlo) }
                }
            }
            SiteKind::SimdRange { lanes } => {
                let (Some(Domain::Ints(i)), Some(Domain::Seq { len, .. })) = (ds.first(), ds.get(1)) else { return Verdict::Unproven };
                let last = i.add(&ISet::one(*lanes as i128 - 1));
                let (Some((_, ihi)), Some((llo, lhi))) = (last.bounds(), len.bounds()) else { return Verdict::Unproven };
                let ilo = i.bounds().map(|(l, _)| l).unwrap_or(0);
                if ilo >= 1 && ihi <= llo {
                    Verdict::Proven { how: "every lane is inside" }
                } else if ilo > lhi || (*lanes as i128) > lhi {
                    Verdict::Fails { counterexample: format!("every time: {} lanes from position {} and there are {} element(s)", lanes, ilo, lhi) }
                } else {
                    Verdict::Fails { counterexample: format!("the {} lanes can reach position {} when there are {} element(s)", lanes, ihi, llo) }
                }
            }
            // A lend into a task is only ever into a structured one (the
            // checker refuses a lend into `concur.detached`), and a block
            // waits for the tasks it started before it gives back anything
            // it owns, on every way out — the end, `return`, `break`, `fail`
            // (`mir::build::pop_scope` and `unwind_to` join first, then
            // drop). So what a task borrowed is still there when it ends:
            // "structured tasks make that provable" (`concurrency.md`).
            SiteKind::TaskLendOutlives => Verdict::Proven { how: "structured: the block waits for its tasks before it lets go of anything" },
            // The ones layer 3 and the verification session take.
            SiteKind::KeyHashAgrees | SiteKind::LendOutlives | SiteKind::ChannelReachable | SiteKind::Deadlock => Verdict::Unproven,
        }
    }

    fn fit(&self, s: &ISet, lo: i128, hi: i128, ty: &Ty) -> Verdict {
        if s.is_empty() {
            return Verdict::Unproven;
        }
        if s.within(lo, hi) {
            return Verdict::Proven { how: "layer 2" };
        }
        if s.misses(lo, hi) {
            let (slo, shi) = s.bounds().unwrap();
            return Verdict::Fails { counterexample: format!("every time: the answer is {} and a `{}` holds {} to {}", if slo == shi { slo.to_string() } else { format!("{} to {}", slo, shi) }, self.spell(ty), lo, hi) };
        }
        let (slo, shi) = s.bounds().unwrap();
        let bad = if shi > hi { shi } else { slo };
        Verdict::Fails { counterexample: format!("the answer can be {}, and a `{}` holds {} to {}", bad, self.spell(ty), lo, hi) }
    }

    fn fits_type(&self, d: &Domain, to: &Ty) -> Verdict {
        match (d, Domain::of_ty(to, &self.prog().items)) {
            (Domain::Ints(s), Domain::Ints(limit)) => {
                let Some((lo, hi)) = limit.bounds() else { return Verdict::Unproven };
                self.fit(s, lo, hi, to)
            }
            (Domain::Duration(s), Domain::Duration(limit)) => {
                let Some((lo, hi)) = limit.bounds() else { return Verdict::Unproven };
                let Some((a, b)) = s.bounds() else { return Verdict::Unproven };
                if a >= lo && b <= hi {
                    Verdict::Proven { how: "layer 2" }
                } else if b < lo {
                    Verdict::Fails { counterexample: format!("every time: {} nanoseconds, and a duration is a length of time — it is never negative", describe_ints(s)) }
                } else {
                    Verdict::Fails { counterexample: "it can go below zero, and a duration is never negative".to_string() }
                }
            }
            (Domain::Bins { lo, hi }, Domain::Bins { lo: llo, hi: lhi }) => {
                let (Some(a), Some(b)) = (lo, hi) else { return Verdict::Unproven };
                let (Some(x), Some(y)) = (llo, lhi) else { return Verdict::Proven { how: "layer 2" } };
                // A plain bin16 or bin32 holds what rounds to a finite value
                // of it: up to, not including, the largest plus half a step
                // (the tie rounds to even, which is past it). 65519 is a
                // bin16's 65504; 65520 is infinity.
                if to.bound().is_none() {
                    if let Some(limit) = bin_round_limit(to) {
                        if *a > -limit && *b < limit {
                            return Verdict::Proven { how: "layer 2" };
                        }
                        let v = if *b >= limit { *b } else { *a };
                        return if *b <= -limit || *a >= limit {
                            Verdict::Fails { counterexample: format!("every time: the value is {} and a `{}` holds {} to {}, what rounds to it included", show_bin(*a, *b), self.spell(to), x, y) }
                        } else {
                            Verdict::Fails { counterexample: format!("the value can be {}, which rounds past what a `{}` holds ({} to {})", v, self.spell(to), x, y) }
                        };
                    }
                }
                if *a >= x && *b <= y {
                    Verdict::Proven { how: "layer 2" }
                } else if *b < x || *a > y {
                    Verdict::Fails { counterexample: format!("every time: the value is {} and a `{}` holds {} to {}", show_bin(*a, *b), self.spell(to), x, y) }
                } else {
                    Verdict::Fails { counterexample: format!("the value can be {}, and a `{}` holds {} to {}", if *b > y { *b } else { *a }, self.spell(to), x, y) }
                }
            }
            (Domain::Seq { len, .. }, Domain::Seq { len: limit, .. }) => {
                let (Some((a, b)), Some((lo, hi))) = (len.bounds(), limit.bounds()) else { return Verdict::Unproven };
                if a >= lo && b <= hi {
                    Verdict::Proven { how: "layer 2" }
                } else if b < lo || a > hi {
                    Verdict::Fails { counterexample: format!("every time: {} element(s), and its type holds {} to {}", describe_ints(len), lo, hi) }
                } else {
                    Verdict::Fails { counterexample: format!("it can hold {} element(s), and its type holds {} to {}", b, lo, hi) }
                }
            }
            (Domain::Text { len, .. }, Domain::Text { len: limit, .. }) => {
                for u in [crate::types::Unit::Byte, crate::types::Unit::Point, crate::types::Unit::Grapheme] {
                    let (have, want) = (len.get(u), limit.get(u));
                    let (Some((a, b)), Some((lo, hi))) = (have.bounds(), want.bounds()) else { continue };
                    if lo == 0 && hi >= i64::MAX as i128 {
                        continue;
                    }
                    if a >= lo && b <= hi {
                        return Verdict::Proven { how: "layer 2" };
                    }
                    if b < lo || a > hi {
                        return Verdict::Fails { counterexample: format!("every time: {} {}, and its type holds {} to {}", describe_ints(have), unit_word(u), lo, hi) };
                    }
                    return Verdict::Fails { counterexample: format!("it can be {} {}, and its type holds {} to {}", b, unit_word(u), lo, hi) };
                }
                Verdict::Unproven
            }
            (Domain::Map { len, .. }, Domain::Map { len: limit, .. }) => {
                let (Some((a, b)), Some((lo, hi))) = (len.bounds(), limit.bounds()) else { return Verdict::Unproven };
                if a >= lo && b <= hi {
                    Verdict::Proven { how: "layer 2" }
                } else if b < lo || a > hi {
                    Verdict::Fails { counterexample: format!("every time: {} entries, and its type holds {} to {}", describe_ints(len), lo, hi) }
                } else {
                    Verdict::Fails { counterexample: format!("it can hold {} entries, and its type holds {} to {}", b, lo, hi) }
                }
            }
            _ => Verdict::Unproven,
        }
    }

    fn spell(&self, t: &Ty) -> String {
        crate::types::TySpelling { ty: t, structs: &self.prog().items.structs, enums: &self.prog().items.enums }.to_string()
    }
}

impl Domain {
    fn clone_with(self, _f: usize) -> Domain {
        self
    }
}

/// The whole value, with one part replaced by what was written there.
fn put(whole: &Domain, proj: &[Proj], d: Domain) -> Domain {
    let Some((first, rest)) = proj.split_first() else { return d };
    match (first, whole) {
        (Proj::Field(i), Domain::Struct(fields)) => {
            let mut out = fields.clone();
            if *i < out.len() {
                let inner = put(&out[*i], rest, d);
                out[*i] = inner;
            }
            Domain::Struct(out)
        }
        // Which element was written is not tracked, so the element domain
        // becomes "what was there, or what was just put there". The length
        // is untouched: writing into a list never changes how long it is.
        (Proj::Index(_), Domain::Seq { len, elem }) => {
            let inner = put(elem, rest, d);
            Domain::Seq { len: len.clone(), elem: Box::new(elem.union(&inner)) }
        }
        _ => whole.clone(),
    }
}

fn arg_operand(a: &Arg) -> Operand {
    match a {
        Arg::Value(o) => o.clone(),
        Arg::Lend { place, .. } | Arg::Copy(place) | Arg::Spread { place, .. } => Operand::Read(place.clone()),
        Arg::Give(place) => Operand::Move(place.clone()),
    }
}

/// Instructions worth replacing by their answer. A `lend` is not one: it
/// is an address, and the address is not what was computed.
fn foldable(rv: &Rvalue) -> bool {
    matches!(rv, Rvalue::Binary { .. } | Rvalue::Unary { .. } | Rvalue::Convert { .. })
}

/// The one value a domain holds, when it holds exactly one. Bins are left
/// alone: rounding at `bin16` and `bin128` is not modelled yet, and a fold
/// is where that would first tell a lie.
fn known_const(d: &Domain) -> Option<Const> {
    match d {
        Domain::Ints(s) => s.single().map(Const::Int),
        Domain::Bools(true, false) => Some(Const::Bool(true)),
        Domain::Bools(false, true) => Some(Const::Bool(false)),
        Domain::Text { values: Some(v), .. } if v.len() == 1 => Some(Const::Str(v[0].clone())),
        Domain::Duration(s) => s.single().map(Const::Duration),
        _ => None,
    }
}

fn cmp_of(op: &BinOp) -> Option<Cmp> {
    match op {
        BinOp::Lt => Some(Cmp::Lt),
        BinOp::Le => Some(Cmp::Le),
        BinOp::Gt => Some(Cmp::Gt),
        BinOp::Ge => Some(Cmp::Ge),
        BinOp::Eq => Some(Cmp::Eq),
        BinOp::Ne => Some(Cmp::Ne),
        _ => None,
    }
}

fn int_limits(ty: &Ty, items: &crate::resolve::Items) -> Option<(i128, i128)> {
    match Domain::of_ty(ty, items) {
        Domain::Ints(s) => s.bounds(),
        _ => None,
    }
}

fn show_bin(a: f64, b: f64) -> String {
    if a == b {
        format!("{}", a)
    } else {
        format!("{} to {}", a, b)
    }
}

fn describe_ints(s: &ISet) -> String {
    match s.bounds() {
        Some((a, b)) if a == b => a.to_string(),
        Some((a, b)) => format!("{} to {}", a, b),
        None => "no value".into(),
    }
}

fn unit_word(u: crate::types::Unit) -> &'static str {
    match u {
        crate::types::Unit::Byte => "bytes",
        crate::types::Unit::Point => "code points",
        _ => "graphemes",
    }
}

fn classes_of(s: &str) -> Vec<Class> {
    let mut v = Vec::new();
    if !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()) {
        v.push(Class::AsciiDigits);
    }
    if s.is_ascii() {
        v.push(Class::Ascii);
    }
    if !s.contains('\n') {
        v.push(Class::NoNewline);
    }
    if !s.is_empty() {
        v.push(Class::NonEmpty);
    }
    v
}

/// Two verdicts for one site, from two paths: the program has to survive
/// both, so the worse one wins.
fn worse(a: Verdict, b: Verdict) -> Verdict {
    use Verdict::*;
    match (&a, &b) {
        (Trusted, _) | (_, Trusted) => Trusted,
        (Fails { .. }, _) => a,
        (_, Fails { .. }) => b,
        (Unproven, _) | (_, Unproven) => Unproven,
        _ => a,
    }
}

fn note_of(v: &Verdict) -> String {
    match v {
        Verdict::Proven { how } => (*how).to_string(),
        Verdict::Fails { counterexample } => counterexample.clone(),
        Verdict::Trusted => "nocheck".into(),
        Verdict::Unproven => "not proven".into(),
    }
}

/// The edges that go round again: an edge to a block that is still on the
/// path taken to reach it. Found once per function, by walking the graph.
pub fn back_edges(f: &Function) -> std::collections::HashSet<(usize, usize)> {
    let mut back = std::collections::HashSet::new();
    let mut colour = vec![0u8; f.blocks.len()]; // 0 unseen, 1 on the path, 2 done
    let mut stack: Vec<(usize, usize)> = vec![(0, 0)];
    colour[0] = 1;
    while let Some((b, next)) = stack.pop() {
        let succs = crate::mir::verify::successors(&f.blocks[b].term);
        if next >= succs.len() {
            colour[b] = 2;
            continue;
        }
        stack.push((b, next + 1));
        let t = succs[next].0 as usize;
        if t >= f.blocks.len() {
            continue;
        }
        match colour[t] {
            1 => {
                back.insert((b, t));
            }
            0 => {
                colour[t] = 1;
                stack.push((t, 0));
            }
            _ => {}
        }
    }
    back
}

/// The blocks of the loop that back edge `b -> h` closes: the head, and
/// everything that reaches `b` without passing through the head again.
pub fn natural_loop(f: &Function, h: usize, b: usize) -> Vec<usize> {
    let mut preds: Vec<Vec<usize>> = vec![Vec::new(); f.blocks.len()];
    for (i, blk) in f.blocks.iter().enumerate() {
        for t in crate::mir::verify::successors(&blk.term) {
            if (t.0 as usize) < preds.len() {
                preds[t.0 as usize].push(i);
            }
        }
    }
    let mut body = vec![h];
    let mut stack = vec![b];
    if b != h {
        body.push(b);
    }
    while let Some(x) = stack.pop() {
        for p in &preds[x] {
            if !body.contains(p) {
                body.push(*p);
                stack.push(*p);
            }
        }
    }
    body
}

fn same(a: &Env, b: &Env) -> bool {
    a.len() == b.len() && a.iter().all(|(k, v)| b.get(k).map(|w| w == v).unwrap_or(false))
}

/// A loop that counts: `c` goes up by `step` exactly once a trip toward a
/// bound that does not change inside the loop, and each accumulator's only
/// write adds something that is not itself.
#[derive(Clone, Debug)]
struct Counted {
    counter: LocalId,
    step: i128,
    /// The head's test: `c <= bound` (or `<` when `strict`), and where the
    /// bound's value is computed in the head, if it is.
    bound: Operand,
    strict: bool,
    /// Accumulators: the local, and (block, statement) of its `t = x + e`.
    accs: Vec<(LocalId, (usize, usize))>,
    /// Lists appended to exactly once a trip, and changed no other way: a
    /// length is an accumulator of ones.
    grows: Vec<LocalId>,
}

/// `inner` is every block of a loop nested inside this one: a write there
/// can happen many times a trip, so nothing written there is counted.
fn counted_loop(f: &crate::mir::Function, head: usize, latch: usize, body: &[usize], inner: &std::collections::HashSet<usize>) -> Option<Counted> {
    let inside = |b: usize| body.contains(&b);
    // The head tests `c <= bound` and goes into the loop when it holds.
    let hb = &f.blocks[head];
    let Terminator::If { cond: Operand::Read(cv), then, else_ } = &hb.term else { return None };
    if !cv.proj.is_empty() || !inside(then.0 as usize) || inside(else_.0 as usize) {
        return None;
    }
    let mut test = None;
    for (si, s) in hb.stmts.iter().enumerate() {
        if let StmtKind::Assign { place, rvalue: Rvalue::Binary { op, lhs: Operand::Read(c), rhs } } = &s.kind {
            if place.local == cv.local && place.proj.is_empty() && c.proj.is_empty() && matches!(op, BinOp::Le | BinOp::Lt) {
                test = Some((c.local, rhs.clone(), *op == BinOp::Lt, si));
            }
        }
    }
    let (counter, bound, strict, _) = test?;
    // Where the bound comes from, if the head computes it, and that nothing
    // in the body can change it.
    let mut bound_src: Option<LocalId> = None;
    if let Operand::Read(bp) = &bound {
        if !bp.proj.is_empty() {
            return None;
        }
        bound_src = Some(bp.local);
        for s in &hb.stmts {
            match &s.kind {
                StmtKind::Op { dest: Some(d), op: Op::ListLength | Op::ArrayLength, args } if d.local == bp.local => {
                    bound_src = args.first().and_then(|a| a.place()).filter(|p| p.proj.is_empty()).map(|p| p.local);
                    bound_src?;
                }
                StmtKind::Assign { place, .. } if place.local == bp.local => return None,
                _ => {}
            }
        }
    }
    // Does every way round the loop pass `b`? From the head, avoiding `b`,
    // the latch cannot be reached.
    let on_every_trip = |b: usize| -> bool {
        if b == head || b == latch {
            return true;
        }
        let mut seen = std::collections::HashSet::new();
        let mut stack = vec![head];
        while let Some(x) = stack.pop() {
            if x == b || !seen.insert(x) {
                continue;
            }
            if x == latch {
                return false;
            }
            for t in crate::mir::verify::successors(&f.blocks[x].term) {
                let t = t.0 as usize;
                if inside(t) && t != head {
                    stack.push(t);
                }
            }
        }
        true
    };
    let mut counter_block: Option<usize> = None;
    let mut appends: HashMap<LocalId, usize> = HashMap::new();
    let mut plus: HashMap<LocalId, (LocalId, Operand, (usize, usize))> = HashMap::new(); // temp -> (x, e, where)
    let mut writes: HashMap<LocalId, Vec<Option<(LocalId, Operand, (usize, usize))>>> = HashMap::new();
    for &b in body {
        let blk = &f.blocks[b];
        for (si, s) in blk.stmts.iter().enumerate() {
            match &s.kind {
                StmtKind::Assign { place, rvalue } if place.proj.is_empty() => {
                    if let Rvalue::Binary { op: BinOp::Add, lhs, rhs } = rvalue {
                        let (x, e) = match (lhs, rhs) {
                            (Operand::Read(x), e) if x.proj.is_empty() && e.place().map_or(true, |p| p.local != x.local) => (Some(x.local), e.clone()),
                            (e, Operand::Read(x)) if x.proj.is_empty() && e.place().map_or(true, |p| p.local != x.local) => (Some(x.local), e.clone()),
                            _ => (None, Operand::Const(Const::Nothing)),
                        };
                        if let Some(x) = x {
                            plus.insert(place.local, (x, e, (b, si)));
                        }
                    }
                    // Inside a loop nested in this one, a write can happen
                    // many times a trip: never counted.
                    let w = match rvalue {
                        Rvalue::Use(Operand::Read(t) | Operand::Move(t)) if t.proj.is_empty() && !inner.contains(&b) => plus.get(&t.local).filter(|(x, _, _)| *x == place.local).cloned(),
                        _ => None,
                    };
                    if w.is_some() && place.local == counter {
                        counter_block = Some(b);
                    }
                    writes.entry(place.local).or_default().push(w);
                }
                StmtKind::Op { op: Op::ListAppend, args, dest: None } if args.first().and_then(|a| a.place()).map_or(false, |p| p.proj.is_empty()) => {
                    let l = args[0].place().unwrap().local;
                    // Two for one inside an inner loop: never counted.
                    *appends.entry(l).or_default() += if inner.contains(&b) { 2 } else { 1 };
                    writes.entry(l).or_default().push(None);
                }
                _ => {
                    let mut w = std::collections::HashSet::new();
                    let one = crate::mir::BasicBlock { stmts: vec![s.clone()], term: Terminator::Unreachable, ..blk.clone() };
                    crate::precog::smt::written_in_pub(&one, &mut w);
                    for l in w {
                        writes.entry(l).or_default().push(None);
                    }
                }
            }
        }
        let mut w = std::collections::HashSet::new();
        let term_only = crate::mir::BasicBlock { stmts: Vec::new(), ..blk.clone() };
        crate::precog::smt::written_in_pub(&term_only, &mut w);
        for l in w {
            writes.entry(l).or_default().push(None);
        }
    }
    // The counter: one write, `c = c + k` with k a positive constant, on
    // every way round — so the trips are bounded by how far it can go.
    let cw = writes.get(&counter)?;
    let [Some((_, Operand::Const(Const::Int(k)), _))] = cw.as_slice() else { return None };
    if *k <= 0 || !counter_block.map_or(false, on_every_trip) {
        return None;
    }
    if let Some(src) = bound_src {
        if writes.contains_key(&src) {
            return None;
        }
    }
    let accs = writes
        .iter()
        .filter(|(l, _)| **l != counter)
        .filter_map(|(l, w)| match w.as_slice() {
            [Some((_, _, at))] => Some((*l, *at)),
            _ => None,
        })
        .collect();
    // One append a trip and no other change: the list's only write is it.
    let grows = appends.iter().filter(|(l, n)| **n == 1 && writes.get(*l).map_or(false, |w| w.len() == 1)).map(|(l, _)| *l).collect();
    Some(Counted { counter, step: *k, bound, strict, accs, grows })
}

/// The constants a function mentions, each with its neighbours, and below a
/// `mod` by one its largest remainder: where a widened bound may stop.
fn thresholds(f: &crate::mir::Function) -> Vec<i128> {
    let mut out: Vec<i128> = Vec::new();
    let add = |o: &Operand, out: &mut Vec<i128>| {
        if let Operand::Const(Const::Int(v)) = o {
            out.extend([v.saturating_sub(1), *v, v.saturating_add(1)]);
        }
    };
    for blk in &f.blocks {
        for s in &blk.stmts {
            match &s.kind {
                StmtKind::Assign { rvalue, .. } => match rvalue {
                    Rvalue::Use(o) | Rvalue::Unary { operand: o, .. } | Rvalue::Convert { operand: o, .. } => add(o, &mut out),
                    Rvalue::Binary { op, lhs, rhs } => {
                        add(lhs, &mut out);
                        add(rhs, &mut out);
                        if let (BinOp::Mod, Operand::Const(Const::Int(d))) = (op, rhs) {
                            out.extend([d.saturating_sub(1).abs(), -(d.saturating_sub(1).abs())]);
                        }
                    }
                    Rvalue::MakeList(os) => os.iter().for_each(|o| add(o, &mut out)),
                    _ => {}
                },
                StmtKind::Op { args, .. } => args.iter().for_each(|o| add(o, &mut out)),
                _ => {}
            }
        }
    }
    out.push(0);
    out.sort_unstable();
    out.dedup();
    out
}

fn join(a: &Env, b: &Env, widening: bool, items: &crate::resolve::Items, ty: impl Fn(LocalId) -> Ty) -> Env {
    join_to(a, b, widening, items, ty, &[])
}

fn join_to(a: &Env, b: &Env, widening: bool, items: &crate::resolve::Items, ty: impl Fn(LocalId) -> Ty, steps: &[i128]) -> Env {
    let mut out = a.clone();
    for (k, v) in b {
        let merged = match a.get(k) {
            None => v.clone(),
            Some(old) => {
                if widening {
                    old.widen_to(v, &ty(*k), items, steps)
                } else {
                    old.union(v)
                }
            }
        };
        out.insert(*k, merged);
    }
    out
}

/// What the build refuses. A site that fails on **every** path that
/// reaches it would fail every time the program ran, so it never builds —
/// whatever its tier (`precog.md`, layer 1: "a check that would fail on
/// known values is a compile error at the line"). A site that fails only
/// sometimes is the tiers' business: a runtime check in a working build,
/// a refusal in a final one, and a refusal now when it is a `check`.
pub fn refusals(a: &Analysis, p: &Program, sources: &crate::source::SourceMap) -> Vec<String> {
    refusals_opts(a, p, sources, false)
}

/// The same, for the final build when `final_build`: there the default tier
/// behaves as `check` (`core-features.md`, owner 2026-09-20), so a site
/// Precog could not prove is a compile error, and the product carries no
/// run-time checks — every site proven, `check`-confirmed, or `nocheck`.
pub fn refusals_opts(a: &Analysis, p: &Program, sources: &crate::source::SourceMap, final_build: bool) -> Vec<String> {
    let mut out = Vec::new();
    for r in &a.sites {
        let f = &p.functions[r.function];
        let site = f.site(r.site);
        let checked = site.tier == Some(Tier::Check) || (final_build && site.tier.is_none());
        // `check` is "I built a guardrail, tell me whether it is enough", so
        // *not knowing* is an answer it has to hear (`core-features.md`:
        // "not proven → compile error naming the counterexample or the
        // missing fact"). Saying nothing would make `check` mean nothing.
        if checked && matches!(r.verdict, Verdict::Unproven) {
            let spell = |t: &Ty| crate::types::TySpelling { ty: t, structs: &p.items.structs, enums: &p.items.enums }.to_string();
            let mut d = crate::diag::Diagnostic::new(crate::diag::Kind::Check, site.sentence(&spell))
                .at(f.span(site.span))
                .why("`check` asks whether the guardrail is enough, and this one could not be shown to be")
                .why("no counterexample either: the guardrail may well be enough, and nothing here could prove it")
                .fix(None, "narrow what reaches here — a bound on the type, or an `if` in front of it");
            if !f.name.is_empty() && f.name != "MAIN" {
                d = d.place(format!("in `{}`", f.name));
            }
            out.push(d.render(sources));
            continue;
        }
        let Verdict::Fails { counterexample } = &r.verdict else { continue };
        // "A site where no value is legal is a compile error only when it
        // is reached on every run" (owner, 2026-09-23). One that may never
        // run stays a check, with a note (see `notes`).
        let certain = counterexample.starts_with("every") && a.always_reached.contains(&(r.function, r.site.0));
        if !certain && !checked {
            continue;
        }
        let spell = |t: &Ty| crate::types::TySpelling { ty: t, structs: &p.items.structs, enums: &p.items.enums }.to_string();
        let sentence = site.sentence(&spell);
        let mut d = crate::diag::Diagnostic::new(if certain { crate::diag::Kind::Error } else { crate::diag::Kind::Check }, sentence)
            .at(f.span(site.span))
            .when(counterexample.trim_start_matches("every time: ").to_string());
        if !f.name.is_empty() && f.name != "MAIN" {
            d = d.place(format!("in `{}`", f.name));
        }
        if certain {
            d = d.why("this is what the program does every time it reaches here, so it cannot be right");
        } else {
            d = d.why("`check` asks whether the guardrail is enough, and this value gets past it");
        }
        out.push(d.render(sources));
    }
    out
}

/// What the build says without refusing: the `note:` kind of
/// `diagnostics.md`. Today that is the `nocheck` site Precog's ordinary
/// work showed fails every run (owner, 2026-09-22).
pub fn notes(a: &Analysis, p: &Program, sources: &crate::source::SourceMap) -> Vec<String> {
    let mut out = Vec::new();
    // A line that fails whenever it runs, but may never run: a check, and
    // the writer is told (owner, 2026-09-23).
    for r in &a.sites {
        let Verdict::Fails { counterexample } = &r.verdict else { continue };
        if !counterexample.starts_with("every") || a.always_reached.contains(&(r.function, r.site.0)) {
            continue;
        }
        let f = &p.functions[r.function];
        let site = f.site(r.site);
        if site.tier == Some(Tier::Check) {
            continue;
        }
        let mut d = crate::diag::Diagnostic::note("this line fails whenever it runs")
            .at(f.span(site.span))
            .when(counterexample.trim_start_matches("every time: ").to_string())
            .why("it may never run, so it is a check here rather than an error; the final build refuses it");
        if !f.name.is_empty() && f.name != "MAIN" {
            d = d.place(format!("in `{}`", f.name));
        }
        out.push(d.render(sources));
    }
    for (f_idx, site_id, what) in &a.certain_under_nocheck {
        let f = &p.functions[*f_idx];
        let site = f.site(*site_id);
        let d = crate::diag::Diagnostic::note("this `nocheck` covers something that fails every time it runs")
            .at(f.span(site.span))
            .when(what.clone())
            .why("`nocheck` removes the check on your word, and the build keeps it removed")
            .why("Precog was not looking; this is what it saw while working out everything else")
            .fix(None, "remove the `nocheck`, or fix what it is covering");
        out.push(d.render(sources));
    }
    out
}

fn describe_disagreement(p: &Program, r: &SiteResult, each: &[(&'static str, smt::Answer)], query: &str) -> String {
    let f = &p.functions[r.function];
    let site = f.site(r.site);
    let spell = |t: &Ty| crate::types::TySpelling { ty: t, structs: &p.items.structs, enums: &p.items.enums }.to_string();
    let dir = std::env::temp_dir().join("cca-smt");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(format!("disagreement-{}-{}.smt2", r.function, r.site.0));
    let _ = std::fs::write(&path, query);
    let said = each.iter().map(|(n, a)| format!("{} said {}", n, match a { smt::Answer::Unsat => "it cannot happen", smt::Answer::Sat(_) => "it can", smt::Answer::Unknown => "it could not tell" })).collect::<Vec<_>>().join(", and ");
    format!(
        "the solvers disagree about `{}` in `{}`: {}.\n  This is a bug in one of them, or in the question CCA asked them — both of them received the same question — and never in the program.\n  The question is written out in {} ; please open an issue at Cyborg's tracker with it attached.\n  The build carries on with the safe answer: the site keeps its check.",
        site.sentence(&spell),
        f.name,
        said,
        path.display()
    )
}


/// Which sites are reached on every run of the program.
///
/// Within a function, a block is reached on every run of it when no path
/// layer 2 found possible leads from the entry to a way out without passing
/// through it. A function runs on every run of the program when MAIN does
/// — MAIN always does — or when it is called, or started as a structured
/// task, from a block reached on every run of a function that does. A
/// detached task may not have run before the program ends, so it never
/// counts. A call to C's `exit` or `abort` is a way out.
fn reached_every_run(p: &Program, edges: &std::collections::HashSet<(usize, usize, usize)>) -> std::collections::HashSet<(usize, u32)> {
    use std::collections::HashSet;
    let ends = |f: &Function, b: usize| -> bool {
        match &f.blocks[b].term {
            Terminator::Return | Terminator::Fail { .. } | Terminator::Trap(_) | Terminator::Unreachable => true,
            Terminator::Call { callee: Callee::Extern(id), .. } => {
                matches!(p.items.externs[id.0 as usize].name.as_str(), "exit" | "_exit" | "_Exit" | "abort" | "quick_exit")
            }
            _ => false,
        }
    };
    let mut always_blocks: Vec<HashSet<usize>> = vec![HashSet::new(); p.functions.len()];
    for (fi, f) in p.functions.iter().enumerate() {
        let mut succ: Vec<Vec<usize>> = vec![Vec::new(); f.blocks.len()];
        for &(g, from, to) in edges {
            if g == fi && from < succ.len() {
                succ[from].push(to);
            }
        }
        // Reached at all.
        let mut seen = vec![false; f.blocks.len()];
        let mut stack = vec![0usize];
        while let Some(x) = stack.pop() {
            if x >= seen.len() || seen[x] {
                continue;
            }
            seen[x] = true;
            stack.extend(succ[x].iter().copied());
        }
        for b in 0..f.blocks.len() {
            if !seen[b] {
                continue;
            }
            // Can the function end without passing b? A block with nowhere
            // left to go counts as an end: nothing says the run continues.
            let mut avoid = vec![false; f.blocks.len()];
            let mut stack = if b == 0 { Vec::new() } else { vec![0usize] };
            let mut escapes = false;
            while let Some(x) = stack.pop() {
                if x == b || avoid[x] {
                    continue;
                }
                avoid[x] = true;
                if ends(f, x) || succ[x].is_empty() {
                    escapes = true;
                    break;
                }
                stack.extend(succ[x].iter().copied());
            }
            if !escapes {
                always_blocks[fi].insert(b);
            }
        }
    }
    // Which functions run on every run.
    let mut runs: HashSet<usize> = p.functions.iter().enumerate().filter(|(_, f)| f.is_main).map(|(i, _)| i).collect();
    let mut work: Vec<usize> = runs.iter().copied().collect();
    while let Some(fi) = work.pop() {
        let f = &p.functions[fi];
        for &b in &always_blocks[fi] {
            let mut starts: Vec<usize> = Vec::new();
            if let Terminator::Call { callee: Callee::Fn(g), .. } = &f.blocks[b].term {
                starts.push(g.0 as usize);
            }
            for s in &f.blocks[b].stmts {
                if let StmtKind::Task { detached: false, body, .. } = &s.kind {
                    starts.push(body.0 as usize);
                }
            }
            for g in starts {
                if g < p.functions.len() && runs.insert(g) {
                    work.push(g);
                }
            }
        }
    }
    let mut out = HashSet::new();
    for &fi in &runs {
        for &b in &always_blocks[fi] {
            for s in &p.functions[fi].blocks[b].stmts {
                if let StmtKind::Site(id) = s.kind {
                    out.insert((fi, id.0));
                }
            }
        }
    }
    out
}

/// Bin `+ - x /` over two intervals at a width of `bits`, or `None` when the
/// answer is not a finite interval Precog can stand behind.
pub fn bin_interval(op: &BinOp, a: (f64, f64), b: (f64, f64), bits: u16) -> Option<Domain> {
    let corners = [(a.0, b.0), (a.0, b.1), (a.1, b.0), (a.1, b.1)];
    let f = |x: f64, y: f64| -> f64 {
        match op {
            BinOp::Add => x + y,
            BinOp::Sub => x - y,
            BinOp::Mul => x * y,
            BinOp::Div => x / y,
            _ => f64::NAN,
        }
    };
    if matches!(op, BinOp::Div) && b.0 <= 0.0 && b.1 >= 0.0 {
        return None;
    }
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for (x, y) in corners {
        let v = f(x, y);
        if !v.is_finite() {
            return None;
        }
        lo = lo.min(v);
        hi = hi.max(v);
    }
    // One unit in the last place at the width, and the smallest step the
    // width has near 0, so a subnormal answer is covered too.
    let (mantissa, tiny) = match bits {
        16 => (10, 2f64.powi(-24)),
        32 => (23, 2f64.powi(-149)),
        64 => (52, f64::from_bits(1)),
        _ => return None,
    };
    let widen = |v: f64| v.abs() * 2f64.powi(-mantissa) + tiny;
    let (lo, hi) = (lo - widen(lo), hi + widen(hi));
    let max = crate::mir::domain::bin_max(crate::types::BinW::Fixed(bits))?;
    if lo < -max || hi > max {
        return None;
    }
    Some(Domain::Bins { lo: Some(lo), hi: Some(hi) })
}

/// `x` rounded to a bin of `bits` (16 or 32), then one step further out
/// (`up` toward +infinity): sound for an end of an interval whether `x` was
/// the exact value or had already been rounded to 64 bits on the way (a
/// literal is read at 64 bits first, and rounding twice can land one step
/// off). An answer past the width's finite range is left to the range check.
fn round_out(x: f64, bits: u16, up: bool) -> f64 {
    if !x.is_finite() {
        return x;
    }
    if bits == 32 {
        // Past the largest at 64 bits, but a literal's digits can still be
        // below the limit (its own site decides, from them): what landed is
        // then the largest. Keeping the 64-bit value folded `'a' == max`
        // to false.
        let mut r = x as f32;
        if !r.is_finite() {
            r = f32::MAX.copysign(x as f32);
        }
        let b = r.to_bits();
        // One step toward the side asked for, over zero too.
        let step = |r: f32, toward_up: bool| -> f32 {
            if r == 0.0 {
                return if toward_up { f32::from_bits(1) } else { -f32::from_bits(1) };
            }
            let pos = r > 0.0;
            if pos == toward_up { f32::from_bits(b + 1) } else { f32::from_bits(b - 1) }
        };
        return step(r, up) as f64;
    }
    // bin16: a unit in its last place, relative, and its smallest step.
    let w = x.abs() * 2f64.powi(-10) + 2f64.powi(-24);
    if up { x + w } else { x - w }
}

/// Where rounding to a plain bin16 or bin32 goes past its largest value:
/// the largest plus half a step. None for bin64, whose every finite value
/// is already a bin64.
fn bin_round_limit(to: &Ty) -> Option<f64> {
    match to.base() {
        Ty::Bin(crate::types::BinW::Fixed(16)) => Some(65520.0),
        Ty::Bin(crate::types::BinW::Fixed(32)) => Some(2f64.powi(128) - 2f64.powi(103)),
        _ => None,
    }
}

/// Whether a bin literal, as written, rounds to a finite plain bin16 or
/// bin32: its magnitude below the rounding limit (65520, or 2^128 - 2^103;
/// the limit itself ties, and the tie goes to infinity). Both limits are
/// whole, so the integer digits decide: below the limit's, it fits; the
/// limit's own or more, it does not. None when it is not that kind of
/// question.
fn literal_fits_bin(text: &str, to: &Ty) -> Option<bool> {
    let limit = match to.base() {
        Ty::Bin(crate::types::BinW::Fixed(16)) => "65520",
        Ty::Bin(crate::types::BinW::Fixed(32)) => "340282356779733661637539395458142568448",
        _ => return None,
    };
    let t: String = text.chars().filter(|c| *c != '_').collect();
    let t = t.trim_start_matches(['-', '+']);
    let whole = t.split('.').next().unwrap_or("");
    if whole.is_empty() || !whole.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let whole = whole.trim_start_matches('0');
    Some(whole.len() < limit.len() || (whole.len() == limit.len() && whole < limit))
}

/// The value a site's check asks about, from its operands' domains: an
/// operation's answer, a narrowed value, a divisor, a position, a shift
/// count. `None` for a kind without one number to show.
fn claimed_value(kind: &SiteKind, ds: &[Domain]) -> Option<Domain> {
    match kind {
        SiteKind::Overflow(op) => {
            let (Some(Domain::Ints(a)), b) = (ds.first(), ds.get(1)) else { return None };
            let r = match (op, b) {
                (BinOp::Add, Some(Domain::Ints(y))) => a.add(y),
                (BinOp::Sub, Some(Domain::Ints(y))) => a.sub(y),
                (BinOp::Mul, Some(Domain::Ints(y))) => a.mul(y),
                (BinOp::Pow, Some(Domain::Ints(y))) => a.pow(y),
                (BinOp::Div, Some(Domain::Ints(y))) => a.div_floor(y),
                (BinOp::Sub, None) => a.neg(),
                _ => return None,
            };
            Some(Domain::Ints(r))
        }
        SiteKind::Narrow { .. } | SiteKind::LengthBound { .. } | SiteKind::DivZero | SiteKind::ModZero | SiteKind::Bounds | SiteKind::NegativeExponent => ds.first().cloned(),
        SiteKind::ShiftWidth | SiteKind::ShiftCount => ds.get(1).cloned(),
        _ => None,
    }
}

// ---------------------------------------------------------------- templates

/// The template "a sum is bounded by trips times the largest element"
/// (`precog.md`), for `A / M` where `M` is a list's length and `A` a sum of
/// that list's elements taken in a counted walk over it. Every condition is
/// checked on the graph, conservatively; any doubt and the answer is `None`
/// (nothing proven, nothing assumed):
///
/// - `M` is set once, to `ListLength[P]`, and nothing in the function writes
///   `P`'s root local or lends it mutably — the list is the same at the walk
///   and at the division;
/// - `A` is set to 0, and otherwise only by `A = T` with `T = A + X` once;
/// - `X` is set once, from `Q[pos I]`, with `Q` a lend of that same `P`
///   (through copies of the borrow);
/// - `I` is set to 1 once, outside every cycle, and otherwise only by
///   `I = I + 1`; every cycle through the add passes through that step;
/// - the read `X = Q[pos I]`, then `T = A + X`, then `A = T` are in one
///   block, in that order: every add uses a fresh read at the current `I`.
///
/// Then each add is at a different `I`, each `I` a position the read
/// checked, so at most `len(P)` of them: `A` is within `len · min(lo, 0)` and
/// `len · max(hi, 0)` for the elements' type range `[lo, hi]`, and `A / len`
/// (when `len` is not zero — else that division's own site stops it) is
/// within `[min(lo, 0), max(hi, 0)]`.
fn mean_bound(func: &Function, lhs: &Operand, rhs: &Operand) -> Option<(i128, i128)> {
    let whole = |o: &Operand| -> Option<LocalId> {
        let p = o.place()?;
        p.proj.is_empty().then_some(p.local)
    };
    let (a, m) = (whole(lhs)?, whole(rhs)?);
    // Every place a local is written, and how.
    enum Write<'a> {
        Assign(&'a Rvalue, usize),
        Other,
    }
    let writes = |l: LocalId| -> Vec<Write> {
        let mut out = Vec::new();
        for (bi, b) in func.blocks.iter().enumerate() {
            for s in &b.stmts {
                match &s.kind {
                    StmtKind::Assign { place, rvalue } if place.local == l => {
                        if place.proj.is_empty() {
                            out.push(Write::Assign(rvalue, bi));
                        } else {
                            out.push(Write::Other);
                        }
                    }
                    StmtKind::Op { dest: Some(d), .. } if d.local == l => out.push(Write::Other),
                    _ => {}
                }
            }
            if let Terminator::Call { dest: Some(d), .. } = &b.term {
                if d.local == l {
                    out.push(Write::Other);
                }
            }
        }
        out
    };
    let ops_writing = |l: LocalId| -> Vec<(&Op, &[Operand], usize)> {
        let mut out = Vec::new();
        for (bi, b) in func.blocks.iter().enumerate() {
            for s in &b.stmts {
                if let StmtKind::Op { dest: Some(d), op, args } = &s.kind {
                    if d.local == l {
                        out.push((op, args.as_slice(), bi));
                    }
                }
            }
        }
        out
    };
    // M = ListLength[P], once.
    let (m_ops, m_writes) = (ops_writing(m), writes(m));
    if m_ops.len() != 1 || m_writes.len() != 1 {
        return None;
    }
    let (op, args, _) = m_ops[0];
    if *op != Op::ListLength {
        return None;
    }
    let p = args.first()?.place()?.clone();
    let root = p.local;
    // Nothing writes P's root, lends it mutably, or changes it by an op.
    for b in &func.blocks {
        for s in &b.stmts {
            match &s.kind {
                StmtKind::Assign { place, rvalue } => {
                    if place.local == root {
                        return None;
                    }
                    if let Rvalue::Lend { place: lp, mutable: true } = rvalue {
                        if lp.local == root {
                            return None;
                        }
                    }
                }
                StmtKind::Op { dest, op, args } => {
                    if dest.as_ref().map_or(false, |d| d.local == root) {
                        return None;
                    }
                    let changes = matches!(op, Op::ListAppend | Op::ListRemove | Op::MapSet | Op::MapRemove | Op::SimdStore);
                    if changes && args.first().and_then(|a| a.place()).map_or(false, |pl| pl.local == root) {
                        return None;
                    }
                }
                _ => {}
            }
        }
        if let Terminator::Call { args, dest, .. } = &b.term {
            if dest.as_ref().map_or(false, |d| d.local == root) {
                return None;
            }
            for arg in args {
                if let Arg::Lend { place, mutable: true } = arg {
                    if place.local == root {
                        return None;
                    }
                }
            }
        }
    }
    // Does a local alias P: set once, by a lend of P or a copy of one that does?
    let aliases_p = |mut q: LocalId| -> bool {
        for _ in 0..8 {
            let w = writes(q);
            if w.len() != 1 {
                return false;
            }
            match &w[0] {
                Write::Assign(Rvalue::Lend { place, mutable: false }, _) => return place.local == p.local && place.proj == p.proj,
                Write::Assign(Rvalue::Use(Operand::Read(src)), _) if src.proj.is_empty() => q = src.local,
                _ => return false,
            }
        }
        false
    };
    // A: 0, and `A = T` with `T = A + X`.
    let mut zero = false;
    let mut add: Option<(LocalId, usize)> = None;
    for w in writes(a) {
        match w {
            Write::Assign(Rvalue::Use(Operand::Const(Const::Int(0))), _) if !zero => zero = true,
            Write::Assign(Rvalue::Use(Operand::Read(t)), _) if t.proj.is_empty() && add.is_none() => {
                let tw = writes(t.local);
                let [Write::Assign(Rvalue::Binary { op: BinOp::Add, lhs, rhs }, tb)] = tw.as_slice() else { return None };
                let x = match (whole(lhs), whole(rhs)) {
                    (Some(l), Some(r)) if l == a => r,
                    (Some(l), Some(r)) if r == a => l,
                    _ => return None,
                };
                add = Some((x, *tb));
            }
            _ => return None,
        }
    }
    let (x, add_block) = add?;
    if !zero {
        return None;
    }
    // X = Q[pos I], once, Q a lend of P.
    let xw = writes(x);
    let [Write::Assign(Rvalue::Use(Operand::Read(read)), _)] = xw.as_slice() else { return None };
    let [Proj::Index(Operand::Read(ip))] = read.proj.as_slice() else { return None };
    if !ip.proj.is_empty() || !aliases_p(read.local) {
        return None;
    }
    let i = ip.local;
    // I: 1 once, outside every cycle; otherwise `I = T2`, `T2 = I + 1`.
    let mut start: Option<usize> = None;
    let mut step: Option<usize> = None;
    for w in writes(i) {
        match w {
            Write::Assign(Rvalue::Use(Operand::Const(Const::Int(1))), bi) if start.is_none() => start = Some(bi),
            // `I = I + 1` written directly.
            Write::Assign(Rvalue::Binary { op: BinOp::Add, lhs, rhs }, bi) if step.is_none() => {
                let ok = (whole(lhs) == Some(i) && matches!(rhs, Operand::Const(Const::Int(1)))) || (whole(rhs) == Some(i) && matches!(lhs, Operand::Const(Const::Int(1))));
                if !ok {
                    return None;
                }
                step = Some(bi);
            }
            Write::Assign(Rvalue::Use(Operand::Read(t)), bi) if t.proj.is_empty() && step.is_none() => {
                let tw = writes(t.local);
                let [Write::Assign(Rvalue::Binary { op: BinOp::Add, lhs, rhs }, _)] = tw.as_slice() else { return None };
                let ok = (whole(lhs) == Some(i) && matches!(rhs, Operand::Const(Const::Int(1)))) || (whole(rhs) == Some(i) && matches!(lhs, Operand::Const(Const::Int(1))));
                if !ok {
                    return None;
                }
                step = Some(bi);
            }
            _ => return None,
        }
    }
    let (start, step) = (start?, step?);
    // Can `from`'s successors reach `to` without passing through `avoid`?
    let reaches = |from: usize, to: usize, avoid: Option<usize>| -> bool {
        let mut seen = vec![false; func.blocks.len()];
        let mut stack: Vec<usize> = crate::mir::verify::successors(&func.blocks[from].term).iter().map(|b| b.0 as usize).collect();
        while let Some(b) = stack.pop() {
            if Some(b) == avoid || b >= seen.len() || seen[b] {
                continue;
            }
            if b == to {
                return true;
            }
            seen[b] = true;
            stack.extend(crate::mir::verify::successors(&func.blocks[b].term).iter().map(|x| x.0 as usize));
        }
        false
    };
    if reaches(start, start, None) || reaches(add_block, add_block, Some(step)) {
        return None;
    }
    // The read, the sum and the store, in one block and in that order, with
    // nothing between them writing X or I: each add adds a fresh element.
    let stmts = &func.blocks[add_block].stmts;
    let at = |pred: &dyn Fn(&StmtKind) -> bool| stmts.iter().position(|s| pred(&s.kind));
    let r = at(&|k| matches!(k, StmtKind::Assign { place, .. } if place.local == x && place.proj.is_empty()));
    let s_ = at(&|k| matches!(k, StmtKind::Assign { rvalue: Rvalue::Binary { op: BinOp::Add, lhs, rhs }, .. } if whole(lhs) == Some(x) || whole(rhs) == Some(x)));
    let st = at(&|k| matches!(k, StmtKind::Assign { place, .. } if place.local == a && place.proj.is_empty()));
    let (Some(r), Some(s_), Some(st)) = (r, s_, st) else { return None };
    if !(r < s_ && s_ < st) {
        return None;
    }
    let touches = |k: &StmtKind| matches!(k, StmtKind::Assign { place, .. } if place.local == x || place.local == i);
    if stmts[r + 1..st].iter().any(|s| touches(&s.kind)) {
        return None;
    }
    // The elements' range: X holds a value of its type.
    let (lo, hi) = smt::int_bounds(&func.locals[x.0 as usize].ty)?;
    Some((lo.min(0), hi.max(0)))
}

/// Is `k` one known value that none of `keys` (every key a map can hold,
/// possibly more) is? Only then is a `set` of it surely a new entry.
fn surely_absent(keys: &Domain, k: &Domain) -> bool {
    match (keys, k) {
        (Domain::Empty, _) => single_value(k),
        (Domain::Ints(ks), Domain::Ints(x)) => x.single().map_or(false, |v| !ks.contains(v)),
        (Domain::Text { values: Some(ks), .. }, Domain::Text { values: Some(x), .. }) => x.len() == 1 && !ks.contains(&x[0]),
        (Domain::Chars { values: Some(ks) }, Domain::Chars { values: Some(x) }) => x.len() == 1 && !ks.contains(&x[0]),
        (Domain::Bools(t, f), Domain::Bools(xt, xf)) => (*xt && !*xf && !*t) || (*xf && !*xt && !*f),
        _ => false,
    }
}

/// Exactly one value, known.
fn single_value(d: &Domain) -> bool {
    match d {
        Domain::Ints(s) | Domain::Duration(s) => s.single().is_some(),
        Domain::Text { values: Some(v), .. } | Domain::Chars { values: Some(v) } => v.len() == 1,
        Domain::Bools(t, f) => t != f,
        _ => false,
    }
}
