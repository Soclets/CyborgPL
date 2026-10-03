//! Layer 3 — the exact question.
//!
//! `design/precog.md`: a site layer 2 could not settle is asked exactly,
//! as a **bit-precise SMT query over the real program semantics**, put to
//! a **portfolio of Z3 and CVC5**, both pinned and both run to their full
//! budget. Any `unsat` proves. Both `sat` → Z3's counterexample, by fixed
//! order, so the same program gives the same answer on every machine.
//! Both `unknown` → unproven.
//!
//! The query is built by unrolling the flow graph into a straight line:
//! each block, at each trip through it, becomes its own copy, and a local
//! gets a fresh name at every assignment. Loops are unrolled to a bound,
//! and **a bound means a proof about a prefix** — so when the unrolling
//! had to cut a path, an `unsat` is reported as *unproven*, exactly as
//! `precog.md` says ("a site inside a loop may be proven for the first 16
//! trips and unproven after"). A `sat` is always real: it is a path the
//! solver actually found.
//!
//! Every limit is a step count. `rlimit` is Z3's own, never a timeout,
//! because a timeout would make the answer depend on the machine.

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::ast::{BinOp, UnOp};
use crate::mir::*;
use crate::types::{Bound, IntW, Ty};

/// Provisional, Kespar's (`precog.md`, Open): how many times one loop is
/// unrolled into the query, and how many unrollings the whole query may
/// hold. Deeper made Z3 quadratic on nested loops.
pub const UNROLL_PER_LOOP: u32 = 16;
pub const UNROLL_TOTAL: usize = 64;
/// Provisional: the solver's own step budget, never a clock.
pub const RLIMIT: u64 = 10_000_000;
/// CVC5's, in its own steps, which are far coarser than Z3's: a query Z3
/// answered in 0.04 s at 10,000,000 had CVC5 working for minutes at the
/// same number, and 1,000,000 of CVC5's took 2.7 s (a word-reading loop,
/// 2026-09-30). Still a step count, so still the same answer everywhere.
pub const CVC5_RLIMIT: u64 = 1_000_000;

#[derive(Debug, Clone, PartialEq)]
pub enum Answer {
    /// No input reaches the site with a bad value: proven.
    Unsat,
    /// One does, and here it is.
    Sat(String),
    /// The budget ran out, or the solver would not say.
    Unknown,
}

/// What the portfolio is made of on this machine. Named in the report,
/// because a portfolio missing a member is not the portfolio the design
/// decided on and the reader should know which answered.
pub struct Portfolio {
    pub members: Vec<Solver>,
}

pub struct Solver {
    pub name: &'static str,
    pub program: &'static str,
    pub version: String,
}

impl Portfolio {
    pub fn find() -> Portfolio {
        let mut members = Vec::new();
        for (name, program, args) in [("Z3", "z3", &["--version"][..]), ("CVC5", "cvc5", &["--version"][..])] {
            if let Ok(out) = std::process::Command::new(program).args(args).output() {
                let version = String::from_utf8_lossy(&out.stdout).lines().next().unwrap_or("").trim().to_string();
                members.push(Solver { name, program, version });
            }
        }
        Portfolio { members }
    }

    pub fn is_complete(&self) -> bool {
        self.members.len() == 2
    }

    pub fn describe(&self) -> String {
        if self.members.is_empty() {
            return "no solver: layer 3 asked nothing".into();
        }
        let list = self.members.iter().map(|s| format!("{} ({})", s.name, s.version)).collect::<Vec<_>>().join(" + ");
        if self.is_complete() {
            list
        } else {
            format!("{} — the portfolio is Z3 *and* CVC5, and only one is here, so an `unsat` rests on one solver where the design wants two", list)
        }
    }

    /// Put one query to every member, each to its full budget — never
    /// first-to-answer, so the answer does not depend on which machine ran
    /// faster.
    pub fn ask(&self, q: &Query) -> (Answer, Vec<(&'static str, Answer)>) {
        let mut answers = Vec::new();
        for s in &self.members {
            answers.push((s.name, run(s, q)));
        }
        let combined = combine(&answers);
        (combined, answers)
    }

    /// Put a bare query to the first member. For asking about a query
    /// itself — whether it was answerable at all — rather than about a
    /// program.
    pub fn ask_text(&self, text: &str) -> Answer {
        let Some(s) = self.members.first() else { return Answer::Unknown };
        run(s, &Query { text: text.to_string(), assumptions: String::new(), truncated: false, inputs: Vec::new() })
    }
}

fn run(s: &Solver, q: &Query) -> Answer {
    // The query goes in on stdin, so no file is left behind — a build that
    // is stopped halfway used to leave its queries in the temp directory,
    // some of them megabytes. `CCA_KEEP_SMT=1` still writes each one out,
    // for reading. The step budget rides in the query itself
    // (`set-option :rlimit`): Z3's `RLIMIT`, and CVC5's own `CVC5_RLIMIT`
    // in its coarser steps.
    let cvc5_text;
    let text: &str = if s.name == "CVC5" {
        cvc5_text = q.text.replacen(&format!("(set-option :rlimit {})", RLIMIT), &format!("(set-option :rlimit {})", CVC5_RLIMIT), 1);
        &cvc5_text
    } else {
        &q.text
    };
    if std::env::var("CCA_KEEP_SMT").is_ok() {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join("cca-smt");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join(format!("q{}-{}-{}.smt2", std::process::id(), n, s.name)), text);
    }
    let mut cmd = std::process::Command::new(s.program);
    match s.name {
        "Z3" => {
            cmd.arg("-smt2").arg("-in");
        }
        _ => {
            cmd.arg("--lang=smt2");
        }
    }
    cmd.stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::null());
    let Ok(mut child) = cmd.spawn() else { return Answer::Unknown };
    if let Some(mut input) = child.stdin.take() {
        use std::io::Write as _;
        let _ = input.write_all(text.as_bytes());
    }
    let Ok(out) = child.wait_with_output() else { return Answer::Unknown };
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    // Not the first line: a solver may print `unsupported`, a warning, or
    // an echo before it answers. The answer is the first line that is one.
    let said = text.lines().map(str::trim).find(|l| matches!(*l, "sat" | "unsat" | "unknown"));
    match said {
        Some("unsat") => Answer::Unsat,
        Some("sat") => Answer::Sat(model_of(&text, &q.inputs)),
        _ => Answer::Unknown,
    }
}

/// Any `unsat` proves — a wrong `unsat` would be a solver bug, which is
/// what the Precog oracle exists to catch. Both `sat` takes the first
/// member's counterexample, by fixed order.
fn combine(answers: &[(&'static str, Answer)]) -> Answer {
    if answers.iter().any(|(_, a)| *a == Answer::Unsat) {
        return Answer::Unsat;
    }
    for (_, a) in answers {
        if let Answer::Sat(m) = a {
            return Answer::Sat(m.clone());
        }
    }
    Answer::Unknown
}

/// Did the members disagree? `unsat` from one and `sat` from the other is
/// a bug in a solver or in the query both of them received — never in the
/// program (`precog.md`, owner: c).
pub fn disagreed(answers: &[(&'static str, Answer)]) -> bool {
    answers.iter().any(|(_, a)| *a == Answer::Unsat) && answers.iter().any(|(_, a)| matches!(a, Answer::Sat(_)))
}

/// The values a solver said make the site fail, in the program's own
/// words. Only the query's inputs are worth showing — a parameter, what a
/// call gave back, how long something was — never the working.
fn model_of(text: &str, inputs: &[Input]) -> String {
    let mut vals = asked_values(text);
    if vals.is_empty() {
        vals = defined_values(text);
    }
    let mut out = Vec::new();
    for i in inputs {
        if let Some(v) = vals.get(&i.sym) {
            if let Some(n) = literal(v, i.signed) {
                out.push(format!("{} = {}", i.show, n));
            }
        }
    }
    out.join(", ")
}

/// What `(get-value (a b …))` answered: `((a v) (b v) …)`, as name → value.
fn asked_values(text: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let b = text.as_bytes();
    // After the `sat` line, the first list is the answer.
    let Some(at) = text.find("sat") else { return out };
    let Some(open) = text[at..].find('(').map(|o| o + at) else { return out };
    let Some((list, _)) = read_sexp(b, open) else { return out };
    let lb = list.as_bytes();
    let mut i = 1;
    while let Some((pair, k)) = read_sexp(lb, i) {
        if pair == ")" || !pair.starts_with('(') {
            break;
        }
        let pb = pair.as_bytes();
        if let (Some((name, j)), true) = (read_sexp(pb, 1), pair.len() > 2) {
            if let Some((value, _)) = read_sexp(pb, j) {
                out.insert(name, value);
            }
        }
        i = k;
    }
    out
}

/// Every `(define-fun name () sort body)` a model holds, as name → body.
/// Read as s-expressions and not line by line, because Z3 puts the body
/// on the line after the sort and CVC5 puts it on the same line.
fn defined_values(text: &str) -> HashMap<String, String> {
    let b = text.as_bytes();
    let mut out = HashMap::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'(' && b[i + 1..].starts_with(b"define-fun") {
            let mut j = i + 1 + "define-fun".len();
            let Some((name, k)) = read_sexp(b, j) else { break };
            j = k;
            let Some(k) = read_sexp(b, j).map(|(_, k)| k) else { break }; // ()
            let Some(k) = read_sexp(b, k).map(|(_, k)| k) else { break }; // the sort
            let Some((body, k)) = read_sexp(b, k) else { break };
            out.insert(name, body);
            i = k;
        } else {
            i += 1;
        }
    }
    out
}

/// One s-expression starting at or after `i`: a `|quoted symbol|`, a bare
/// atom, or a balanced `(…)`. Gives the text and where it ended.
fn read_sexp(b: &[u8], mut i: usize) -> Option<(String, usize)> {
    while i < b.len() && b[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= b.len() {
        return None;
    }
    let start = i;
    match b[i] {
        b'|' => {
            i += 1;
            while i < b.len() && b[i] != b'|' {
                i += 1;
            }
            i += 1;
        }
        b'(' => {
            let mut depth = 0usize;
            while i < b.len() {
                match b[i] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            i += 1;
                            break;
                        }
                    }
                    b'|' => {
                        i += 1;
                        while i < b.len() && b[i] != b'|' {
                            i += 1;
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
            if depth != 0 {
                return None;
            }
        }
        _ => {
            while i < b.len() && !b[i].is_ascii_whitespace() && b[i] != b'(' && b[i] != b')' {
                i += 1;
            }
        }
    }
    if i > b.len() {
        return None;
    }
    Some((String::from_utf8_lossy(&b[start..i]).into_owned(), i))
}

/// A model body that is a plain number, written the way the type reads
/// it: a `uint8` of 200 is 200 and not −56.
fn literal(v: &str, signed: bool) -> Option<String> {
    if v == "true" || v == "false" {
        return Some(v.to_string());
    }
    let (digits, radix) = if let Some(h) = v.strip_prefix("#x") {
        (h, 16)
    } else if let Some(b) = v.strip_prefix("#b") {
        (b, 2)
    } else {
        return None;
    };
    let n = u128::from_str_radix(digits, radix).ok()?;
    let bits = digits.len() * if radix == 16 { 4 } else { 1 };
    if signed {
        Some(as_signed(n, bits).to_string())
    } else {
        Some(n.to_string())
    }
}

/// How a length reads in a counterexample.
fn length_word(op: &Op) -> &'static str {
    match op {
        Op::ListLength => "how many elements the list has",
        Op::ArrayLength => "how many elements the array has",
        Op::MapLength => "how many entries the map has",
        Op::StrBytes => "how many bytes the text has",
        Op::StrPoints => "how many code points the text has",
        Op::StrGraphemes => "how many characters the text has",
        _ => "a length",
    }
}

fn as_signed(n: u128, bits: usize) -> i128 {
    if bits >= 128 {
        return n as i128;
    }
    let top = 1u128 << (bits - 1);
    if n >= top {
        (n as i128) - ((1i128) << bits)
    } else {
        n as i128
    }
}

// ---------------------------------------------------------------- the query

pub struct Query {
    pub text: String,
    /// Everything the program allows, without the question. If *this*
    /// has no model then the question was never really asked: every
    /// answer would be `unsat`, and `unsat` reads as a proof. Nothing
    /// should ever be proven by a query that could not be satisfied
    /// whatever the program did.
    pub assumptions: String,
    /// The unrolling had to cut a path, so an `unsat` is a proof about a
    /// prefix and not about the program.
    pub truncated: bool,
    /// What the query knows nothing about, and so what a counterexample
    /// is made of.
    pub inputs: Vec<Input>,
}

/// A value the query cannot see inside: a parameter, what a call gave
/// back, how long something is. `sym` is what the query calls it, `show`
/// is what the program calls it.
pub struct Input {
    pub sym: String,
    pub show: String,
    pub signed: bool,
}

/// Build the exact question for one site: *can any input reach here with a
/// value this instruction cannot take?*
pub fn encode(p: &Program, f: usize, site: SiteId, facts: &crate::precog::Facts) -> Option<Query> {
    let mut aliased = std::collections::HashSet::new();
    for b in &p.functions[f].blocks {
        for s in &b.stmts {
            if let StmtKind::Assign { rvalue: Rvalue::Lend { place, mutable: true }, .. } = &s.kind {
                aliased.insert(place.local);
            }
        }
    }
    let borrows = shared_borrows(&p.functions[f]);
    let mut e = Enc { p, f, defs: Vec::new(), next: 0, truncated: false, reached: Vec::new(), inputs: Vec::new(), aliased, unexpressible: false, facts, borrows, entries: HashMap::new() };
    e.build(site)?;
    if e.reached.is_empty() || e.unexpressible {
        return None;
    }
    let mut text = String::new();
    let _ = writeln!(text, "; CyborgPL — Precog layer 3, {}", describe_site(p, f, site));
    let _ = writeln!(text, "(set-option :rlimit {})", RLIMIT);
    let _ = writeln!(text, "(set-option :produce-models true)");
    let _ = writeln!(text, "(set-logic QF_BV)");
    for d in &e.defs {
        let _ = writeln!(text, "{}", d);
    }
    let assumptions = format!("{}(check-sat)\n", text);
    let _ = writeln!(text, "(assert (or {}))", e.reached.join(" "));
    let _ = writeln!(text, "(check-sat)");
    // Only the inputs' values, never the whole model: a model of a query
    // with 95,000 definitions was printed as 3.9 GB, and reading it took
    // twenty minutes of a build (check claude's slow builds, 2026-09-29).
    if !e.inputs.is_empty() {
        let _ = writeln!(text, "(get-value ({}))", e.inputs.iter().map(|i| i.sym.clone()).collect::<Vec<_>>().join(" "));
    }
    Some(Query { text, assumptions, truncated: e.truncated, inputs: e.inputs })
}

fn describe_site(p: &Program, f: usize, site: SiteId) -> String {
    let func = &p.functions[f];
    let s = func.site(site);
    let spell = |t: &Ty| crate::types::TySpelling { ty: t, structs: &p.items.structs, enums: &p.items.enums }.to_string();
    format!("in {}: {}", func.name, s.sentence(&spell))
}

/// A term, and the width it is held at. `None` means the value is not
/// something bit-vectors describe (a text, a list, a struct) — the query
/// then knows nothing about it, which is sound and simply proves less.
#[derive(Clone)]
struct Val {
    term: String,
    bits: u32,
    signed: bool,
}

struct Enc<'a> {
    p: &'a Program,
    f: usize,
    defs: Vec<String>,
    next: u32,
    truncated: bool,
    /// One term per copy of the site: it is reached *and* the value is bad.
    reached: Vec<String>,
    inputs: Vec<Input>,
    /// Locals something in this function takes a mutable borrow of into a
    /// name: a write through that name changes them without saying so, so
    /// nothing about them is trusted past a call, a task or an operation.
    aliased: std::collections::HashSet<LocalId>,
    /// Some copy of the site could not be asked about: the query proves
    /// nothing.
    unexpressible: bool,
    /// Layer 2's invariants, per block.
    facts: &'a crate::precog::Facts,
    /// Locals that are nothing but a shared borrow of one place, reached by
    /// fields alone, bound once in the whole function.
    borrows: HashMap<LocalId, Place>,
    /// Per loop head, the states it was entered with from outside it.
    entries: HashMap<usize, Vec<Env>>,
}

/// What the query knows at one point: the value of each whole-number
/// local, and the length of each list, text or map it has looked at.
///
/// A length is part of a collection's value. Reading `list.length` twice
/// with nothing in between that could change the list must give the same
/// number both times — a fresh unknown for each read would let a guard on
/// the first read say nothing about the second, and the example of that is
/// `if list.length[…] == 0 { fail … }` followed by a division by it.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Key {
    Local(LocalId),
    /// A length of a place: its local, then its field path and which
    /// length (bytes, points, elements…), spelled out.
    Len(LocalId, String),
}

type Env = HashMap<Key, Val>;

impl<'a> Enc<'a> {
    /// A value the query knows nothing about. It is declared under a plain
    /// name and remembered under the program's own, so a counterexample
    /// reads as the program reads.
    fn fresh(&mut self, prefix: &str, bits: u32, signed: bool, show: String) -> Val {
        let sym = format!("{}{}", prefix, self.next);
        self.next += 1;
        self.defs.push(format!("(declare-fun {} () (_ BitVec {}))", sym, bits));
        // Two inputs that would read the same are told apart, or the
        // counterexample would name one thing twice.
        let same = self.inputs.iter().filter(|i| i.show == show || i.show.starts_with(&format!("{} (", show))).count();
        let show = if same == 0 { show } else { format!("{} ({})", show, same + 1) };
        self.inputs.push(Input { sym: sym.clone(), show, signed });
        Val { term: sym, bits, signed }
    }

    fn define(&mut self, bits: u32, body: &str, signed: bool) -> Val {
        let name = format!("v{}", self.next);
        self.next += 1;
        self.defs.push(format!("(define-fun {} () (_ BitVec {}) {})", name, bits, body));
        Val { term: name, bits, signed }
    }

    /// A branch point the query does not decide — a `match`, a call's ok
    /// and failure edges, an `if` on a condition it cannot read — split
    /// into one guard per way out, told apart by a free selector. Every
    /// successor may be the one taken, and never two at once: handing them
    /// all the same guard let the join keep only the first (check claude,
    /// 2026-09-29: a `match` case's 0 never reached the division, which was
    /// "proven").
    fn choices(&mut self, guard: &str, n: usize) -> Vec<String> {
        if n <= 1 {
            return vec![guard.to_string(); n];
        }
        let sel = format!("sel{}", self.next);
        self.next += 1;
        self.defs.push(format!("(declare-fun {} () (_ BitVec 16))", sel));
        (0..n).map(|k| self.define_bool(&format!("(and {} (= {} (_ bv{} 16)))", guard, sel, k))).collect()
    }

    fn define_bool(&mut self, body: &str) -> String {
        let name = format!("b{}", self.next);
        self.next += 1;
        self.defs.push(format!("(define-fun {} () Bool {})", name, body));
        name
    }

    fn func(&self) -> &'a Function {
        &self.p.functions[self.f]
    }

    /// What to call a callee in a counterexample.
    fn callee_name(&self, c: &Callee) -> String {
        match c {
            Callee::Fn(i) => self.p.functions[i.0 as usize].name.clone(),
            Callee::Extern(id) => self.p.items.externs[id.0 as usize].name.clone(),
            Callee::Value(_) => "that function".to_string(),
            Callee::Op(_) => "that operation".to_string(),
            Callee::Convert(t) => format!("that `to.{}`", crate::types::TySpelling { ty: t, structs: &self.p.items.structs, enums: &self.p.items.enums }),
        }
    }

    /// Unroll the graph and emit the question at every copy of the site.
    fn build(&mut self, site: SiteId) -> Option<()> {
        let f = self.func();
        let back = crate::precog::back_edges(f);
        // Nodes of the unrolled graph: (block, level). A back edge moves to
        // the next level; everything else stays.
        let mut incoming: HashMap<(usize, u32), Vec<(String, Env)>> = HashMap::new();
        let mut order: Vec<(usize, u32)> = Vec::new();
        let mut seen: Vec<(usize, u32)> = Vec::new();
        let mut edges: Vec<((usize, u32), (usize, u32))> = Vec::new();
        let mut dropped: Vec<((usize, u32), usize)> = Vec::new();

        // The entry: parameters are whatever their type allows.
        let mut env0: Env = Env::new();
        for l in &f.params {
            let ty = f.locals[l.0 as usize].ty.clone();
            if let Some((bits, signed)) = int_shape(&ty) {
                let v = self.fresh("in", bits, signed, self.p.functions[self.f].locals[l.0 as usize].name.clone().unwrap_or_else(|| "a parameter".to_string()));
                self.constrain_range(&v, &ty);
                env0.insert(Key::Local(*l), v);
            }
        }
        incoming.insert((0, 0), vec![("true".to_string(), env0)]);
        let mut queue = vec![(0usize, 0u32)];
        let mut unrollings = 0usize;

        while let Some(node) = queue.first().copied() {
            queue.remove(0);
            if seen.contains(&node) {
                continue;
            }
            seen.push(node);
            order.push(node);
            let (b, level) = node;
            for t in crate::mir::verify::successors(&f.blocks[b].term) {
                let ti = t.0 as usize;
                if ti >= f.blocks.len() {
                    continue;
                }
                let is_back = back.contains(&(b, ti));
                let next_level = if is_back { level + 1 } else { level };
                if is_back && next_level > UNROLL_PER_LOOP {
                    // Not cut: the head at the last level already stands for
                    // every later trip (see `last_trip`), so nothing is lost
                    // by not going round again — when that copy of the head
                    // is on the way here. Checked below.
                    dropped.push((node, ti));
                    continue;
                }
                if is_back && unrollings >= UNROLL_TOTAL {
                    // Cut: the whole query ran out of room before this loop
                    // reached its last level, so an `unsat` would only be
                    // about the trips we did look at.
                    self.truncated = true;
                    continue;
                }
                if is_back {
                    unrollings += 1;
                }
                edges.push((node, (ti, next_level)));
                if !seen.contains(&(ti, next_level)) && !queue.contains(&(ti, next_level)) {
                    queue.push((ti, next_level));
                }
            }
        }

        // Levels are shared, so an inner loop's trips climb them too. An
        // outer loop whose inner loop runs twelve times reached the last
        // level inside its second trip, and its way round from there was
        // dropped — but its head's last copy was reached only where the
        // inner loop ended sooner, which it never does. Every way out of
        // the loop was then impossible and `unsat` proved a narrowing after
        // it (check claude, seed 6313). A way round is dropped soundly only
        // when every path to it passes that last copy of its head; where
        // one does not, that copy is widened: every entry into the loop
        // arrives there as well (see `send`). With no such copy to stand
        // for them, the query is cut.
        let mut widened: std::collections::HashSet<(usize, u32)> = std::collections::HashSet::new();
        if !dropped.is_empty() {
            let mut next: HashMap<(usize, u32), Vec<(usize, u32)>> = HashMap::new();
            for (a, b) in &edges {
                next.entry(*a).or_default().push(*b);
            }
            let mut avoiding: HashMap<usize, std::collections::HashSet<(usize, u32)>> = HashMap::new();
            for (from, head) in &dropped {
                let reach = avoiding.entry(*head).or_insert_with(|| {
                    let skip = (*head, UNROLL_PER_LOOP);
                    let mut got = std::collections::HashSet::new();
                    let mut stack = vec![(0usize, 0u32)];
                    while let Some(n) = stack.pop() {
                        if n == skip || !got.insert(n) {
                            continue;
                        }
                        stack.extend(next.get(&n).into_iter().flatten().copied());
                    }
                    got
                });
                if reach.contains(from) {
                    if seen.contains(&(*head, UNROLL_PER_LOOP)) {
                        widened.insert((*head, UNROLL_PER_LOOP));
                    } else {
                        self.truncated = true;
                    }
                }
            }
        }

        // Walked in an order where every node comes after everything that
        // arrives at it. The order of discovery is not that: two loops in a
        // row share the levels, so the second loop's head at the last level
        // was reached by its own unrolling from the first loop's first trip
        // — where the first loop has not ended — and emitted before the
        // first loop's real way out at that level arrived. That arrival was
        // dropped, the site was asked only on a path that cannot happen,
        // and `unsat` said proven (check claude, seed 133).
        let order = {
            let mut preds: HashMap<(usize, u32), usize> = order.iter().map(|n| (*n, 0)).collect();
            for (_, to) in &edges {
                *preds.entry(*to).or_default() += 1;
            }
            let mut ready: Vec<(usize, u32)> = order.iter().copied().filter(|n| preds[n] == 0).collect();
            let mut sorted = Vec::with_capacity(order.len());
            while !ready.is_empty() {
                let n = ready.remove(0);
                sorted.push(n);
                for (from, to) in &edges {
                    if *from == n {
                        let p = preds.get_mut(to).unwrap();
                        *p -= 1;
                        if *p == 0 {
                            ready.push(*to);
                        }
                    }
                }
            }
            // A cycle left over means the graph was not what an unrolling
            // makes (an irreducible loop): nothing is claimed about it.
            if sorted.len() != order.len() {
                return None;
            }
            sorted
        };

        // For each loop head, everything any loop through it may write, and
        // what it may resize (an element store resizes nothing).
        let mut last_trip: HashMap<usize, std::collections::HashSet<LocalId>> = HashMap::new();
        let mut resized: HashMap<usize, std::collections::HashSet<LocalId>> = HashMap::new();
        let mut bodies: HashMap<usize, std::collections::HashSet<usize>> = HashMap::new();
        let mut latches: HashMap<usize, usize> = HashMap::new();
        for (from, head) in back.iter() {
            let body = crate::precog::natural_loop(f, *head, *from);
            *latches.entry(*head).or_default() += 1;
            let entry = last_trip.entry(*head).or_default();
            let grows = resized.entry(*head).or_default();
            for blk in &body {
                written_in(&f.blocks[*blk], entry);
                resized_in(&f.blocks[*blk], grows);
            }
            bodies.entry(*head).or_default().extend(body);
        }
        // Lock-step pairs, for a loop with one way round and no loop inside.
        let mut lockstep: HashMap<usize, Vec<(LocalId, LocalId)>> = HashMap::new();
        for (head, body) in &bodies {
            let nested = back.iter().any(|(fr, h)| h != head && body.contains(h) && body.contains(fr));
            if latches.get(head) == Some(&1) && !nested {
                let pairs = lockstep_pairs(f, *head, body);
                if !pairs.is_empty() {
                    lockstep.insert(*head, pairs);
                }
            }
        }

        // Walk the nodes in the order they were discovered, merging what
        // arrives and emitting what each block does.
        let mut walked: std::collections::HashSet<(usize, u32)> = std::collections::HashSet::new();
        for node in order.clone() {
            let (b, level) = node;
            let Some(arrivals) = incoming.get(&node).cloned() else { continue };
            walked.insert(node);
            let (guard, mut env) = self.merge(&arrivals);
            let stmts = self.func().blocks[b].stmts.clone();
            for s in &stmts {
                match &s.kind {
                    StmtKind::Assign { place, rvalue } => {
                        let v = self.rvalue(rvalue, &env);
                        self.assign(&mut env, place, v);
                    }
                    StmtKind::Op { dest, op, args } => {
                        let v = match op {
                            Op::ListLength | Op::ArrayLength | Op::MapLength | Op::StrBytes | Op::StrPoints | Op::StrGraphemes => self.length(op, args, &mut env),
                            _ => self.op(op, args, &env),
                        };
                        // An operation that changes a collection's size.
                        if matches!(op, Op::ListAppend | Op::ListRemove | Op::MapSet | Op::MapRemove | Op::SimdStore | Op::SimdStoreSome) {
                            if let Some(p) = args.first().and_then(|a| a.place()) {
                                // An append makes the list one longer, a
                                // removal one shorter: said, not forgotten.
                                let key = Key::Len(p.local, format!("{:?}{:?}", Vec::<Proj>::new(), Op::ListLength));
                                let before = if p.proj.is_empty() && !self.aliased.contains(&p.local) { env.get(&key).cloned() } else { None };
                                self.forget(&mut env, p.local);
                                if let (Some(old), Op::ListAppend | Op::ListRemove) = (before, op) {
                                    let f = if *op == Op::ListAppend { "bvadd" } else { "bvsub" };
                                    let v = self.define(old.bits, &format!("({} {} {})", f, old.term, bv(1, old.bits)), false);
                                    env.insert(key, v);
                                }
                            }
                            self.forget_aliased(&mut env);
                        }
                        if let Some(d) = dest {
                            self.assign(&mut env, d, v);
                            // `list.fill`: as long as its count, which its
                            // site has said is not below 0 (owner,
                            // 2026-09-30) — tied, so `'xs'[n]` after
                            // `list.fill[…, 'n']` is inside.
                            if *op == Op::ListFill && d.proj.is_empty() && !self.aliased.contains(&d.local) {
                                // At 64 bits, as the code generator takes it.
                                if let Some(c) = args.get(1).and_then(|a| self.operand(a, &env)) {
                                    let n = self.resize(c, 64, false);
                                    let key = Key::Len(d.local, format!("{:?}{:?}", Vec::<Proj>::new(), Op::ListLength));
                                    env.insert(key, n);
                                }
                            }
                        }
                    }
                    StmtKind::Task { captures, .. } => {
                        // A task lent something mutably may change it at any
                        // moment until its block ends.
                        for (verb, p) in captures {
                            if matches!(verb, crate::ast::Verb::LendMut | crate::ast::Verb::Give) {
                                self.forget(&mut env, p.local);
                            }
                        }
                        self.forget_aliased(&mut env);
                    }
                    StmtKind::Asm { outs, lends, .. } => {
                        for (p, _) in outs {
                            self.forget(&mut env, p.local);
                        }
                        // Written through its address: the values, never
                        // the length, which the assembly cannot reach.
                        for (p, m, _) in lends {
                            if *m {
                                self.forget_value(&mut env, p.local);
                            }
                        }
                        self.forget_aliased(&mut env);
                    }
                    StmtKind::Site(id) if *id == site => {
                        match self.bad_value(*id, &env) {
                            Some(bad) => {
                                let both = self.define_bool(&format!("(and {} {})", guard, bad));
                                self.reached.push(both);
                            }
                            // A copy of the site the query cannot put into
                            // words. Leaving it out would ask only about the
                            // copies it can, and an `unsat` about some of the
                            // ways to reach a site is not a proof about it.
                            None => self.unexpressible = true,
                        }
                    }
                    _ => {}
                }
            }
            // Where it goes, and what each successor then knows.
            let term = self.func().blocks[b].term.clone();
            let mut send = |this: &mut Self, to: BlockId, g: String, mut e: Env| {
                let ti = to.0 as usize;
                let is_back = crate::precog::back_edges(this.func()).contains(&(b, ti));
                let lvl = if is_back { level + 1 } else { level };
                // The last level a loop head is copied at stands for that
                // trip *and every one after it*: what the loop writes is
                // forgotten, what it leaves alone is kept. Any later trip
                // starts in a state that is one instance of this, so a site
                // proven here is proven for them too — and the back edge out
                // of this copy need not be followed at all.
                // Entered from outside the loop: remembered, with the length
                // of each list that moves in lock-step, so the last trip can
                // be tied back to it.
                let entering = !is_back && bodies.get(&ti).map_or(false, |body| !body.contains(&b));
                // A widened head's last copy (above) stands for trips that
                // began at any entry into the loop, so every entry arrives
                // there too — with what the loop writes forgotten, and the
                // conditions of the path that entered kept. An entry walked
                // after that copy could not be added: the query is cut.
                if entering && lvl < UNROLL_PER_LOOP && widened.contains(&(ti, UNROLL_PER_LOOP)) {
                    if walked.contains(&(ti, UNROLL_PER_LOOP)) {
                        this.truncated = true;
                    } else {
                        let mut late = e.clone();
                        let grows = resized.get(&ti).cloned().unwrap_or_default();
                        for l in sorted(last_trip.get(&ti).cloned().unwrap_or_default()) {
                            if grows.contains(&l) {
                                this.forget(&mut late, l);
                            } else {
                                this.forget_value(&mut late, l);
                            }
                            this.assume_fact(&mut late, ti, l);
                        }
                        this.forget_aliased(&mut late);
                        incoming.entry((ti, UNROLL_PER_LOOP)).or_default().push((g.clone(), late));
                    }
                }
                if entering {
                    for (list, _) in lockstep.get(&ti).cloned().unwrap_or_default() {
                        let _ = this.length(&Op::ListLength, &[Operand::Read(Place::local(list))], &mut e);
                    }
                    this.entries.entry(ti).or_default().push(e.clone());
                }
                if lvl == UNROLL_PER_LOOP {
                    if let Some(written) = last_trip.get(&ti).cloned() {
                        let grows = resized.get(&ti).cloned().unwrap_or_default();
                        for l in &sorted(written) {
                            if grows.contains(l) {
                                this.forget(&mut e, *l);
                            } else {
                                // Written only element by element: every
                                // length the query knows of it stays.
                                this.forget_value(&mut e, *l);
                            }
                            // What layer 2 knows holds on every trip through
                            // this head: `'i'` counts up from 1, so it is
                            // never below 1, whichever trip this stands for.
                            this.assume_fact(&mut e, ti, *l);
                        }
                        this.forget_aliased(&mut e);
                        // Lock-step: the length is the entry's length plus
                        // how far the counter has come since. Only for a
                        // loop entered from one place, so "entry" is one
                        // state.
                        if let (Some(pairs), Some([entry])) = (lockstep.get(&ti), this.entries.get(&ti).map(|v| v.as_slice())) {
                            let entry = entry.clone();
                            for (list, c) in pairs {
                                let key = Key::Len(*list, format!("{:?}{:?}", Vec::<Proj>::new(), Op::ListLength));
                                let (Some(len0), Some(c0), Some(ch)) = (entry.get(&key).cloned(), entry.get(&Key::Local(*c)).cloned(), e.get(&Key::Local(*c)).cloned()) else { continue };
                                let Some(lenh) = this.length(&Op::ListLength, &[Operand::Read(Place::local(*list))], &mut e) else { continue };
                                let w = 130;
                                let (l0, lh, c0, ch) = (this.resize(len0, w, true), this.resize(lenh, w, true), this.resize(c0, w, true), this.resize(ch, w, true));
                                this.defs.push(format!("(assert (= {} (bvadd {} (bvsub {} {}))))", lh.term, l0.term, ch.term, c0.term));
                            }
                        }
                    }
                }
                if seen.contains(&(ti, lvl)) || incoming.contains_key(&(ti, lvl)) {
                    incoming.entry((ti, lvl)).or_default().push((g, e));
                }
            };
            match &term {
                Terminator::Goto(t) => send(self, *t, guard.clone(), env.clone()),
                Terminator::If { cond, then, else_ } => {
                    let c = self.operand(cond, &env);
                    match c {
                        Some(v) if v.bits == 1 => {
                            let is_true = format!("(= {} (_ bv1 1))", v.term);
                            let g1 = self.define_bool(&format!("(and {} {})", guard, is_true));
                            let g2 = self.define_bool(&format!("(and {} (not {}))", guard, is_true));
                            send(self, *then, g1, env.clone());
                            send(self, *else_, g2, env.clone());
                        }
                        _ => {
                            let g = self.choices(&guard, 2);
                            send(self, *then, g[0].clone(), env.clone());
                            send(self, *else_, g[1].clone(), env.clone());
                        }
                    }
                }
                Terminator::Call { callee, args, dest, ok, fail } => {
                    // What the call may have written: everything lent to it
                    // mutably, everything given away, and anything reachable
                    // through a borrow this function holds.
                    let mut after = env.clone();
                    for a in args {
                        match a {
                            Arg::Lend { place, mutable: true } | Arg::Give(place) => self.forget(&mut after, place.local),
                            _ => {}
                        }
                    }
                    self.forget_aliased(&mut after);
                    let mut e = after.clone();
                    if let Some(d) = dest {
                        // What a call gives back is not followed here; its
                        // type is all the query knows about it.
                        let ty = self.place_ty(d);
                        let v = int_shape(&ty).map(|(bits, signed)| {
                            let f = self.fresh("call", bits, signed, format!("what {} gave back", self.callee_name(callee)));
                            self.constrain_range(&f, &ty);
                            f
                        });
                        self.assign(&mut e, d, v);
                    }
                    let g = self.choices(&guard, 1 + fail.len());
                    send(self, *ok, g[0].clone(), e);
                    for (k, arm) in fail.iter().enumerate() {
                        let mut fe = after.clone();
                        if let Some(p) = &arm.payload {
                            self.forget(&mut fe, p.local);
                        }
                        send(self, arm.block, g[k + 1].clone(), fe);
                    }
                }
                Terminator::TaskOutcome { ok, fail, .. } => {
                    let g = self.choices(&guard, 1 + fail.len());
                    send(self, *ok, g[0].clone(), env.clone());
                    for (k, arm) in fail.iter().enumerate() {
                        let mut fe = env.clone();
                        if let Some(p) = &arm.payload {
                            self.forget(&mut fe, p.local);
                        }
                        send(self, arm.block, g[k + 1].clone(), fe);
                    }
                }
                Terminator::Whichever { arms, closed } => {
                    let g = self.choices(&guard, arms.len() + closed.iter().count());
                    for (k, arm) in arms.iter().enumerate() {
                        let mut ae = env.clone();
                        self.forget(&mut ae, arm.dest.local);
                        send(self, arm.block, g[k].clone(), ae);
                    }
                    if let Some(c) = closed {
                        send(self, *c, g[arms.len()].clone(), env.clone());
                    }
                }
                other => {
                    let succ = crate::mir::verify::successors(other);
                    let g = self.choices(&guard, succ.len());
                    for (k, t) in succ.into_iter().enumerate() {
                        send(self, t, g[k].clone(), env.clone());
                    }
                }
            }
        }
        Some(())
    }

    /// One state from several arriving ones: a value is whichever the path
    /// that got here left, and the block is reached if any path reached it.
    fn merge(&mut self, arrivals: &[(String, Env)]) -> (String, Env) {
        if arrivals.len() == 1 {
            return arrivals[0].clone();
        }
        let guard = self.define_bool(&format!("(or {})", arrivals.iter().map(|(g, _)| g.clone()).collect::<Vec<_>>().join(" ")));
        let mut env = Env::new();
        let mut names: Vec<Key> = arrivals.iter().flat_map(|(_, e)| e.keys().cloned()).collect();
        names.sort();
        names.dedup();
        for l in names {
            let mut holders: Vec<(String, Val)> = Vec::new();
            for (g, e) in arrivals {
                if let Some(v) = e.get(&l) {
                    holders.push((g.clone(), v.clone()));
                }
            }
            if holders.len() != arrivals.len() {
                continue;
            }
            let bits = holders[0].1.bits;
            if holders.iter().any(|(_, v)| v.bits != bits) {
                continue;
            }
            let mut term = holders.last().unwrap().1.term.clone();
            for (g, v) in holders.iter().rev().skip(1) {
                term = format!("(ite {} {} {})", g, v.term, term);
            }
            let signed = holders[0].1.signed;
            let v = self.define(bits, &term, signed);
            env.insert(l, v);
        }
        (guard, env)
    }

    fn assign(&mut self, env: &mut Env, place: &Place, v: Option<Val>) {
        // Whatever is written, every length known of this local is stale —
        // except for a store into one element: `'xs'[i] = v` leaves
        // every length the query tracks (fields only) as it was.
        if !place.proj.iter().any(|p| matches!(p, Proj::Index(_))) {
            self.forget_lengths(env, place.local);
        }
        if !place.proj.is_empty() {
            // Part of a value: what the whole holds is no longer described.
            env.remove(&Key::Local(place.local));
            return;
        }
        // A value is held at its local's width: a sum asked at its true
        // value lands in an `int64` as the machine stores it. One that does
        // not fit stopped at the landing's own site, so nothing after it
        // sees the difference.
        // And read by its local's sign: a bare 1 stored in a `uint64` is
        // unsigned from here on, or a later widening would sign-extend it.
        let v = v.map(|v| match int_shape(&self.func().locals[place.local.0 as usize].ty.clone()) {
            Some((bits, signed)) => self.resize(v, bits, signed),
            _ => v,
        });
        match v.or_else(|| self.unknown_of(place.local)) {
            Some(v) => {
                env.insert(Key::Local(place.local), v);
            }
            None => {
                env.remove(&Key::Local(place.local));
            }
        }
    }

    /// The local's value is unknown now, but not its lengths.
    fn forget_value(&mut self, env: &mut Env, l: LocalId) {
        match self.unknown_of(l) {
            Some(v) => {
                env.insert(Key::Local(l), v);
            }
            None => {
                env.remove(&Key::Local(l));
            }
        }
    }

    /// Nothing is known of this local any more: its value and every length
    /// of it. What a call wrote through a mutable lend, what a task holds,
    /// what assembly produced.
    fn forget(&mut self, env: &mut Env, l: LocalId) {
        self.forget_lengths(env, l);
        // Not removed: one unknown of its type, the same one for every read
        // that follows, so a later test of it (a loop's `'i' </== n`) still
        // constrains the value the site sees.
        match self.unknown_of(l) {
            Some(v) => {
                env.insert(Key::Local(l), v);
            }
            None => {
                env.remove(&Key::Local(l));
            }
        }
    }

    /// Narrow an unknown by what layer 2 proved about it at this block.
    fn assume_fact(&mut self, env: &mut Env, block: usize, l: LocalId) {
        let Some(crate::mir::domain::Domain::Ints(set)) = self.facts.get(&(self.f, block)).and_then(|m| m.get(&l)).cloned() else { return };
        let Some((lo, hi)) = set.bounds() else { return };
        let Some(v) = env.get(&Key::Local(l)).cloned() else { return };
        if let (Some(a), Some(b)) = (self.cmp_const(&v, "bvsge", lo), self.cmp_const(&v, "bvsle", hi)) {
            self.defs.push(format!("(assert (and {} {}))", a, b));
        }
    }

    /// A place, seen through the shared borrows that name it. A `for` over
    /// `lend 's'.scores` walks a temporary holding that borrow, and the
    /// temporary's length *is* the list's: the same list, not a second one
    /// that merely happens to be as long.
    fn through_borrows(&self, p: &Place) -> Place {
        let mut p = p.clone();
        for _ in 0..8 {
            let Some(target) = self.borrows.get(&p.local) else { break };
            let mut next = target.clone();
            next.proj.extend(p.proj.iter().cloned());
            p = next;
        }
        p
    }

    /// Anything a whole-number local's type allows, named after it.
    fn unknown_of(&mut self, l: LocalId) -> Option<Val> {
        let decl = &self.func().locals[l.0 as usize];
        let ty = decl.ty.clone();
        let name = decl.name.clone().unwrap_or_else(|| "a value".to_string());
        let (bits, signed) = int_shape(&ty)?;
        let v = self.fresh("in", bits, signed, name);
        self.constrain_range(&v, &ty);
        Some(v)
    }

    fn forget_lengths(&mut self, env: &mut Env, l: LocalId) {
        env.retain(|k, _| !matches!(k, Key::Len(x, _) if *x == l));
    }

    /// Everything that can be reached through a borrow held somewhere in
    /// this function may have changed behind the query's back.
    fn forget_aliased(&mut self, env: &mut Env) {
        for l in sorted(self.aliased.clone()) {
            self.forget(env, l);
        }
    }

    /// A length, the same one each time it is read until something could
    /// have changed it. Only a place reached by fields alone is remembered:
    /// `'xs'['i']` names a different element each time `'i'` moves.
    fn length(&mut self, op: &Op, args: &[Operand], env: &mut Env) -> Option<Val> {
        let key = args.first().and_then(|a| a.place()).map(|p| self.through_borrows(p)).and_then(|p| {
            let p = &p;
            let fields_only = p.proj.iter().all(|x| matches!(x, Proj::Field(_)));
            if fields_only && !self.aliased.contains(&p.local) {
                Some(Key::Len(p.local, format!("{:?}{:?}", p.proj, op)))
            } else {
                None
            }
        });
        if let Some(k) = &key {
            if let Some(v) = env.get(k) {
                return Some(v.clone());
            }
        }
        let v = self.fresh("len", 64, false, length_word(op).to_string());
        self.defs.push(format!("(assert (bvsge {} (_ bv0 64)))", v.term));
        if let Some(k) = key {
            env.insert(k, v.clone());
        }
        Some(v)
    }

    fn place_ty(&self, p: &Place) -> Ty {
        let base = self.func().locals[p.local.0 as usize].ty.clone();
        crate::mir::build::projected_ty(&base, &p.proj, &self.p.items)
    }

    fn operand(&mut self, o: &Operand, env: &Env) -> Option<Val> {
        match o {
            Operand::Const(Const::Int(v)) => {
                let bits = 64;
                Some(self.define(bits, &bv(*v, bits), true))
            }
            Operand::Const(Const::Bool(b)) => Some(self.define(1, &bv(*b as i128, 1), false)),
            Operand::Const(Const::Duration(v)) => Some(self.define(64, &bv(*v, 64), true)),
            Operand::Const(_) => None,
            Operand::Read(p) | Operand::Move(p) => {
                if !p.proj.is_empty() {
                    return None;
                }
                env.get(&Key::Local(p.local)).cloned()
            }
        }
    }

    fn rvalue(&mut self, rv: &Rvalue, env: &Env) -> Option<Val> {
        match rv {
            Rvalue::Use(o) => self.operand(o, env),
            Rvalue::Unary { op, operand } => {
                let v = self.operand(operand, env)?;
                match op {
                    UnOp::Neg => Some(self.define(v.bits, &format!("(bvneg {})", v.term), v.signed)),
                    UnOp::Not | UnOp::NotBit => Some(self.define(v.bits, &format!("(bvnot {})", v.term), v.signed)),
                }
            }
            Rvalue::Binary { op, lhs, rhs } => {
                let a = self.operand(lhs, env)?;
                let b = self.operand(rhs, env)?;
                // Plain math is exact (owner, 2026-09-23), each `div.X` and
                // `mod.X` rounds as named (`operators.md`), and a
                // comparison is of the two numbers: all of them are asked
                // at the true values — each operand widened by its own
                // sign, wide enough that nothing wraps. Asked at the
                // operands' width, `up[…] + 9e18` wrapped into a small
                // number and a landing that overflows read as proven.
                if matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div(_) | BinOp::Mod(_) | BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge) {
                    // Two bits past the wider operand: every answer that some
                    // 64-bit type holds is exact there, sums and products
                    // alike. A product past that wraps — but it is past the
                    // window too, and the program stops at that site before
                    // any later one sees it. (Twice the width was exact
                    // everywhere and sent the solvers into minutes of work.)
                    let widest = a.bits.max(b.bits);
                    let w = widest + 2;
                    if w > 256 {
                        return None;
                    }
                    let x = self.resize(a, w, true);
                    let y = self.resize(b, w, true);
                    let body = match op {
                        BinOp::Add => format!("(bvadd {} {})", x.term, y.term),
                        BinOp::Sub => format!("(bvsub {} {})", x.term, y.term),
                        BinOp::Mul => format!("(bvmul {} {})", x.term, y.term),
                        BinOp::Div(rounding) | BinOp::Mod(rounding) => {
                            // Truncated, the solvers' own, then one step
                            // where the rounding asks: up (`q + 1`, `r - y`)
                            // when the remainder and the divisor share a
                            // sign, down (`q - 1`, `r + y`) when not.
                            let zero = bv(0, w);
                            let q = format!("(bvsdiv {} {})", x.term, y.term);
                            let r = format!("(bvsrem {} {})", x.term, y.term);
                            let nz = format!("(not (= {r} {zero}))");
                            let up = format!("(= (bvslt {r} {zero}) (bvslt {} {zero}))", y.term);
                            let step = match rounding {
                                crate::ast::Round::Trunc => "false".to_string(),
                                crate::ast::Round::Floor | crate::ast::Round::Bare => format!("(and {nz} (not {up}))"),
                                crate::ast::Round::Ceil => format!("(and {nz} {up})"),
                                crate::ast::Round::Near => {
                                    // |r| against |y| - |r|, a tie to even.
                                    let abs = |t: &str| format!("(ite (bvslt {t} {zero}) (bvneg {t}) {t})");
                                    let (ar, ay) = (abs(&r), abs(&y.term));
                                    let rest = format!("(bvsub {ay} {ar})");
                                    let odd = format!("(= ((_ extract 0 0) {q}) #b1)");
                                    format!("(or (bvsgt {ar} {rest}) (and (= {ar} {rest}) {odd}))")
                                }
                            };
                            let one = bv(1, w);
                            if op.is_div() {
                                format!("(ite {step} (ite {up} (bvadd {q} {one}) (bvsub {q} {one})) {q})")
                            } else {
                                format!("(ite {step} (ite {up} (bvsub {r} {yt}) (bvadd {r} {yt})) {r})", yt = y.term)
                            }
                        }
                        _ => {
                            let pred = compare(op, true, &x.term, &y.term);
                            return Some(self.define(1, &format!("(ite {} (_ bv1 1) (_ bv0 1))", pred), false));
                        }
                    };
                    return Some(self.define(w, &body, true));
                }
                let (a, b) = self.same_width(a, b);
                // The operation's sign is its typed side's — a bare number
                // beside a `uint64` does not make it signed — and a shift's
                // is its left operand's alone (check claude, 2026-09-29:
                // `'d' shr 35` on a `uint64` was an arithmetic shift).
                let signed = match (op, lhs) {
                    (BinOp::Shl | BinOp::Shr, _) => a.signed,
                    (_, Operand::Const(_)) => b.signed,
                    _ => a.signed,
                };
                let w = a.bits;
                let body = match op {
                    BinOp::WrapAdd => format!("(bvadd {} {})", a.term, b.term),
                    BinOp::WrapSub => format!("(bvsub {} {})", a.term, b.term),
                    BinOp::WrapMul => format!("(bvmul {} {})", a.term, b.term),
                    BinOp::AndBit | BinOp::And => format!("(bvand {} {})", a.term, b.term),
                    BinOp::OrBit | BinOp::Or => format!("(bvor {} {})", a.term, b.term),
                    BinOp::Xor => format!("(bvxor {} {})", a.term, b.term),
                    BinOp::Shl => format!("(bvshl {} {})", a.term, b.term),
                    BinOp::Shr => format!("({} {} {})", if signed { "bvashr" } else { "bvlshr" }, a.term, b.term),
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                        let pred = compare(op, signed, &a.term, &b.term);
                        return Some(self.define(1, &format!("(ite {} (_ bv1 1) (_ bv0 1))", pred), false));
                    }
                    _ => return None,
                };
                Some(self.define(w, &body, signed))
            }
            Rvalue::Convert { to, operand, .. } => {
                let v = self.operand(operand, env)?;
                let (bits, signed) = int_shape(to)?;
                Some(self.resize(v, bits, signed))
            }
            _ => None,
        }
    }

    fn op(&mut self, op: &Op, args: &[Operand], env: &Env) -> Option<Val> {
        match op {
            // A length is never negative, and that is worth telling the
            // solver even when nothing else about it is known.
            _ => {
                let _ = (args, env);
                None
            }
        }
    }

    fn same_width(&mut self, a: Val, b: Val) -> (Val, Val) {
        if a.bits == b.bits {
            return (a, b);
        }
        if a.bits > b.bits {
            let signed = b.signed;
            let nb = self.resize(b, a.bits, signed);
            (a, nb)
        } else {
            let signed = a.signed;
            let na = self.resize(a, b.bits, signed);
            (na, b)
        }
    }

    fn resize(&mut self, v: Val, bits: u32, signed: bool) -> Val {
        if v.bits == bits {
            return Val { signed, ..v };
        }
        let body = if bits > v.bits {
            let by = bits - v.bits;
            if v.signed {
                format!("((_ sign_extend {}) {})", by, v.term)
            } else {
                format!("((_ zero_extend {}) {})", by, v.term)
            }
        } else {
            format!("((_ extract {} 0) {})", bits - 1, v.term)
        };
        self.define(bits, &body, signed)
    }

    /// The width at which a value can be read as a signed number without
    /// changing it: an unsigned one needs a bit more room for the sign.
    fn signed_width(v: &Val) -> u32 {
        v.bits + if v.signed { 0 } else { 1 }
    }

    /// Compare a value against a constant bound, at a width that holds
    /// both of them.
    ///
    /// A bound need not fit the value's own width: a `uint32` is asked
    /// whether it is below 0, an `int32` whether it is above 4294967295.
    /// Asked at the value's width the constant wraps — 4294967295 as 32
    /// bits *is* −1 — and the solver then answers truthfully about a
    /// different question. That is how a query could come back `unsat`
    /// because it was impossible rather than because the program was
    /// safe, and an impossible query reads as a proof. Widen first.
    fn cmp_const(&mut self, v: &Val, op: &str, bound: i128) -> Option<String> {
        let w = Self::signed_width(v).max(signed_bits(bound));
        if w > 256 {
            // Wider than the query can write a literal for. Nothing is
            // claimed rather than something unchecked.
            return None;
        }
        let ext = self.resize(v.clone(), w, true);
        Some(format!("({} {} {})", op, ext.term, bv(bound, w)))
    }

    /// The same, between two values.
    fn cmp_vals(&mut self, a: &Val, op: &str, b: &Val) -> Option<String> {
        let w = Self::signed_width(a).max(Self::signed_width(b));
        if w > 256 {
            return None;
        }
        let x = self.resize(a.clone(), w, true);
        let y = self.resize(b.clone(), w, true);
        Some(format!("({} {} {})", op, x.term, y.term))
    }

    fn constrain_range(&mut self, v: &Val, ty: &Ty) {
        if let Some((lo, hi)) = int_bounds(ty) {
            let (Some(l), Some(h)) = (self.cmp_const(v, "bvsge", lo), self.cmp_const(v, "bvsle", hi)) else {
                // Saying nothing about the value is sound: it allows more
                // than the type does, so nothing is proven that is not.
                return;
            };
            self.defs.push(format!("(assert (and {} {}))", l, h));
        }
    }

    /// The site's question: what makes this instruction wrong.
    fn bad_value(&mut self, id: SiteId, env: &Env) -> Option<String> {
        let site = self.func().sites[id.0 as usize].clone();
        let mut ops = Vec::new();
        for o in &site.operands {
            let v = match self.operand(o, env) {
                Some(v) => Some(v),
                // A whole number the query knows nothing about here is still
                // a whole number of its type: anything that type allows.
                None => match o.place() {
                    Some(p) if p.proj.is_empty() => {
                        let ty = self.place_ty(p);
                        int_shape(&ty).map(|(bits, signed)| {
                            let name = self.func().locals[p.local.0 as usize].name.clone().unwrap_or_else(|| "a value".to_string());
                            let v = self.fresh("in", bits, signed, name);
                            self.constrain_range(&v, &ty);
                            v
                        })
                    }
                    _ => None,
                },
            };
            ops.push(v);
        }
        // A literal takes its context's family here as everywhere: at the
        // result's width and sign, when the result's type can hold it.
        if let Some((bits, signed)) = site.result_ty.as_ref().and_then(int_shape) {
            let (lo, hi) = site.result_ty.as_ref().and_then(int_bounds).unwrap_or((i128::MIN, i128::MAX));
            for (i, o) in site.operands.iter().enumerate() {
                if let Operand::Const(Const::Int(v)) = o {
                    if *v >= lo && *v <= hi {
                        let lit = self.define(bits, &bv(*v, bits), signed);
                        if let Some(slot) = ops.get_mut(i) {
                            *slot = Some(lit);
                        }
                    }
                }
            }
        }
        match &site.kind {
            SiteKind::Overflow(op) => {
                let ty = site.result_ty.clone()?;
                let (bits, signed) = int_shape(&ty)?;
                let (lo, hi) = int_bounds(&ty)?;
                let a = ops.first().cloned().flatten()?;
                match op {
                    // When both operands are the result's own type — the usual
                    // case, a literal having taken its family — the solvers'
                    // own overflow predicates ask exactly the question, at the
                    // type's width. Widening a 64-bit multiply to 128 bits to
                    // ask it instead sent CVC5 into minutes of work its step
                    // budget did not count.
                    BinOp::Add | BinOp::Sub | BinOp::Mul if {
                        let b = ops.get(1).cloned().flatten();
                        // The exact type's range is not its width's: its
                        // question is the 64-bit window, asked the long way.
                        let full = !ty.is_exact() && crate::resolve::int_range(signed, bits as u16) == (lo, hi);
                        full && a.bits == bits && a.signed == signed && b.as_ref().map(|b| b.bits == bits && b.signed == signed).unwrap_or(false)
                    } =>
                    {
                        let b = ops.get(1).cloned().flatten()?;
                        let pred = match (op, signed) {
                            (BinOp::Add, true) => "bvsaddo",
                            (BinOp::Add, false) => "bvuaddo",
                            (BinOp::Sub, true) => "bvssubo",
                            (BinOp::Sub, false) => "bvusubo",
                            (_, true) => "bvsmulo",
                            (_, false) => "bvumulo",
                        };
                        Some(format!("({} {} {})", pred, a.term, b.term))
                    }
                    BinOp::Add | BinOp::Sub | BinOp::Mul => {
                        // Compute wide enough that the true answer fits, then
                        // ask whether it is outside what the type holds. Two
                        // bits more than the wider operand holds a sum of
                        // either signedness, and every product inside the
                        // question's range; a product past it is caught by the
                        // solvers' own overflow test at that width, which they
                        // answer quickly (a product at twice the width did not).
                        let b0 = ops.get(1).cloned().flatten()?;
                        let wide = a.bits.max(b0.bits).max(bits) + 2;
                        let a = self.resize(a, wide, true);
                        let b = self.resize(b0, wide, true);
                        let body = match op {
                            BinOp::Add => format!("(bvadd {} {})", a.term, b.term),
                            BinOp::Sub => format!("(bvsub {} {})", a.term, b.term),
                            _ => format!("(bvmul {} {})", a.term, b.term),
                        };
                        let r = self.define(wide, &body, true);
                        let past = if *op == BinOp::Mul { format!(" (bvsmulo {} {})", a.term, b.term) } else { String::new() };
                        Some(format!("(or {} {}{})", self.cmp_const(&r, "bvsgt", hi)?, self.cmp_const(&r, "bvslt", lo)?, past))
                    }
                    BinOp::Div(_) => {
                        // The one division that leaves the type: the lowest
                        // value by -1.
                        let b = ops.get(1).cloned().flatten()?;
                        let (a, b) = self.same_width(a, b);
                        Some(format!("(and (= {} {}) (= {} {}))", a.term, bv(lo, a.bits), b.term, bv(-1, b.bits)))
                    }
                    // An unsigned `mod.ceil` / `mod.near` leaves the type
                    // when it steps up: what is left is then below 0.
                    BinOp::Mod(rounding @ (crate::ast::Round::Ceil | crate::ast::Round::Near)) if !a.signed => {
                        let b = ops.get(1).cloned().flatten()?;
                        let (a, b) = self.same_width(a, b);
                        let zero = bv(0, a.bits);
                        let r = format!("(bvurem {} {})", a.term, b.term);
                        let nz = format!("(not (= {r} {zero}))");
                        if *rounding == crate::ast::Round::Ceil {
                            return Some(nz);
                        }
                        let rest = format!("(bvsub {} {r})", b.term);
                        let odd = format!("(= ((_ extract 0 0) (bvudiv {} {})) #b1)", a.term, b.term);
                        Some(format!("(and {nz} (or (bvugt {r} {rest}) (and (= {r} {rest}) {odd})))"))
                    }
                    _ => None,
                }
            }
            SiteKind::DivZero | SiteKind::ModZero => {
                let b = ops.first().cloned().flatten()?;
                Some(format!("(= {} {})", b.term, bv(0, b.bits)))
            }
            SiteKind::Bounds => {
                let i = ops.first().cloned().flatten()?;
                let len = ops.get(1).cloned().flatten()?;
                // 1-based: legal is 1 ..= len.
                Some(format!("(not (and {} {}))", self.cmp_const(&i, "bvsge", 1)?, self.cmp_vals(&i, "bvsle", &len)?))
            }
            SiteKind::NegativeExponent | SiteKind::FillCount => {
                let e = ops.first().cloned().flatten()?;
                self.cmp_const(&e, "bvslt", 0)
            }
            SiteKind::SliceRange => {
                let a = ops.first().cloned().flatten()?;
                let b = ops.get(1).cloned().flatten()?;
                let n = ops.get(2).cloned().flatten()?;
                // `a <= b + 1` one bit wider, so the + 1 cannot wrap.
                let w = Self::signed_width(&a).max(Self::signed_width(&b)) + 1;
                if w > 128 {
                    return None;
                }
                let (x, y) = (self.resize(a.clone(), w, true), self.resize(b.clone(), w, true));
                let not_backwards = format!("(bvsle {} (bvadd {} {}))", x.term, y.term, bv(1, w));
                Some(format!("(not (and {} {} {}))", self.cmp_const(&a, "bvsge", 1)?, not_backwards, self.cmp_vals(&b, "bvsle", &n)?))
            }
            SiteKind::Narrow { to } | SiteKind::LengthBound { to } => {
                // A duration's site carries both sides of a subtraction, and
                // what must fit is their difference: asking about the first
                // alone "proved" `'a' - 2 seconds` never negative.
                if *to == Ty::Duration || ops.len() > 1 {
                    return None;
                }
                let v = ops.first().cloned().flatten()?;
                let (lo, hi) = int_bounds(to)?;
                Some(format!("(or {} {})", self.cmp_const(&v, "bvsgt", hi)?, self.cmp_const(&v, "bvslt", lo)?))
            }
            // The count outside 0 to width - 1, for `shl` and `shr`; and for
            // `shl`, an answer that no longer fits the type — as layer 2
            // asks. Only the count was asked here, and a `shl` that moved
            // bits out of a uint32 in a loop was "proven" and wrapped at run
            // time (check claude, seed 9148). The width is the result
            // type's, not the operand's.
            SiteKind::ShiftWidth | SiteKind::ShiftCount => {
                let a = ops.first().cloned().flatten()?;
                let b = ops.get(1).cloned().flatten()?;
                let (bits, _) = site.result_ty.as_ref().and_then(int_shape)?;
                let (lo, hi) = site.result_ty.as_ref().and_then(int_bounds)?;
                let width = bits as i128;
                let count = format!("(or {} {})", self.cmp_const(&b, "bvsge", width)?, self.cmp_const(&b, "bvslt", 0)?);
                if matches!(site.kind, SiteKind::ShiftCount) {
                    return Some(count);
                }
                // Shifted wide enough that nothing falls off: the operand's
                // bits and the type's, plus the largest count that passes.
                let wide = a.bits.max(bits) + bits + 2;
                let x = self.resize(a, wide, true);
                let by = self.resize(b, wide, false);
                let r = self.define(wide, &format!("(bvshl {} {})", x.term, by.term), true);
                Some(format!("(or {} {} {})", count, self.cmp_const(&r, "bvsgt", hi)?, self.cmp_const(&r, "bvslt", lo)?))
            }
            _ => None,
        }
    }
}

fn compare(op: &BinOp, signed: bool, a: &str, b: &str) -> String {
    let p = |s: &str, u: &str| if signed { s.to_string() } else { u.to_string() };
    match op {
        BinOp::Eq => format!("(= {} {})", a, b),
        BinOp::Ne => format!("(not (= {} {}))", a, b),
        BinOp::Lt => format!("({} {} {})", p("bvslt", "bvult"), a, b),
        BinOp::Le => format!("({} {} {})", p("bvsle", "bvule"), a, b),
        BinOp::Gt => format!("({} {} {})", p("bvsgt", "bvugt"), a, b),
        BinOp::Ge => format!("({} {} {})", p("bvsge", "bvuge"), a, b),
        _ => "false".to_string(),
    }
}

/// Locals bound exactly once, to a shared borrow of a place reached by
/// fields alone, or to another such local. Anything bound twice may name a
/// different place the second time, so it is left out.
/// A set's locals in their order: the query is written the same way on
/// every run, so the solvers' step budgets end in the same place (a set's
/// own order changes from one process to the next, and a sieve proved in
/// 13 builds of 20).
fn sorted(set: std::collections::HashSet<LocalId>) -> Vec<LocalId> {
    let mut v: Vec<LocalId> = set.into_iter().collect();
    v.sort();
    v
}

fn shared_borrows(f: &Function) -> HashMap<LocalId, Place> {
    let mut count: HashMap<LocalId, u32> = HashMap::new();
    let mut target: HashMap<LocalId, Place> = HashMap::new();
    for b in &f.blocks {
        for s in &b.stmts {
            let (place, src) = match &s.kind {
                StmtKind::Assign { place, rvalue } => (place, match rvalue {
                    Rvalue::Lend { place: src, mutable: false } => Some(src.clone()),
                    Rvalue::Use(Operand::Read(src)) if src.proj.is_empty() && matches!(f.locals[src.local.0 as usize].kind, LocalKind::Lent { mutable: false }) => Some(src.clone()),
                    _ => None,
                }),
                StmtKind::Op { dest: Some(place), .. } => (place, None),
                _ => continue,
            };
            *count.entry(place.local).or_default() += 1;
            if let Some(src) = src {
                if place.proj.is_empty() && src.proj.iter().all(|x| matches!(x, Proj::Field(_))) {
                    target.insert(place.local, src);
                }
            }
        }
    }
    target.retain(|l, _| count.get(l) == Some(&1) && matches!(f.locals[l.0 as usize].kind, LocalKind::Lent { mutable: false }));
    target
}

/// Every local one block may write, by any means: assignment, an
/// operation's result or its target, a call's result or what it was lent
/// mutably or given, a task's captures, assembly, what a channel or a
/// failure hands over.
/// What a block may *resize*: `written_in` without the stores into one
/// element, which change no length the query tracks.
fn resized_in(b: &BasicBlock, out: &mut std::collections::HashSet<LocalId>) {
    let mut only = b.clone();
    only.stmts.retain(|s| !matches!(&s.kind, StmtKind::Assign { place, .. } if place.proj.iter().any(|p| matches!(p, Proj::Index(_)))));
    written_in(&only, out);
}

/// Lists and counters that move in lock-step round one loop: the body is
/// straight (no branch, no loop inside), and on each trip appends to the
/// list exactly once and adds 1 to the counter exactly once, with nothing
/// else resizing the list or writing the counter. Then the length minus the
/// counter never changes: `length = length on entry + (counter - counter on
/// entry)` at the head, on every trip. `while 'i' </== 'n' { list.append[…];
/// 'i' = 'i' + 1; }` builds a list of exactly n + 1.
fn lockstep_pairs(f: &Function, head: usize, body: &std::collections::HashSet<usize>) -> Vec<(LocalId, LocalId)> {
    let inside = |t: &BlockId| body.contains(&(t.0 as usize));
    for &b in body {
        let succ = crate::mir::verify::successors(&f.blocks[b].term);
        let n_in = succ.iter().filter(|t| inside(t)).count();
        if b != head && n_in != 1 {
            return Vec::new();
        }
        if b != head && succ.iter().any(|t| t.0 as usize == head) && succ.len() != 1 {
            return Vec::new();
        }
    }
    let mut appends: HashMap<LocalId, usize> = HashMap::new();
    let mut steps: HashMap<LocalId, usize> = HashMap::new();
    let mut plus_one: HashMap<LocalId, LocalId> = HashMap::new(); // temp -> counter
    let mut other_writes: std::collections::HashSet<LocalId> = std::collections::HashSet::new();
    for &b in body {
        if b == head {
            let mut w = std::collections::HashSet::new();
            written_in(&f.blocks[b], &mut w);
            other_writes.extend(w);
            continue;
        }
        let blk = &f.blocks[b];
        for s in &blk.stmts {
            match &s.kind {
                StmtKind::Assign { place, rvalue } if place.proj.is_empty() => match rvalue {
                    Rvalue::Binary { op: BinOp::Add, lhs: Operand::Read(c), rhs: Operand::Const(Const::Int(1)) } if c.proj.is_empty() => {
                        plus_one.insert(place.local, c.local);
                    }
                    Rvalue::Use(Operand::Read(t) | Operand::Move(t)) if t.proj.is_empty() && plus_one.get(&t.local) == Some(&place.local) => {
                        *steps.entry(place.local).or_default() += 1;
                    }
                    _ => {
                        other_writes.insert(place.local);
                    }
                },
                StmtKind::Op { op: Op::ListAppend, args, dest } => {
                    if let Some(d) = dest {
                        other_writes.insert(d.local);
                    }
                    match args.first().and_then(|a| a.place()) {
                        Some(p) if p.proj.is_empty() => *appends.entry(p.local).or_default() += 1,
                        Some(p) => {
                            other_writes.insert(p.local);
                        }
                        None => {}
                    }
                }
                _ => {
                    let mut w = std::collections::HashSet::new();
                    let one = BasicBlock { stmts: vec![s.clone()], term: Terminator::Unreachable, ..blk.clone() };
                    written_in(&one, &mut w);
                    other_writes.extend(w);
                }
            }
        }
        let mut w = std::collections::HashSet::new();
        let term_only = BasicBlock { stmts: Vec::new(), ..blk.clone() };
        written_in(&term_only, &mut w);
        other_writes.extend(w);
    }
    let mut out = Vec::new();
    for (list, n) in &appends {
        if *n != 1 || other_writes.contains(list) {
            continue;
        }
        for (c, k) in &steps {
            // The temporary holding `c + 1` is written too; only `c` itself
            // must have no other write.
            if *k == 1 && !other_writes.contains(c) && !appends.contains_key(c) {
                out.push((*list, *c));
            }
        }
    }
    out
}

/// Everything a block may write, for layer 2's loop shapes.
pub(crate) fn written_in_pub(b: &BasicBlock, out: &mut std::collections::HashSet<LocalId>) {
    written_in(b, out)
}

fn written_in(b: &BasicBlock, out: &mut std::collections::HashSet<LocalId>) {
    for s in &b.stmts {
        match &s.kind {
            StmtKind::Assign { place, .. } => {
                out.insert(place.local);
            }
            StmtKind::Op { dest, op, args } => {
                if let Some(d) = dest {
                    out.insert(d.local);
                }
                // The operations that change the collection they are handed
                // — the same list the walk itself forgets on. Reading a
                // list, printing one, measuring one changes nothing.
                if matches!(op, Op::ListAppend | Op::ListRemove | Op::MapSet | Op::MapRemove | Op::SimdStore | Op::SimdStoreSome) {
                    if let Some(p) = args.first().and_then(|a| a.place()) {
                        out.insert(p.local);
                    }
                }
            }
            StmtKind::Task { captures, .. } => {
                for (_, p) in captures {
                    out.insert(p.local);
                }
            }
            StmtKind::Asm { outs, lends, .. } => {
                for (p, _) in outs {
                    out.insert(p.local);
                }
                for (p, m, _) in lends {
                    if *m {
                        out.insert(p.local);
                    }
                }
            }
            _ => {}
        }
    }
    match &b.term {
        Terminator::Call { args, dest, fail, .. } => {
            if let Some(d) = dest {
                out.insert(d.local);
            }
            for a in args {
                match a {
                    Arg::Lend { place, mutable: true } | Arg::Give(place) => {
                        out.insert(place.local);
                    }
                    _ => {}
                }
            }
            for arm in fail {
                if let Some(p) = &arm.payload {
                    out.insert(p.local);
                }
            }
        }
        Terminator::TaskOutcome { fail, .. } => {
            for arm in fail {
                if let Some(p) = &arm.payload {
                    out.insert(p.local);
                }
            }
        }
        Terminator::Whichever { arms, .. } => {
            for arm in arms {
                out.insert(arm.dest.local);
            }
        }
        _ => {}
    }
}

/// How many bits two's complement needs to hold a number exactly.
fn signed_bits(v: i128) -> u32 {
    let mut n = 1u32;
    while n < 128 {
        let max = (1i128 << (n - 1)) - 1;
        let min = -(1i128 << (n - 1));
        if v >= min && v <= max {
            return n;
        }
        n += 1;
    }
    128
}

/// A bit-vector literal, two's complement at `bits`.
fn bv(v: i128, bits: u32) -> String {
    // Past 128 bits a negative literal is its 128-bit self, sign-extended:
    // the true product of two 64-bit values needs 129.
    if bits > 128 {
        return if v < 0 { format!("((_ sign_extend {}) (_ bv{} 128))", bits - 128, v as u128) } else { format!("(_ bv{} {})", v, bits) };
    }
    let mask: u128 = if bits >= 128 { u128::MAX } else { (1u128 << bits) - 1 };
    let raw = (v as u128) & mask;
    format!("(_ bv{} {})", raw, bits)
}

/// The width and signedness a type is held at, when it is a whole number.
pub fn int_shape(ty: &Ty) -> Option<(u32, bool)> {
    match ty.base() {
        Ty::Int(IntW::Fixed { signed, bits }) => Some((*bits as u32, *signed)),
        // A width Precog has not chosen is held at the machine's natural
        // one, which is what the lowering does with it too.
        Ty::Int(IntW::Free(_)) => Some((64, true)),
        Ty::Bool => Some((1, false)),
        Ty::Duration => Some((64, true)),
        _ => None,
    }
}

/// What a type holds, as two numbers.
pub fn int_bounds(ty: &Ty) -> Option<(i128, i128)> {
    if let Ty::Bounded { bound: Bound::Int { lo, hi }, .. } = ty {
        return Some((*lo, *hi));
    }
    if let Ty::Bounded { bound: Bound::Len { lo, hi, .. }, .. } = ty {
        return Some((*lo as i128, *hi as i128));
    }
    match ty.base() {
        Ty::Int(IntW::Fixed { signed, bits }) => Some(crate::resolve::int_range(*signed, *bits)),
        Ty::Int(IntW::Free(_)) => Some((i64::MIN as i128, i64::MAX as i128)),
        Ty::Bool => Some((0, 1)),
        Ty::Duration => Some((0, i64::MAX as i128)),
        _ => None,
    }
}
