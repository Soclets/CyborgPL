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
/// The largest question put to the solvers, in bytes of its text. Past it,
/// a question takes the solvers minutes inside their step budgets (check
/// claude, seed 22300: one of 13 MB, 140 s); the site is checked at run
/// time instead. A size, not a clock, so the same everywhere.
pub const QUERY_CAP: usize = 4_000_000;
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
        run(s, &Query { text: text.to_string(), assumptions: String::new(), truncated: false, assumed: false, reach: String::new(), inputs: Vec::new(), points: Vec::new() })
    }
}

fn run(s: &Solver, q: &Query) -> Answer {
    // The query goes in on stdin, so no file is left behind — a build that
    // is stopped halfway used to leave its queries in the temp directory,
    // some of them megabytes. `KAYO_KEEP_SMT=1` still writes each one out,
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
    if std::env::var("KAYO_KEEP_SMT").is_ok() {
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
    let started = std::time::Instant::now();
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
    // `KAYO_SMT_TIMES=1`: each question, how big, how long, what came back.
    if std::env::var("KAYO_SMT_TIMES").is_ok() {
        eprintln!("smt: {} {} bytes, {:.2} s, {}", s.name, q.text.len(), started.elapsed().as_secs_f64(), said.unwrap_or("no answer"));
    }
    match said {
        Some("unsat") => Answer::Unsat,
        Some("sat") => Answer::Sat(model_of(&text, q)),
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
fn model_of(text: &str, q: &Query) -> String {
    let mut vals = asked_values(text);
    if vals.is_empty() {
        vals = defined_values(text);
    }
    let mut out = Vec::new();
    // The copy of the site that went wrong, by its own operands: the
    // program's values there, not the many the query is built from
    // (design claude, 2026-10-26: 124 lines of "a value (13) = …").
    if let Some(pt) = q.points.iter().find(|pt| vals.get(&pt.bad).map(String::as_str) == Some("true")) {
        for i in &pt.operands {
            if let Some(n) = vals.get(&i.sym).and_then(|v| literal(v, i.signed)) {
                out.push(format!("{} = {}", i.show, n));
            }
        }
        if !out.is_empty() {
            return out.join(", ");
        }
    }
    for i in &q.inputs {
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
/// it: a `u8` of 200 is 200 and not −56.
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

/// A place reached by fields alone, as a key.
fn field_key(p: &Place) -> Option<String> {
    (!p.proj.is_empty() && p.proj.iter().all(|x| matches!(x, Proj::Field(_)))).then(|| format!("{:?}", p.proj))
}

/// What `Key::Pristine` holds: only that it is there.
fn pristine() -> Val {
    Val { term: "pristine".to_string(), bits: 0, signed: false }
}

/// A length, of a place the program names: "how many elements $xs has".
fn length_of(op: &Op, whose: &str) -> String {
    let w = length_word(op);
    for thing in ["the list", "the array", "the map", "the text"] {
        if w.contains(thing) {
            return w.replacen(thing, whose, 1);
        }
    }
    w.to_string()
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
    /// Landings before the site were assumed to have passed: an `unsat`
    /// is a proof only if the site can still be reached with them, which
    /// `reach` asks (check claude, 2026-10-15: an assumption that could
    /// never hold made every later site of a function "proven").
    pub assumed: bool,
    pub reach: String,
    /// What the query knows nothing about, and so what a counterexample
    /// is made of.
    pub inputs: Vec<Input>,
    /// Per copy of the site: the term saying it went wrong there, and the
    /// operands' terms as the program names them. A counterexample shows
    /// the copy the solver chose, in its own values.
    pub points: Vec<Point>,
}

pub struct Point {
    pub bad: String,
    pub operands: Vec<Input>,
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
    encode_opts(p, f, site, facts, true, false)
}

/// `encode`, saying whether the landings walked before the site may be
/// assumed to have passed, and whether a counting loop's counter may be
/// assumed to stay on its side of its first value. The second is asked
/// only when a question without it was not proved: it is true, and it
/// makes some questions too heavy to answer (bench/sieve's nested loops).
pub fn encode_opts(p: &Program, f: usize, site: SiteId, facts: &crate::precog::Facts, assume: bool, counted: bool) -> Option<Query> {
    encode_unrolled(p, f, site, facts, assume, counted, UNROLL_PER_LOOP)
}

/// `encode_opts`, with each loop walked `unroll` times before its head
/// stands for every later trip. At 0 the head stands for every trip, the
/// first too: a weaker question, and a much smaller one. Only its proofs
/// are worth having (design claude, 2026-10-26: std's `sits_at`, whose
/// counted loop's sixteen trips made questions CVC5 took 29 s over).
pub fn encode_unrolled(p: &Program, f: usize, site: SiteId, facts: &crate::precog::Facts, assume: bool, counted: bool, unroll: u32) -> Option<Query> {
    encode_with(p, f, site, facts, assume, counted, unroll, &[])
}

/// `encode_unrolled`, with loop invariants Spacer found assumed at their
/// heads.
pub fn encode_with_invariants(p: &Program, f: usize, site: SiteId, facts: &crate::precog::Facts, counted: bool, unroll: u32, invariants: &[Invariant]) -> Option<Query> {
    encode_with(p, f, site, facts, true, counted, unroll, invariants)
}

fn encoder<'a>(p: &'a Program, f: usize, facts: &'a crate::precog::Facts, assume: bool, counted: bool, unroll: u32, goal: Goal, invariants: &[Invariant]) -> Enc<'a> {
    let mut aliased = std::collections::HashSet::new();
    for b in &p.functions[f].blocks {
        for s in &b.stmts {
            if let StmtKind::Assign { rvalue: Rvalue::Lend { place, mutable: true }, .. } = &s.kind {
                aliased.insert(place.local);
            }
        }
    }
    let borrows = shared_borrows(&p.functions[f]);
    Enc { p, f, defs: Vec::new(), next: 0, truncated: false, reached: Vec::new(), inputs: Vec::new(), aliased, unexpressible: false, facts, borrows, entries: HashMap::new(), assume, counted, assumed: false, reach: Vec::new(), points: Vec::new(), operands: Vec::new(), entry_fields: HashMap::new(), field_places: HashMap::new(), unroll, goal, head_keys: Vec::new(), start_terms: Vec::new(), recorded: Vec::new(), invariants: invariants.to_vec(), sorts: HashMap::new(), cuts: HashMap::new(), arrivals: Vec::new() }
}

#[allow(clippy::too_many_arguments)]
fn encode_with(p: &Program, f: usize, site: SiteId, facts: &crate::precog::Facts, assume: bool, counted: bool, unroll: u32, invariants: &[Invariant]) -> Option<Query> {
    let mut e = encoder(p, f, facts, assume, counted, unroll, Goal::Site, invariants);
    e.build(site)?;
    if e.reached.is_empty() || e.unexpressible {
        return None;
    }
    let mut text = String::new();
    let _ = writeln!(text, "; CyborgPL — the prover layer 3, {}", describe_site(p, f, site));
    let _ = writeln!(text, "(set-option :rlimit {})", RLIMIT);
    let _ = writeln!(text, "(set-option :produce-models true)");
    let _ = writeln!(text, "(set-logic QF_BV)");
    for d in &e.defs {
        let _ = writeln!(text, "{}", d);
    }
    let assumptions = format!("{}(check-sat)\n", text);
    let reach = format!("{}(assert (or false {}))\n(check-sat)\n", text, e.reach.join(" "));
    let _ = writeln!(text, "(assert (or {}))", e.reached.join(" "));
    let _ = writeln!(text, "(check-sat)");
    // Only the inputs' values, never the whole model: a model of a query
    // with 95,000 definitions was printed as 3.9 GB, and reading it took
    // twenty minutes of a build (check claude's slow builds, 2026-09-29).
    let mut asked: Vec<String> = e.inputs.iter().map(|i| i.sym.clone()).collect();
    for pt in &e.points {
        asked.push(pt.bad.clone());
        asked.extend(pt.operands.iter().map(|o| o.sym.clone()));
    }
    if !asked.is_empty() {
        let _ = writeln!(text, "(get-value ({}))", asked.join(" "));
    }
    Some(Query { text, assumptions, truncated: e.truncated, assumed: e.assumed, reach, inputs: e.inputs, points: e.points })
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
    /// Whether a landing walked before the site may be assumed to have
    /// passed, and whether one was.
    assume: bool,
    counted: bool,
    assumed: bool,
    /// One term per copy of the site: it is reached, whatever the value.
    reach: Vec<String>,
    points: Vec<Point>,
    /// The last site asked about: its operands, named.
    operands: Vec<Input>,
    /// A parameter's fields as they were on entry, one unknown each, made
    /// when first read (design claude, 2026-10-26: `if $c.now at_least
    /// 2880 { return …; }` said nothing about the `$c.now + 15` after it).
    entry_fields: HashMap<(LocalId, String), Option<Val>>,
    /// Each field key's place, for a join that needs its entry value before
    /// anything has read it (a `while` testing a field it writes).
    field_places: HashMap<(LocalId, String), Place>,
    /// How many times each loop is walked before its head stands for the
    /// trips after.
    unroll: u32,
    /// What the walk is for: a site's question, or one of the three parts
    /// of a loop's Horn clauses for Spacer.
    goal: Goal,
    /// For Spacer's parts: the values the invariant is about, what each
    /// started as when the walk starts at the head, and each arrival at the
    /// head with its guard and those values.
    head_keys: Vec<HeadKey>,
    start_terms: Vec<String>,
    recorded: Vec<(String, Vec<String>)>,
    /// Invariants confirmed for loops of this function, assumed at their
    /// heads wherever a head stands for later trips.
    invariants: Vec<Invariant>,
    /// Each bit-vector symbol's width, and whether the value it stands for
    /// is read signed: what the integer clauses need to read it as a number.
    sorts: HashMap<String, (u32, bool)>,
    /// For `Goal::Cut`: the heads that are cut points, what each one's
    /// invariant is about, and every arrival at one (head, guard, terms).
    cuts: HashMap<usize, Vec<HeadKey>>,
    arrivals: Vec<(usize, String, Vec<String>)>,
}

/// What a walk of the flow graph is for. A site's question starts at the
/// function's entry and asks about the site; Spacer's three parts of one
/// loop are: what arrives at its head from outside it, what one trip round
/// it does, and what the site sees from its head on.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Goal {
    Site,
    Entry(usize),
    Trip(usize),
    FromHead(usize),
    /// One walk of a function's clauses with every loop head on the way to
    /// the site a cut point: from the entry (`usize::MAX`) or from one
    /// head, to every head it reaches next and to the site.
    Cut(usize),
}

/// One value a loop invariant is about: a whole-number local the loop
/// reads or writes, or a length it reads.
#[derive(Clone, Debug)]
struct HeadKey {
    key: Key,
    bits: u32,
    signed: bool,
}

/// A loop invariant Spacer found and both solvers confirmed: it holds on
/// every arrival at the loop's head (owner, 2026-10-25: measured before
/// deciding).
#[derive(Clone, Debug)]
pub struct Invariant {
    pub head: usize,
    keys: Vec<HeadKey>,
    /// Spacer's parameter list and body, as it wrote them.
    params: String,
    body: String,
    /// In the program's names, for the log.
    pub shown: String,
}

impl Invariant {
    fn define(&self) -> String {
        format!("(define-fun inv{} {} Bool {})", self.head, self.params, self.body)
    }
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
    /// A whole-number field of a parameter, reached by fields alone
    /// (`$c.now`), as written since the function was entered.
    Field(LocalId, String),
    /// The parameter's fields are still what they were on entry, except
    /// those a `Field` says were written since.
    Pristine(LocalId),
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
        self.sorts.insert(sym.clone(), (bits, signed));
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
        self.sorts.insert(name.clone(), (bits, signed));
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
        self.sorts.insert(sel.clone(), (16, false));
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

    /// The one statement that gives a temporary its value, if one does.
    fn made_by(&self, l: LocalId) -> Option<&'a StmtKind> {
        let mut from = None;
        for b in &self.func().blocks {
            for st in &b.stmts {
                let dest = match &st.kind {
                    StmtKind::Assign { place, .. } => Some(place),
                    StmtKind::Op { dest: Some(place), .. } => Some(place),
                    _ => None,
                };
                if dest.is_some_and(|d| d.local == l) {
                    if from.is_some() {
                        return None;
                    }
                    from = Some(&st.kind);
                }
            }
        }
        from
    }

    /// The named values a temporary was worked out from, through the
    /// operations and conversions that made it.
    fn made_from(&self, l: LocalId, depth: u32, out: &mut Vec<&'a Operand>) {
        if depth > 6 {
            return;
        }
        let from: Vec<&'a Operand> = match self.made_by(l) {
            Some(StmtKind::Assign { rvalue: Rvalue::Binary { lhs, rhs, .. }, .. }) => vec![lhs, rhs],
            Some(StmtKind::Assign { rvalue: Rvalue::Unary { operand, .. } | Rvalue::Convert { operand, .. } | Rvalue::Use(operand), .. }) => vec![operand],
            _ => return,
        };
        for o in from {
            let Some(p) = o.place() else { continue };
            if self.spell(o, 0).is_some() {
                out.push(o);
            } else if p.proj.is_empty() {
                self.made_from(p.local, depth + 1, out);
            }
        }
    }

    /// An operand as the program writes it: `$a.minutes`, or a temporary
    /// by what it was made from. None when it has no name to give.
    fn spell(&self, o: &Operand, depth: u32) -> Option<String> {
        let p = o.place()?;
        let f = self.func();
        let decl = &f.locals[p.local.0 as usize];
        let base = match &decl.name {
            Some(n) => format!("${}", n),
            // A temporary made once, from a value, a copy or a length.
            None if depth < 4 => match self.made_by(p.local)? {
                StmtKind::Assign { rvalue: Rvalue::Use(src), .. } => self.spell(src, depth + 1)?,
                StmtKind::Assign { rvalue: Rvalue::Duplicate(src), .. } => self.spell(&Operand::Read(src.clone()), depth + 1)?,
                StmtKind::Op { op, args, .. } if length_word(op) != "a length" => {
                    let whose = self.spell(args.first()?, depth + 1)?;
                    length_of(op, &whose)
                }
                _ => return None,
            },
            None => return None,
        };
        if p.proj.is_empty() {
            return Some(base);
        }
        let bare = Place { local: p.local, proj: Vec::new() };
        let whole = f.place_name(p, &self.p.items.structs);
        let head = f.place_name(&bare, &self.p.items.structs);
        Some(whole.replacen(&head, &base, 1))
    }

    /// What to call a callee in a counterexample.
    fn callee_name(&self, c: &Callee) -> String {
        match c {
            Callee::Fn(i) => self.p.functions[i.0 as usize].name.clone(),
            Callee::Extern(id) => self.p.items.externs[id.0 as usize].name.clone(),
            Callee::Value(_) => "that function".to_string(),
            Callee::Op(_) => "that operation".to_string(),
            Callee::Convert(t, _) => format!("that `to.{}`", crate::types::TySpelling { ty: t, structs: &self.p.items.structs, enums: &self.p.items.enums }),
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
            // Its fields, unless a name lends it mutably: a write through
            // that name would change them without saying so.
            if !self.aliased.contains(l) {
                env0.insert(Key::Pristine(*l), pristine());
            }
        }
        for inv in self.invariants.clone() {
            self.defs.push(inv.define());
        }
        // Spacer's parts start at the loop's head, or stop at it.
        let mut start = 0usize;
        match self.goal {
            Goal::Site => {}
            Goal::Entry(h) => self.head_keys = self.keys_of_loop(h),
            Goal::Cut(h) if h == usize::MAX => {}
            Goal::Trip(h) | Goal::FromHead(h) | Goal::Cut(h) => {
                self.head_keys = match self.goal {
                    Goal::Cut(_) => self.cuts.get(&h).cloned().unwrap_or_default(),
                    _ => self.keys_of_loop(h),
                };
                start = h;
                // Any state the head can be in: each value the invariant is
                // about unknown, the parameters' fields as they are at this
                // trip (unknown, the same at every read), and what layer 2
                // knows holds at the head on every trip.
                env0.retain(|k, _| matches!(k, Key::Pristine(_)));
                for hk in self.head_keys.clone() {
                    let v = self.fresh(if matches!(hk.key, Key::Len(..)) { "len" } else { "in" }, hk.bits, hk.signed, "a value at the loop's head".into());
                    match &hk.key {
                        Key::Local(l) => self.constrain_range(&v, &f.locals[l.0 as usize].ty.clone()),
                        Key::Len(..) => self.defs.push(format!("(assert (bvsge {} (_ bv0 64)))", v.term)),
                        _ => {}
                    }
                    self.start_terms.push(v.term.clone());
                    env0.insert(hk.key.clone(), v);
                }
                for hk in self.head_keys.clone() {
                    if let Key::Local(l) = hk.key {
                        self.assume_fact(&mut env0, h, l);
                    }
                }
            }
        }
        incoming.insert((start, 0), vec![("true".to_string(), env0)]);
        let mut queue = vec![(start, 0u32)];
        let mut unrollings = 0usize;
        // Only the blocks the site can be reached from: what comes after it
        // says nothing about it, and a loop there must not cut the question
        // (check claude, seed 28124: a site before any loop was "proved for
        // the first trips" because a later loop ran out of room).
        // Spacer's entry part goes to the ways into the loop's head from
        // outside it, never through the head; its trip part to the ways
        // round.
        let loop_body = |h: usize| -> std::collections::HashSet<usize> { back.iter().filter(|(_, hd)| *hd == h).flat_map(|(from, _)| crate::precog::natural_loop(f, h, *from)).collect() };
        let mut to_site: std::collections::HashSet<usize> = match self.goal {
            Goal::Site | Goal::FromHead(_) => f.blocks.iter().enumerate().filter(|(_, bb)| bb.stmts.iter().any(|st| matches!(st.kind, StmtKind::Site(id) if id == site))).map(|(i, _)| i).collect(),
            Goal::Entry(h) => {
                let body = loop_body(h);
                f.blocks.iter().enumerate().filter(|(i, bb)| !body.contains(i) && crate::mir::verify::successors(&bb.term).iter().any(|t| t.0 as usize == h)).map(|(i, _)| i).collect()
            }
            Goal::Trip(h) => back.iter().filter(|(_, hd)| *hd == h).map(|(from, _)| *from).collect(),
            Goal::Cut(_) => f
                .blocks
                .iter()
                .enumerate()
                .filter(|(_, bb)| bb.stmts.iter().any(|st| matches!(st.kind, StmtKind::Site(id) if id == site)) || crate::mir::verify::successors(&bb.term).iter().any(|t| self.cuts.contains_key(&(t.0 as usize))))
                .map(|(i, _)| i)
                .collect(),
        };
        let mut preds: HashMap<usize, Vec<usize>> = HashMap::new();
        for (i, bb) in f.blocks.iter().enumerate() {
            for t in crate::mir::verify::successors(&bb.term) {
                preds.entry(t.0 as usize).or_default().push(i);
            }
        }
        let mut stack: Vec<usize> = to_site.iter().copied().collect();
        while let Some(n) = stack.pop() {
            for &pb in preds.get(&n).into_iter().flatten() {
                if matches!(self.goal, Goal::Entry(h) if h == pb) || matches!(self.goal, Goal::Cut(_)) && self.cuts.contains_key(&pb) {
                    continue;
                }
                if to_site.insert(pb) {
                    stack.push(pb);
                }
            }
        }

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
                if ti >= f.blocks.len() || !to_site.contains(&ti) {
                    continue;
                }
                let is_back = back.contains(&(b, ti));
                let next_level = if is_back { level + 1 } else { level };
                if is_back && next_level > self.unroll {
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

        // Levels are shared, so an inner loop's trips ckit them too. An
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
                    let skip = (*head, self.unroll);
                    let mut got = std::collections::HashSet::new();
                    let mut stack = vec![(start, 0u32)];
                    while let Some(n) = stack.pop() {
                        if n == skip || !got.insert(n) {
                            continue;
                        }
                        stack.extend(next.get(&n).into_iter().flatten().copied());
                    }
                    got
                });
                if reach.contains(from) {
                    if seen.contains(&(*head, self.unroll)) {
                        widened.insert((*head, self.unroll));
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
            for (si, s) in stmts.iter().enumerate() {
                match &s.kind {
                    StmtKind::Assign { place, rvalue } => {
                        let v = self.rvalue(rvalue, &env);
                        self.assign(&mut env, place, v);
                        // A list written out holds as many as it lists. Only
                        // in the joint walks, which were measured with it;
                        // layer 3's own questions stay as they were.
                        if let (Goal::Cut(_), Rvalue::MakeList(items)) = (self.goal, rvalue) {
                            if place.proj.is_empty() && !self.aliased.contains(&place.local) {
                                let n = self.define(64, &bv(items.len() as i128, 64), false);
                                env.insert(Key::Len(place.local, format!("{:?}{:?}", Vec::<Proj>::new(), Op::ListLength)), n);
                            }
                        }
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
                            // A const array read in place: its length is
                            // known while building (std claude, 2026-10-21).
                            if *op == Op::ConstView && d.proj.is_empty() && !self.aliased.contains(&d.local) {
                                let n = match args.first() {
                                    Some(Operand::Const(Const::Array { items, .. })) => Some(items.len()),
                                    Some(Operand::Const(Const::Bytes(b))) => Some(b.len()),
                                    _ => None,
                                };
                                if let Some(n) = n {
                                    let v = self.define(64, &bv(n as i128, 64), false);
                                    let key = Key::Len(d.local, format!("{:?}{:?}", Vec::<Proj>::new(), Op::ListLength));
                                    env.insert(key, v);
                                }
                            }
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
                                self.reached.push(both.clone());
                                let operands = std::mem::take(&mut self.operands);
                                self.points.push(Point { bad: both, operands });
                                self.reach.push(guard.clone());
                            }
                            // A copy of the site the query cannot put into
                            // words. Leaving it out would ask only about the
                            // copies it can, and an `unsat` about some of the
                            // ways to reach a site is not a proof about it.
                            None => self.unexpressible = true,
                        }
                    }
                    // Another site on the way that keeps a number in its
                    // type: reaching the asked one means it passed (`precog.md`: "whether any input reaches the
                    // site"), since a check that fails stops the program and
                    // a proven one cannot fail — by induction on the first
                    // site to fail. A trusted one is never checked, so
                    // nothing follows from it (cord claude: `'pos' plus 'm'`
                    // wrapped round to 0 on an unrolled trip, then indexed).
                    StmtKind::Site(id) => {
                        let other = &self.func().sites[id.0 as usize];
                        let trusted = matches!(other.verdict, crate::mir::Verdict::Trusted) || matches!(other.tier, Some(crate::ast::Tier::Nocheck | crate::ast::Tier::NoChecks));
                        // Only a landing, which keeps a number inside its
                        // type: the rest add weight to every unrolled trip
                        // and take nothing round to a wrong value. A counting
                        // loop's step lands where the loop just saw room, so
                        // it says nothing new and is left out too.
                        let steps_counter = matches!(stmts.get(si + 1).map(|n| &n.kind), Some(StmtKind::Assign { place, .. }) if place.proj.is_empty() && self.func().fors.iter().any(|fl| fl.counter == place.local));
                        let landing = matches!(other.kind, SiteKind::Narrow { .. }) && !steps_counter;
                        if self.assume && !trusted && landing {
                            if let Some(bad) = self.bad_value(*id, &env) {
                                // Only while no copy of the asked site walked
                                // so far went wrong: past one that did, the
                                // program has stopped, and nothing after it
                                // is reached (`5 div 'v'` landing in a
                                // `u64` after the division by 'v' "proved"
                                // 'v' was never 0).
                                let ok = if self.reached.is_empty() { "true".to_string() } else { format!("(not (or false {}))", self.reached.join(" ")) };
                                self.defs.push(format!("(assert (=> (and {} {}) (not {})))", guard, ok, bad));
                                self.assumed = true;
                            }
                        }
                    }
                    _ => {}
                }
            }
            // Where it goes, and what each successor then knows.
            let term = self.func().blocks[b].term.clone();
            let mut send = |this: &mut Self, to: BlockId, mut g: String, mut e: Env| {
                let ti = to.0 as usize;
                let is_back = crate::precog::back_edges(this.func()).contains(&(b, ti));
                let lvl = if is_back { level + 1 } else { level };
                // Spacer's parts: an arrival at the head they are about is
                // what they record, and the walk stops there.
                match this.goal {
                    Goal::Entry(h) if ti == h => {
                        let terms = this.key_terms(&mut e, false);
                        this.recorded.push((g, terms));
                        return;
                    }
                    Goal::Trip(h) if ti == h && is_back => {
                        let terms = this.key_terms(&mut e, false);
                        this.recorded.push((g, terms));
                        return;
                    }
                    Goal::Cut(_) if this.cuts.contains_key(&ti) => {
                        let keys = this.cuts[&ti].clone();
                        let terms = this.terms_of(&mut e, &keys, false);
                        this.arrivals.push((ti, g, terms));
                        return;
                    }
                    _ => {}
                }
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
                if entering && lvl < this.unroll && widened.contains(&(ti, this.unroll)) {
                    if walked.contains(&(ti, this.unroll)) {
                        this.truncated = true;
                    } else {
                        let mut late = e.clone();
                        let before = e.clone();
                        let grows = resized.get(&ti).cloned().unwrap_or_default();
                        let body = bodies.get(&ti).cloned().unwrap_or_default();
                        for l in sorted(last_trip.get(&ti).cloned().unwrap_or_default()) {
                            this.forget_in_loop(&mut late, &body, l, grows.contains(&l));
                            this.assume_fact(&mut late, ti, l);
                        }
                        let written = last_trip.get(&ti).cloned().unwrap_or_default();
                        this.assume_counted(&mut late, ti, &written);
                        this.assume_kept_positions(&mut late, ti, &written, &body, &[before]);
                        this.forget_aliased(&mut late);
                        // Either the trips go on as walked, or this entry is
                        // one of the later ones the last copy stands for:
                        // one or the other, never both. Sharing the entry's
                        // own condition made both hold at once, and the join
                        // took the walked trip's values, so the later trips
                        // were never asked (check claude, corpus 71a8d5a2:
                        // `'i'` of -2 on an outer loop's third trip, after
                        // its inner loop had used the levels up).
                        let ways = this.choices(&g, 2);
                        g = ways[0].clone();
                        this.assume_invariant(&mut late, ti, &ways[1]);
                        incoming.entry((ti, this.unroll)).or_default().push((ways[1].clone(), late));
                    }
                }
                if entering {
                    for (list, _) in lockstep.get(&ti).cloned().unwrap_or_default() {
                        let _ = this.length(&Op::ListLength, &[Operand::Read(Place::local(list))], &mut e);
                    }
                    this.entries.entry(ti).or_default().push(e.clone());
                }
                if lvl == this.unroll {
                    if let Some(written) = last_trip.get(&ti).cloned() {
                        let grows = resized.get(&ti).cloned().unwrap_or_default();
                        let body = bodies.get(&ti).cloned().unwrap_or_default();
                        for l in &sorted(written.clone()) {
                            // Written only element by element: every length
                            // the query knows of it stays.
                            this.forget_in_loop(&mut e, &body, *l, grows.contains(l));
                            // What layer 2 knows holds on every trip through
                            // this head: `'i'` counts up from 1, so it is
                            // never below 1, whichever trip this stands for.
                            this.assume_fact(&mut e, ti, *l);
                        }
                        this.assume_counted(&mut e, ti, &written);
                        let entries = this.entries.get(&ti).cloned().unwrap_or_default();
                        this.assume_kept_positions(&mut e, ti, &written, &body, &entries);
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
                        this.assume_invariant(&mut e, ti, &g);
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
        // A field written on one way in and not another holds, on the
        // other, what it held on entry, when that way still knows it. When
        // it does not, no field of that parameter is known past here.
        let mut arrivals: Vec<(String, Env)> = arrivals.to_vec();
        let mut lost: Vec<LocalId> = Vec::new();
        for k in &names {
            let Key::Field(l, path) = k else { continue };
            for (_, e) in arrivals.iter_mut() {
                if e.contains_key(k) {
                    continue;
                }
                let place = self.field_places.get(&(*l, path.clone())).cloned();
                let entry = match (e.contains_key(&Key::Pristine(*l)), place) {
                    (true, Some(place)) => self.entry_field(&place),
                    _ => None,
                };
                match entry {
                    Some(v) => {
                        e.insert(k.clone(), v);
                    }
                    None => lost.push(*l),
                }
            }
        }
        for l in names {
            if let Key::Field(x, _) | Key::Pristine(x) = &l {
                if lost.contains(x) {
                    continue;
                }
            }
            if let Key::Pristine(_) = &l {
                if arrivals.iter().all(|(_, e)| e.contains_key(&l)) {
                    env.insert(l, pristine());
                }
                continue;
            }
            let mut holders: Vec<(String, Val)> = Vec::new();
            for (g, e) in &arrivals {
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
            // One whole-number field is what was written, the rest as they
            // were; any other write leaves no field known.
            let leaf = field_key(place).filter(|_| int_shape(&self.place_ty(place)).is_some());
            match leaf {
                Some(k) if env.contains_key(&Key::Pristine(place.local)) || env.contains_key(&Key::Field(place.local, k.clone())) => {
                    let ty = self.place_ty(place);
                    let v = match v {
                        Some(v) => {
                            let (bits, signed) = int_shape(&ty).unwrap_or((v.bits, v.signed));
                            Some(self.resize(v, bits, signed))
                        }
                        None => int_shape(&ty).map(|(bits, signed)| {
                            let show = self.spell(&Operand::Read(place.clone()), 0).unwrap_or_else(|| "a field".to_string());
                            let u = self.fresh("in", bits, signed, show);
                            self.constrain_range(&u, &ty);
                            u
                        }),
                    };
                    match v {
                        Some(v) => {
                            self.field_places.insert((place.local, k.clone()), place.clone());
                            env.insert(Key::Field(place.local, k), v);
                        }
                        None => self.forget_fields(env, place.local),
                    }
                }
                _ => self.forget_fields(env, place.local),
            }
            return;
        }
        self.forget_fields(env, place.local);
        // A value is held at its local's width: a sum asked at its true
        // value lands in an `i64` as the machine stores it. One that does
        // not fit stopped at the landing's own site, so nothing after it
        // sees the difference.
        // And read by its local's sign: a bare 1 stored in a `u64` is
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

    /// No field of this local is known any more.
    fn forget_fields(&mut self, env: &mut Env, l: LocalId) {
        env.retain(|k, _| !matches!(k, Key::Field(x, _) | Key::Pristine(x) if *x == l));
    }

    /// A loop that writes `l` only through whole-number fields (`change
    /// $c.now = …` in a `while`): those fields, at its head, are anything
    /// their types allow, and the parameter's other fields are what they
    /// were. Any other write to it, and nothing of its fields is known.
    fn forget_in_loop(&mut self, env: &mut Env, body: &std::collections::HashSet<usize>, l: LocalId, grows: bool) {
        let leaf = |this: &Self, st: &Statement| matches!(&st.kind, StmtKind::Assign { place, .. } if place.local == l && field_key(place).is_some() && int_shape(&this.place_ty(place)).is_some());
        let mut fields: Vec<Place> = Vec::new();
        // A whole-number field's write resizes nothing, whatever the walk's
        // own count of resized locals says.
        let mut other = !env.contains_key(&Key::Pristine(l));
        let mut blocks: Vec<usize> = body.iter().copied().collect();
        blocks.sort();
        for b in blocks {
            let block = &self.func().blocks[b];
            for st in &block.stmts {
                if let (true, StmtKind::Assign { place, .. }) = (leaf(self, st), &st.kind) {
                    if !fields.contains(place) {
                        fields.push(place.clone());
                    }
                }
            }
            let mut only = block.clone();
            only.stmts.retain(|st| !leaf(self, st));
            let mut w = std::collections::HashSet::new();
            written_in(&only, &mut w);
            other |= w.contains(&l);
        }
        if other || fields.is_empty() {
            if grows {
                self.forget(env, l);
            } else {
                self.forget_value(env, l);
            }
            return;
        }
        env.remove(&Key::Local(l));
        for p in fields {
            let ty = self.place_ty(&p);
            let Some((bits, signed)) = int_shape(&ty) else { continue };
            let show = self.spell(&Operand::Read(p.clone()), 0).unwrap_or_else(|| "a field".to_string());
            let v = self.fresh("in", bits, signed, show);
            self.constrain_range(&v, &ty);
            if let Some(k) = field_key(&p) {
                self.field_places.insert((l, k.clone()), p.clone());
                env.insert(Key::Field(l, k), v);
            }
        }
    }

    /// A parameter's field as it was on entry: anything its type allows,
    /// the same at every read.
    fn entry_field(&mut self, p: &Place) -> Option<Val> {
        let k = (p.local, field_key(p)?);
        if let Some(v) = self.entry_fields.get(&k) {
            return v.clone();
        }
        let ty = self.place_ty(p);
        let v = int_shape(&ty).map(|(bits, signed)| {
            let show = self.spell(&Operand::Read(p.clone()), 0).unwrap_or_else(|| "a field".to_string());
            let v = self.fresh("in", bits, signed, show);
            self.constrain_range(&v, &ty);
            v
        });
        self.entry_fields.insert(k, v.clone());
        v
    }

    /// The local's value is unknown now, but not its lengths.
    fn forget_value(&mut self, env: &mut Env, l: LocalId) {
        self.forget_fields(env, l);
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
        self.forget_fields(env, l);
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

    /// A counting loop's counter at its head, on any trip: never below its
    /// first value going up, never above it going down — when nothing in
    /// the loop writes the first value or the step, so their values here
    /// are the ones the counter started from (cord claude: `'part'[pos 'k'
    /// minus 'pos' plus 1]` in `for 'k' from 'pos' to …` was given a `'k'`
    /// below `'pos'`).
    fn assume_counted(&mut self, env: &mut Env, head: usize, written: &std::collections::HashSet<LocalId>) {
        if !self.counted {
            return;
        }
        let fors: Vec<_> = self.func().fors.iter().filter(|fl| fl.head.0 as usize == head).cloned().collect();
        for fl in fors {
            let (Some(start), Some(step)) = (fl.start.as_ref(), fl.step) else { continue };
            if written.contains(&step) || start.place().is_some_and(|p| !p.proj.is_empty() || written.contains(&p.local)) {
                continue;
            }
            let (Some(k), Some(first), Some(by)) = (env.get(&Key::Local(fl.counter)).cloned(), self.operand(start, env), env.get(&Key::Local(step)).cloned()) else { continue };
            let w = 130;
            let (k, first, by) = (self.resize(k, w, true), self.resize(first, w, true), self.resize(by, w, true));
            self.defs.push(format!("(assert (and (=> (bvsge {by} (_ bv0 {w})) (bvsge {k} {first})) (=> (bvslt {by} (_ bv0 {w})) (bvsle {k} {first}))))", by = by.term, k = k.term, first = first.term, w = w));
            // An assumption like the landings: a proof leaning on it is
            // checked for a site that can still be reached.
            self.assumed = true;
        }
    }

    /// A position kept from a walk over a list (design claude, 2026-10-26:
    /// `change $at = $i` in `for $i, $x in lend $xs`, then `$xs[pos: $at]`
    /// after it): a local the body sets only from the walk's counter, or
    /// from a copy of it, holds at the head either what it held on the way
    /// in or a position the walk was at, 1 to the length it took. The
    /// counter is inside that while the body runs: the head tested it.
    fn assume_kept_positions(&mut self, env: &mut Env, head: usize, written: &std::collections::HashSet<LocalId>, body: &std::collections::HashSet<usize>, entries: &[Env]) {
        if entries.is_empty() {
            return;
        }
        let f = self.func();
        let fors: Vec<_> = f.fors.iter().filter(|fl| fl.head.0 as usize == head && fl.start.is_none() && fl.step.is_none()).cloned().collect();
        for fl in fors {
            if written.contains(&fl.stop) || self.aliased.contains(&fl.counter) {
                continue;
            }
            // Every write in the body to each local: a copy of which local,
            // or None for anything else.
            let mut writes: HashMap<LocalId, Vec<Option<LocalId>>> = HashMap::new();
            let mut blocks: Vec<usize> = body.iter().copied().collect();
            blocks.sort();
            for b in blocks {
                for st in &f.blocks[b].stmts {
                    match &st.kind {
                        StmtKind::Assign { place, rvalue } => {
                            let from = match rvalue {
                                Rvalue::Use(Operand::Read(src) | Operand::Move(src)) if src.proj.is_empty() && place.proj.is_empty() => Some(src.local),
                                _ => None,
                            };
                            writes.entry(place.local).or_default().push(from);
                        }
                        StmtKind::Op { dest: Some(d), .. } => writes.entry(d.local).or_default().push(None),
                        _ => {}
                    }
                }
            }
            for l in written {
                writes.entry(*l).or_default();
            }
            // Copies of the counter, then copies of those.
            let mut sources = vec![fl.counter];
            for _ in 0..4 {
                let more: Vec<LocalId> = sorted(writes.keys().copied().collect())
                    .into_iter()
                    .filter(|l| !sources.contains(l) && !self.aliased.contains(l))
                    .filter(|l| writes[l].iter().all(|w| w.is_some_and(|s| sources.contains(&s))) && !writes[l].is_empty())
                    .collect();
                if more.is_empty() {
                    break;
                }
                sources.extend(more);
            }
            let Some(stop) = env.get(&Key::Local(fl.stop)).cloned() else { continue };
            for x in sources.into_iter().skip(1) {
                let Some(cur) = env.get(&Key::Local(x)).cloned() else { continue };
                let Some(was) = entries.iter().map(|e| e.get(&Key::Local(x)).cloned()).collect::<Option<Vec<Val>>>() else { continue };
                let w = 130;
                let (cs, ts) = (cur.signed, stop.signed);
                let cur = self.resize(cur, w, cs);
                let top = self.resize(stop.clone(), w, ts);
                let mut ways: Vec<String> = was.into_iter().map(|v| {
                    let v = self.resize(v.clone(), w, v.signed);
                    format!("(= {} {})", cur.term, v.term)
                }).collect();
                ways.push(format!("(and (bvsge {c} {one}) (bvsle {c} {top}))", c = cur.term, one = bv(1, w), top = top.term));
                self.defs.push(format!("(assert (or {}))", ways.join(" ")));
                self.assumed = true;
            }
        }
    }

    /// What a loop invariant for the loop at `head` is about: each named
    /// whole-number local the loop reads or writes, each one made before
    /// the loop and read in it (a `for`'s stop), the counters, and the
    /// lengths the loop reads. Values made and used within one trip are
    /// left out: no trip carries them to the next.
    fn keys_of_loop(&self, head: usize) -> Vec<HeadKey> {
        let f = self.func();
        let back = crate::precog::back_edges(f);
        let body: std::collections::BTreeSet<usize> = back.iter().filter(|(_, h)| *h == head).flat_map(|(from, _)| crate::precog::natural_loop(f, head, *from)).collect();
        let mut written = std::collections::HashSet::new();
        let mut mentioned: std::collections::BTreeSet<LocalId> = std::collections::BTreeSet::new();
        let mut lens: Vec<Key> = Vec::new();
        for &b in &body {
            let block = &f.blocks[b];
            written_in(block, &mut written);
            for st in &block.stmts {
                mentioned.extend(crate::mir::channels::stmt_reads(st));
                // A list the loop appends to or removes from: its length,
                // as the walk keeps it across the change.
                // (Integer clauses only: the bit-vector path stays as it was
                // measured, 2026-10-29.)
                if let (true, StmtKind::Op { op: Op::ListAppend | Op::ListRemove, args, .. }) = (spacer_integers(), &st.kind) {
                    if let Some(p) = args.first().and_then(|a| a.place()).filter(|p| p.proj.is_empty() && !self.aliased.contains(&p.local)) {
                        let k = Key::Len(p.local, format!("{:?}{:?}", Vec::<Proj>::new(), Op::ListLength));
                        if !lens.contains(&k) {
                            lens.push(k);
                        }
                    }
                }
                if let StmtKind::Op { op: op @ (Op::ListLength | Op::ArrayLength | Op::MapLength | Op::StrBytes | Op::StrPoints | Op::StrGraphemes), args, .. } = &st.kind {
                    if let Some(k) = self.len_key(op, args) {
                        if !lens.contains(&k) {
                            lens.push(k);
                        }
                    }
                }
            }
            mentioned.extend(crate::mir::channels::term_reads(&block.term));
        }
        mentioned.extend(written.iter().copied());
        let mut outside = std::collections::HashSet::new();
        for (i, block) in f.blocks.iter().enumerate() {
            if !body.contains(&i) {
                written_in(block, &mut outside);
            }
        }
        let counters: Vec<LocalId> = f.fors.iter().filter(|fl| fl.head.0 as usize == head).flat_map(|fl| [Some(fl.counter), Some(fl.stop), fl.step]).flatten().collect();
        let mut keys = Vec::new();
        for l in mentioned {
            let decl = &f.locals[l.0 as usize];
            let carried = decl.name.is_some() || counters.contains(&l) || (outside.contains(&l) && !written.contains(&l)) || f.params.contains(&l);
            if !carried || self.aliased.contains(&l) {
                continue;
            }
            if let Some((bits, signed)) = int_shape(&decl.ty) {
                keys.push(HeadKey { key: Key::Local(l), bits, signed });
            }
        }
        keys.extend(lens.into_iter().map(|key| HeadKey { key, bits: 64, signed: false }));
        keys
    }

    /// The terms of the values an invariant is about, in this state, each
    /// at its width. One the state does not know is anything at all; with
    /// `keep`, it is put into the state, so every later read agrees.
    fn key_terms(&mut self, env: &mut Env, keep: bool) -> Vec<String> {
        let keys = self.head_keys.clone();
        self.terms_of(env, &keys, keep)
    }

    fn terms_of(&mut self, env: &mut Env, keys: &[HeadKey], keep: bool) -> Vec<String> {
        let mut out = Vec::new();
        for hk in keys {
            let v = match env.get(&hk.key).cloned() {
                Some(v) => self.resize(v, hk.bits, hk.signed),
                None => {
                    let v = match &hk.key {
                        Key::Local(l) => self.unknown_of(*l).unwrap_or_else(|| self.fresh("in", hk.bits, hk.signed, "a value".into())),
                        Key::Len(..) => {
                            let v = self.fresh("len", 64, false, "a length".into());
                            self.defs.push(format!("(assert (bvsge {} (_ bv0 64)))", v.term));
                            v
                        }
                        _ => self.fresh("in", hk.bits, hk.signed, "a value".into()),
                    };
                    if keep {
                        env.insert(hk.key.clone(), v.clone());
                    }
                    self.resize(v, hk.bits, hk.signed)
                }
            };
            out.push(v.term);
        }
        out
    }

    /// A loop invariant both solvers confirmed holds on every arrival at
    /// its head, this one too (when the path here is the one taken).
    fn assume_invariant(&mut self, env: &mut Env, head: usize, guard: &str) {
        let Some(inv) = self.invariants.iter().find(|i| i.head == head).cloned() else { return };
        let terms = self.terms_of(env, &inv.keys, true);
        self.defs.push(format!("(assert (=> {} (inv{} {})))", guard, head, terms.join(" ")));
        // An assumption like the landings: a proof leaning on it is checked
        // for a site that can still be reached.
        self.assumed = true;
    }

    /// The key a length is remembered under, when it is one the query
    /// remembers.
    fn len_key(&self, op: &Op, args: &[Operand]) -> Option<Key> {
        args.first().and_then(|a| a.place()).map(|p| self.through_borrows(p)).and_then(|p| {
            let fields_only = p.proj.iter().all(|x| matches!(x, Proj::Field(_)));
            if fields_only && !self.aliased.contains(&p.local) {
                // An array's length is the one a list's would be: one key, or
                // `array.length` and a read's bounds were two unknowns (std
                // claude, 2026-10-21: "7 and 0 at once").
                let op = if *op == Op::ArrayLength { &Op::ListLength } else { op };
                Some(Key::Len(p.local, format!("{:?}{:?}", p.proj, op)))
            } else {
                None
            }
        })
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
        let key = self.len_key(op, args);
        if let Some(k) = &key {
            if let Some(v) = env.get(k) {
                return Some(v.clone());
            }
        }
        let whose = args.first().and_then(|o| self.spell(o, 0));
        let v = self.fresh("len", 64, false, match whose {
            Some(w) => length_of(op, &w),
            None => length_word(op).to_string(),
        });
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
            // At a width that holds it: 2^63, the exact `(-2^63) div.floor
            // (-1)` folded, read at 64 bits was -2^63 (check claude,
            // 2026-10-15), and a landing assumed passed on that value was
            // false, so every site after it was "proven".
            Operand::Const(Const::Int(v)) => {
                let need = if *v >= 0 { 129 - v.leading_zeros() } else { 129 - (!*v).leading_zeros() };
                let bits = need.max(64);
                Some(self.define(bits, &bv(*v, bits), true))
            }
            Operand::Const(Const::Bool(b)) => Some(self.define(1, &bv(*b as i128, 1), false)),
            Operand::Const(Const::Duration(v)) => Some(self.define(64, &bv(*v, 64), true)),
            Operand::Const(_) => None,
            Operand::Read(p) | Operand::Move(p) => {
                if !p.proj.is_empty() {
                    let k = field_key(p)?;
                    if let Some(v) = env.get(&Key::Field(p.local, k)) {
                        return Some(v.clone());
                    }
                    if env.contains_key(&Key::Pristine(p.local)) {
                        return self.entry_field(p);
                    }
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
                    // A bin's: not a bit-vector.
                    UnOp::NegIeee => None,
                }
            }
            Rvalue::Binary { op, lhs, rhs } => {
                let op = &op.plain();
                let a = self.operand(lhs, env)?;
                let b = self.operand(rhs, env)?;
                // Plain math is exact (owner, 2026-09-23), each `div.X` and
                // `mod.X` rounds as named (`operators.md`), and a
                // comparison is of the two numbers: all of them are asked
                // at the true values — each operand widened by its own
                // sign, wide enough that nothing wraps. Asked at the
                // operands' width, `up[…] + 9e18` wrapped into a small
                // number and a landing that overflows read as proven.
                if matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div(_) | BinOp::DivIeee(_) | BinOp::Mod(_) | BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge) {
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
                        BinOp::Div(rounding) | BinOp::DivIeee(rounding) | BinOp::Mod(rounding) => {
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
                // beside a `u64` does not make it signed — and a shift's
                // is its left operand's alone (check claude, 2026-09-29:
                // `'d' shr 35` on a `u64` was an arithmetic shift).
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
    /// A bound need not fit the value's own width: a `u32` is asked
    /// whether it is below 0, an `i32` whether it is above 4294967295.
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
        // What a counterexample shows: each operand that is not written out
        // in the code already, as the program names it.
        self.operands.clear();
        let sides = site.operands.len();
        for (n, o) in site.operands.iter().enumerate() {
            let (Some(v), Some(p)) = (ops.get(n).cloned().flatten(), o.place()) else { continue };
            if let Some(show) = self.spell(o, 0) {
                self.operands.push(Input { sym: v.term.clone(), show, signed: v.signed });
                continue;
            }
            // A temporary an operation made just before (`$departure +
            // $a.minutes` landing in an `i32`): what it was made from.
            let mut made = Vec::new();
            if p.proj.is_empty() {
                self.made_from(p.local, 0, &mut made);
            }
            let before = self.operands.len();
            for m in made {
                if let (Some(show), Some(mv)) = (self.spell(m, 0), self.operand(m, env)) {
                    self.operands.push(Input { sym: mv.term, show, signed: mv.signed });
                }
            }
            if self.operands.len() == before {
                // Nothing it was made from is known here: the value itself,
                // spelled as it was worked out (`1290 + $a.minutes`).
                let show = match self.made_by(p.local) {
                    Some(StmtKind::Assign { rvalue: Rvalue::Binary { op, lhs, rhs }, .. }) if p.proj.is_empty() => {
                        let side = |o: &Operand| match o {
                            Operand::Const(Const::Int(k)) => k.to_string(),
                            _ => self.spell(o, 0).unwrap_or_else(|| "…".to_string()),
                        };
                        format!("{} {} {}", side(lhs), op.spelled(), side(rhs))
                    }
                    _ => match (sides, n) {
                        (2, 0) => "the left side",
                        (2, _) => "the right side",
                        _ => "the value",
                    }
                    .to_string(),
                };
                self.operands.push(Input { sym: v.term.clone(), show, signed: v.signed });
            }
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
                        let full = !ty.is_exact() && crate::resolve::int_range(signed, bits as u32) == (lo, hi);
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
                    BinOp::Div(_) | BinOp::DivIeee(_) => {
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
            SiteKind::Position => {
                let e = ops.first().cloned().flatten()?;
                self.cmp_const(&e, "bvslt", 1)
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
            // bits out of a u32 in a loop was "proven" and wrapped at run
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

/// Spacer's step budget, in Z3's own `rlimit` steps and never a clock, so
/// the same build proves the same sites every time (owner, 2026-10-25:
/// Spacer measured before deciding; the budget is the owner's call).
pub const SPACER_RLIMIT: u64 = 10_000_000;

/// The budget a build uses: `SPACER_RLIMIT`, or `KAYO_SPACER_RLIMIT` for
/// measuring others (still a step count, so still the same every time).
pub fn spacer_rlimit() -> u64 {
    std::env::var("KAYO_SPACER_RLIMIT").ok().and_then(|v| v.replace('_', "").parse().ok()).unwrap_or(SPACER_RLIMIT)
}

/// Spacer, Z3's Horn clause engine, is Precog's second way to find a
/// loop's invariant, on by default; `KAYO_NO_SPACER` turns it off (owner,
/// 2026-10-25: "yes"; whether it stays is decided from the measurement).
pub fn spacer_on() -> bool {
    std::env::var("KAYO_NO_SPACER").is_err()
}

/// What Spacer made of one loop, for one site.
#[derive(Debug, Clone)]
pub enum Spacer {
    /// It found an invariant, and both solvers confirmed it.
    Confirmed(Invariant),
    /// It found one, and a solver did not confirm it: never used.
    NotConfirmed { shown: String, why: String },
    /// It spent its step budget.
    Budget,
    /// It found a way through its clauses to the bad value: no invariant
    /// of this loop alone proves the site.
    Refuted,
    /// It was not asked, and why.
    Unasked(String),
}

/// The heads of the loops on some way from the function's entry to the
/// site: the loops whose invariants the site's question may need.
pub fn loops_before(p: &Program, f: usize, site: SiteId) -> Vec<usize> {
    let func = &p.functions[f];
    let back = crate::precog::back_edges(func);
    let mut heads: Vec<usize> = back.iter().map(|(_, h)| *h).collect();
    heads.sort();
    heads.dedup();
    let mut to_site: std::collections::HashSet<usize> = func.blocks.iter().enumerate().filter(|(_, bb)| bb.stmts.iter().any(|st| matches!(st.kind, StmtKind::Site(id) if id == site))).map(|(i, _)| i).collect();
    let mut preds: HashMap<usize, Vec<usize>> = HashMap::new();
    for (i, bb) in func.blocks.iter().enumerate() {
        for t in crate::mir::verify::successors(&bb.term) {
            preds.entry(t.0 as usize).or_default().push(i);
        }
    }
    let mut stack: Vec<usize> = to_site.iter().copied().collect();
    while let Some(n) = stack.pop() {
        for &pb in preds.get(&n).into_iter().flatten() {
            if to_site.insert(pb) {
                stack.push(pb);
            }
        }
    }
    heads.retain(|h| to_site.contains(h));
    heads
}

/// Ask Spacer for an invariant of the loop at `head` strong enough for the
/// site, then have both solvers confirm it is one: it holds on every way
/// into the loop, and one trip round keeps it. Spacer only proposes; a
/// bug in it can make the confirmation fail, never a wrong proof.
///
/// The clauses come from three walks of the flow graph with every loop's
/// head standing for every trip (as the shallow question does): from the
/// entry to the ways into this loop, round it once from its head, and
/// from its head to the site.
pub fn spacer(p: &Program, f: usize, site: SiteId, facts: &crate::precog::Facts, head: usize, portfolio: &Portfolio) -> Spacer {
    let Some(z3) = portfolio.members.iter().find(|s| s.name == "Z3") else { return Spacer::Unasked("Z3 is not here".into()) };
    if !portfolio.is_complete() {
        return Spacer::Unasked("only one solver is here, and an invariant is confirmed by both".into());
    }
    let none = SiteId(u32::MAX);
    let mut entry = encoder(p, f, facts, true, false, 0, Goal::Entry(head), &[]);
    // A loop entered where the function starts, or with no way in found,
    // would leave the first condition asked about nothing, and an
    // invariant confirmed by that would be confirmed by default.
    if head == 0 || entry.build(none).is_none() || entry.truncated || entry.recorded.is_empty() {
        return Spacer::Unasked("the ways into the loop could not all be put into words".into());
    }
    let keys = entry.head_keys.clone();
    if keys.is_empty() {
        return Spacer::Unasked("the loop has no whole number for an invariant to be about".into());
    }
    let mut trip = encoder(p, f, facts, true, false, 0, Goal::Trip(head), &[]);
    if trip.build(none).is_none() || trip.truncated {
        return Spacer::Unasked("a trip round the loop could not be put into words".into());
    }
    let mut from = encoder(p, f, facts, true, false, 0, Goal::FromHead(head), &[]);
    if from.build(site).is_none() || from.reached.is_empty() || from.unexpressible {
        return Spacer::Unasked("the site could not be asked about from the loop's head".into());
    }
    let integers = spacer_integers();
    let sorts: Vec<String> = keys.iter().map(|k| if integers { "Int".to_string() } else { format!("(_ BitVec {})", k.bits) }).collect();
    let mut text = String::new();
    let _ = writeln!(text, "; CyborgPL — Precog's Horn clauses for one loop, {}", describe_site(p, f, site));
    let _ = writeln!(text, "(set-option :rlimit {})", spacer_rlimit());
    let _ = writeln!(text, "(set-logic HORN)");
    // Spacer's global guidance (`KAYO_SPACER_GLOBAL`, for measuring): it
    // generalises across a loop's bounds where it would count up to them.
    if std::env::var("KAYO_SPACER_GLOBAL").is_ok() {
        let _ = writeln!(text, "(set-option :fp.spacer.global true)");
    }
    let _ = writeln!(text, "(declare-fun Inv ({}) Bool)", sorts.join(" "));
    if integers {
        int_clauses(&mut text, &keys, &entry, &trip, &from);
    } else {
        let (vars, conj) = horn_parts(&entry.defs);
        for (g, terms) in &entry.recorded {
            text.push_str(&horn_rule(&vars, &conj, &[g.clone()], &format!("(Inv {})", terms.join(" "))));
        }
        let (vars, conj) = horn_parts(&trip.defs);
        let at_head = format!("(Inv {})", trip.start_terms.join(" "));
        for (g, terms) in &trip.recorded {
            text.push_str(&horn_rule(&vars, &conj, &[at_head.clone(), g.clone()], &format!("(Inv {})", terms.join(" "))));
        }
        let (vars, conj) = horn_parts(&from.defs);
        let bad = format!("(or false {})", from.reached.join(" "));
        text.push_str(&horn_rule(&vars, &conj, &[format!("(Inv {})", from.start_terms.join(" ")), bad], "false"));
    }
    let _ = writeln!(text, "(check-sat)\n(get-model)\n(get-info :reason-unknown)");
    if text.len() > QUERY_CAP {
        return Spacer::Unasked(format!("its clauses came out too big to ask ({} MB)", text.len() / 1_000_000));
    }
    let out = run_raw(z3, &text);
    let said = out.lines().map(str::trim).find(|l| matches!(*l, "sat" | "unsat" | "unknown"));
    match said {
        Some("sat") => {}
        Some("unsat") => return Spacer::Refuted,
        Some("unknown") if out.contains("resource") || out.contains("canceled") => return Spacer::Budget,
        _ => return Spacer::Unasked(format!("Spacer did not say: {}", out.lines().last().unwrap_or("").trim())),
    }
    let Some(at) = out.find("(define-fun Inv ") else { return Spacer::Unasked("Spacer gave no invariant".into()) };
    let b = out.as_bytes();
    let Some((params, i)) = read_sexp(b, at + "(define-fun Inv ".len()) else { return Spacer::Unasked("Spacer's invariant could not be read".into()) };
    let Some((sort, i)) = read_sexp(b, i) else { return Spacer::Unasked("Spacer's invariant could not be read".into()) };
    let Some((body, _)) = read_sexp(b, i) else { return Spacer::Unasked("Spacer's invariant could not be read".into()) };
    if sort != "Bool" {
        return Spacer::Unasked("Spacer's invariant could not be read".into());
    }
    let shown = show_invariant(p, f, &keys, &body);
    // An integer invariant is read back into bit-vectors, and confirmed
    // there like any other.
    let (params, body) = if integers {
        let names: Vec<String> = match crate::precog::ints::parse(&params) {
            Some(crate::precog::ints::Sx::List(ps)) => ps.iter().filter_map(|p| match p {
                crate::precog::ints::Sx::List(pair) => match pair.first() {
                    Some(crate::precog::ints::Sx::Atom(n)) => Some(n.clone()),
                    _ => None,
                },
                _ => None,
            }).collect(),
            _ => Vec::new(),
        };
        if names.len() != keys.len() {
            return Spacer::NotConfirmed { shown, why: "its parameters could not be read".into() };
        }
        let typed: Vec<(String, u32, bool)> = names.iter().zip(&keys).map(|(n, k)| (n.clone(), k.bits, k.signed)).collect();
        let Some(bits) = crate::precog::ints::parse(&body).and_then(|b| crate::precog::ints::to_bits(&b, &typed)) else {
            return Spacer::NotConfirmed { shown, why: "it could not be read back into bit-vectors".into() };
        };
        (format!("({})", typed.iter().map(|(n, b, _)| format!("({} (_ BitVec {}))", n, b)).collect::<Vec<_>>().join(" ")), bits)
    } else {
        (params, body)
    };
    let inv = Invariant { head, keys, params, body, shown: shown.clone() };
    // Confirmed: on every way in, and kept by every way round.
    let header = format!("(set-option :rlimit {})\n(set-logic QF_BV)\n", RLIMIT);
    let fails = |recorded: &[(String, Vec<String>)]| -> String {
        let ways: Vec<String> = recorded.iter().map(|(g, t)| format!("(and {} (not (inv{} {})))", g, head, t.join(" "))).collect();
        format!("(assert (or false {}))\n(check-sat)\n", ways.join(" "))
    };
    let base = format!("{}{}\n{}\n{}", header, entry.defs.join("\n"), inv.define(), fails(&entry.recorded));
    let step = format!("{}{}\n{}\n(assert (inv{} {}))\n{}", header, trip.defs.join("\n"), inv.define(), head, trip.start_terms.join(" "), fails(&trip.recorded));
    for (what, q) in [("the ways into the loop", base), ("a trip round it", step)] {
        let (_, each) = portfolio.ask(&Query { text: q, assumptions: String::new(), truncated: false, assumed: false, reach: String::new(), inputs: Vec::new(), points: Vec::new() });
        if !each.iter().all(|(_, a)| *a == Answer::Unsat) {
            let said: Vec<String> = each.iter().map(|(n, a)| format!("{} {}", n, match a { Answer::Unsat => "confirmed it", Answer::Sat(_) => "found it false", Answer::Unknown => "could not say" })).collect();
            return Spacer::NotConfirmed { shown, why: format!("on {}: {}", what, said.join(", ")) };
        }
    }
    Spacer::Confirmed(inv)
}

/// Every loop on the way to the site as one system of Horn clauses (owner,
/// 2026-10-29, measured beside one loop at a time; `KAYO_SPACER_JOINT`,
/// over the integers): each loop head a cut point with its own invariant,
/// one walk from the entry and one from each head to the next heads and to
/// the site. Spacer then finds the invariants together, as a site after
/// three loops needs (the first one's length, kept by the second, bounding
/// the third). Every invariant is confirmed in bit-vectors by Z3 and CVC5
/// together: each walk, starting from its head's invariant, keeps every
/// head's it arrives at, and the entry's walk gives them.
pub fn spacer_joint(p: &Program, f: usize, site: SiteId, facts: &crate::precog::Facts, heads: &[usize], portfolio: &Portfolio) -> Result<Vec<Invariant>, Spacer> {
    use crate::precog::ints;
    let Some(z3) = portfolio.members.iter().find(|s| s.name == "Z3") else { return Err(Spacer::Unasked("Z3 is not here".into())) };
    if !portfolio.is_complete() {
        return Err(Spacer::Unasked("only one solver is here, and an invariant is confirmed by both".into()));
    }
    if heads.is_empty() || heads.contains(&0) {
        return Err(Spacer::Unasked("the ways into the loops could not all be put into words".into()));
    }
    let probe = encoder(p, f, facts, true, false, 0, Goal::Site, &[]);
    let cuts: HashMap<usize, Vec<HeadKey>> = heads.iter().map(|h| (*h, probe.keys_of_loop(*h))).collect();
    // The walks: from the entry, then from each head.
    let mut walks = Vec::new();
    for start in std::iter::once(usize::MAX).chain(heads.iter().copied()) {
        let mut e = encoder(p, f, facts, true, false, 0, Goal::Cut(start), &[]);
        e.cuts = cuts.clone();
        if e.build(site).is_none() || e.truncated || e.unexpressible {
            return Err(Spacer::Unasked("a walk between the loops could not be put into words".into()));
        }
        walks.push((start, e));
    }
    if walks.iter().all(|(_, e)| e.reached.is_empty()) {
        return Err(Spacer::Unasked("the site could not be asked about from the loops".into()));
    }
    let mut text = String::new();
    let _ = writeln!(text, "; CyborgPL — Precog's Horn clauses for every loop on the way, {}", describe_site(p, f, site));
    let _ = writeln!(text, "(set-option :rlimit {})", spacer_rlimit());
    let _ = writeln!(text, "(set-logic HORN)");
    if std::env::var("KAYO_SPACER_GLOBAL").is_ok() {
        let _ = writeln!(text, "(set-option :fp.spacer.global true)");
    }
    for h in heads {
        let _ = writeln!(text, "(declare-fun Inv{} ({}) Bool)", h, cuts[h].iter().map(|_| "Int").collect::<Vec<_>>().join(" "));
    }
    let read = |s: &str| ints::parse(s).unwrap_or(ints::Sx::Atom("true".into()));
    for (start, e) in &walks {
        let (mut vars, conj, mut tr) = ints::part(&e.defs, &e.sorts);
        let args = |tr: &mut ints::Ints, terms: &[String], keys: &[HeadKey]| -> String {
            terms.iter().zip(keys).map(|(t, k)| tr.number(&read(t), k.signed).unwrap_or_else(|| tr.fresh_int())).collect::<Vec<_>>().join(" ")
        };
        let from = if *start == usize::MAX { None } else { Some(format!("(Inv{} {})", start, args(&mut tr, &e.start_terms, &cuts[start]))) };
        let mut rules = Vec::new();
        for (to, g, terms) in &e.arrivals {
            let g = tr.boolean(&read(g)).unwrap_or_else(|| "true".into());
            let a = args(&mut tr, terms, &cuts[to]);
            rules.push((from.iter().cloned().chain([g]).collect::<Vec<_>>(), format!("(Inv{} {})", to, a)));
        }
        if !e.reached.is_empty() {
            let bad: Vec<String> = e.reached.iter().filter_map(|r| tr.boolean(&read(r))).collect();
            rules.push((from.iter().cloned().chain([format!("(or false {})", bad.join(" "))]).collect(), "false".into()));
        }
        vars.extend(tr.unknowns.drain(..));
        for (extra, head) in rules {
            text.push_str(&horn_rule(&vars, &conj, &extra, &head));
        }
    }
    let _ = writeln!(text, "(check-sat)\n(get-model)\n(get-info :reason-unknown)");
    if text.len() > QUERY_CAP {
        return Err(Spacer::Unasked(format!("its clauses came out too big to ask ({} MB)", text.len() / 1_000_000)));
    }
    let out = run_raw(z3, &text);
    match out.lines().map(str::trim).find(|l| matches!(*l, "sat" | "unsat" | "unknown")) {
        Some("sat") => {}
        Some("unsat") => return Err(Spacer::Refuted),
        Some("unknown") if out.contains("resource") || out.contains("canceled") => return Err(Spacer::Budget),
        _ => return Err(Spacer::Unasked(format!("Spacer did not say: {}", out.lines().last().unwrap_or("").trim()))),
    }
    // Each head's invariant, read back into bit-vectors.
    let mut invs = Vec::new();
    let mut shown_all = Vec::new();
    for h in heads {
        let keys = cuts[h].clone();
        let tag = format!("(define-fun Inv{} (", h);
        let Some(at) = out.find(&tag) else { return Err(Spacer::Unasked("Spacer gave no invariant".into())) };
        let b = out.as_bytes();
        let Some((params, i)) = read_sexp(b, at + tag.len() - 1) else { return Err(Spacer::Unasked("Spacer's invariant could not be read".into())) };
        let Some((_, i)) = read_sexp(b, i) else { return Err(Spacer::Unasked("Spacer's invariant could not be read".into())) };
        let Some((body, _)) = read_sexp(b, i) else { return Err(Spacer::Unasked("Spacer's invariant could not be read".into())) };
        let shown = show_invariant(p, f, &keys, &body);
        shown_all.push(format!("line-{} loop: {}", h, shown));
        let names: Vec<String> = match ints::parse(&params) {
            Some(ints::Sx::List(ps)) => ps.iter().filter_map(|p| match p {
                ints::Sx::List(pair) => match pair.first() {
                    Some(ints::Sx::Atom(n)) => Some(n.clone()),
                    _ => None,
                },
                _ => None,
            }).collect(),
            _ => Vec::new(),
        };
        let typed: Vec<(String, u32, bool)> = names.iter().zip(&keys).map(|(n, k)| (n.clone(), k.bits, k.signed)).collect();
        let bits = if names.len() == keys.len() { ints::parse(&body).and_then(|bd| ints::to_bits(&bd, &typed)) } else { None };
        let Some(bits) = bits else { return Err(Spacer::NotConfirmed { shown, why: "it could not be read back into bit-vectors".into() }) };
        let params = format!("({})", typed.iter().map(|(n, b, _)| format!("({} (_ BitVec {}))", n, b)).collect::<Vec<_>>().join(" "));
        invs.push(Invariant { head: *h, keys, params, body: bits, shown });
    }
    // Confirmed in bit-vectors, walk by walk, every invariant at once.
    let defines: String = invs.iter().map(|i| i.define() + "\n").collect();
    for (start, e) in &walks {
        if e.arrivals.is_empty() {
            continue;
        }
        let mut q = format!("(set-option :rlimit {})\n(set-logic QF_BV)\n{}\n{}", RLIMIT, e.defs.join("\n"), defines);
        if *start != usize::MAX {
            let _ = writeln!(q, "(assert (inv{} {}))", start, e.start_terms.join(" "));
        }
        let ways: Vec<String> = e.arrivals.iter().map(|(to, g, t)| format!("(and {} (not (inv{} {})))", g, to, t.join(" "))).collect();
        let _ = writeln!(q, "(assert (or false {}))\n(check-sat)", ways.join(" "));
        let (_, each) = portfolio.ask(&Query { text: q, assumptions: String::new(), truncated: false, assumed: false, reach: String::new(), inputs: Vec::new(), points: Vec::new() });
        if !each.iter().all(|(_, a)| *a == Answer::Unsat) {
            let said: Vec<String> = each.iter().map(|(n, a)| format!("{} {}", n, match a { Answer::Unsat => "confirmed it", Answer::Sat(_) => "found it false", Answer::Unknown => "could not say" })).collect();
            let what = if *start == usize::MAX { "the walk from the entry".to_string() } else { format!("the walk from the loop at block {}", start) };
            return Err(Spacer::NotConfirmed { shown: shown_all.join("; "), why: format!("on {}: {}", what, said.join(", ")) });
        }
    }
    Ok(invs)
}

/// The joint system is asked when `KAYO_SPACER_JOINT` is set (with
/// `KAYO_SPACER_INTEGERS`).
pub fn spacer_joint_on() -> bool {
    std::env::var("KAYO_SPACER_JOINT").is_ok() && spacer_integers()
}

/// Spacer searches over the integers instead of bit-vectors when
/// `KAYO_SPACER_INTEGERS` is set (owner, 2026-10-29: "Test out number 2
/// first"); bit-vectors stay the default until the owner decides.
pub fn spacer_integers() -> bool {
    std::env::var("KAYO_SPACER_INTEGERS").is_ok()
}

/// The three parts of a loop's clauses over the integers (`ints.rs`).
fn int_clauses(text: &mut String, keys: &[HeadKey], entry: &Enc, trip: &Enc, from: &Enc) {
    use crate::precog::ints;
    let read = |s: &str| ints::parse(s).unwrap_or(ints::Sx::Atom("true".into()));
    // Each part: its lines as conjuncts, then what arrives at the head (or
    // the bad value), each term read as the key's number.
    let args = |tr: &mut ints::Ints, terms: &[String]| -> String {
        terms.iter().zip(keys).map(|(t, k)| tr.number(&read(t), k.signed).unwrap_or_else(|| tr.fresh_int())).collect::<Vec<_>>().join(" ")
    };
    let (mut vars, conj, mut tr) = ints::part(&entry.defs, &entry.sorts);
    let mut rules = Vec::new();
    for (g, terms) in &entry.recorded {
        let g = tr.boolean(&read(g)).unwrap_or_else(|| "true".into());
        let a = args(&mut tr, terms);
        rules.push((vec![g], format!("(Inv {})", a)));
    }
    vars.extend(tr.unknowns.drain(..));
    for (extra, head) in rules {
        text.push_str(&horn_rule(&vars, &conj, &extra, &head));
    }
    let (mut vars, conj, mut tr) = ints::part(&trip.defs, &trip.sorts);
    let at_head = format!("(Inv {})", args(&mut tr, &trip.start_terms));
    let mut rules = Vec::new();
    for (g, terms) in &trip.recorded {
        let g = tr.boolean(&read(g)).unwrap_or_else(|| "true".into());
        let a = args(&mut tr, terms);
        rules.push((vec![at_head.clone(), g], format!("(Inv {})", a)));
    }
    vars.extend(tr.unknowns.drain(..));
    for (extra, head) in rules {
        text.push_str(&horn_rule(&vars, &conj, &extra, &head));
    }
    let (mut vars, conj, mut tr) = ints::part(&from.defs, &from.sorts);
    let at_head = format!("(Inv {})", args(&mut tr, &from.start_terms));
    let bad: Vec<String> = from.reached.iter().filter_map(|r| tr.boolean(&read(r))).collect();
    vars.extend(tr.unknowns.drain(..));
    text.push_str(&horn_rule(&vars, &conj, &[at_head, format!("(or false {})", bad.join(" "))], "false"));
}

/// A query's lines as one Horn clause body: each value it declares or
/// defines is a variable of the clause, each definition and assertion a
/// conjunct.
fn horn_parts(defs: &[String]) -> (Vec<String>, Vec<String>) {
    let mut vars = Vec::new();
    let mut conj = Vec::new();
    for d in defs {
        if let Some(rest) = d.strip_prefix("(declare-fun ") {
            let Some((name, rest)) = rest.split_once(' ') else { continue };
            let sort = rest.strip_prefix("() ").and_then(|s| s.strip_suffix(')')).unwrap_or("Bool");
            vars.push(format!("({} {})", name, sort));
        } else if let Some(rest) = d.strip_prefix("(define-fun ") {
            let Some((name, rest)) = rest.split_once(' ') else { continue };
            let Some(rest) = rest.strip_prefix("() ") else { continue };
            let (sort, body) = match rest.strip_prefix("Bool ") {
                Some(body) => ("Bool", body),
                None => {
                    let Some(end) = rest.find(") ") else { continue };
                    (&rest[..=end], &rest[end + 2..])
                }
            };
            let body = body.strip_suffix(')').unwrap_or(body);
            vars.push(format!("({} {})", name, sort));
            conj.push(format!("(= {} {})", name, body));
        } else if let Some(rest) = d.strip_prefix("(assert ") {
            conj.push(rest.strip_suffix(')').unwrap_or(rest).to_string());
        }
    }
    (vars, conj)
}

fn horn_rule(vars: &[String], conj: &[String], extra: &[String], head: &str) -> String {
    let body = format!("(and true {} {})", conj.join(" "), extra.join(" "));
    if vars.is_empty() {
        format!("(assert (=> {} {}))\n", body, head)
    } else {
        format!("(assert (forall ({}) (=> {} {})))\n", vars.join(" "), body, head)
    }
}

/// Spacer's invariant in the program's names, for the log.
fn show_invariant(p: &Program, f: usize, keys: &[HeadKey], body: &str) -> String {
    let func = &p.functions[f];
    let mut s = body.split_whitespace().collect::<Vec<_>>().join(" ");
    for (i, k) in keys.iter().enumerate().rev() {
        let name = |l: &LocalId| func.locals[l.0 as usize].name.clone().map(|n| format!("${}", n)).unwrap_or_else(|| format!("_{}", l.0));
        let shown = match &k.key {
            Key::Local(l) => name(l),
            Key::Len(l, _) => format!("len({})", name(l)),
            _ => format!("v{}", i),
        };
        s = s.replace(&format!("x!{}", i), &shown);
    }
    s
}

/// Z3 on a text, its whole answer back.
fn run_raw(s: &Solver, text: &str) -> String {
    if std::env::var("KAYO_KEEP_SMT").is_ok() {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join("cca-smt");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join(format!("h{}-{}.smt2", std::process::id(), n)), text);
    }
    let mut cmd = std::process::Command::new(s.program);
    cmd.arg("-smt2").arg("-in");
    cmd.stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::null());
    let started = std::time::Instant::now();
    let Ok(mut child) = cmd.spawn() else { return String::new() };
    if let Some(mut input) = child.stdin.take() {
        use std::io::Write as _;
        let _ = input.write_all(text.as_bytes());
    }
    let Ok(out) = child.wait_with_output() else { return String::new() };
    let out = String::from_utf8_lossy(&out.stdout).to_string();
    if std::env::var("KAYO_SMT_TIMES").is_ok() {
        eprintln!("smt: Spacer {} bytes, {:.2} s, {}", text.len(), started.elapsed().as_secs_f64(), out.lines().next().unwrap_or("no answer"));
    }
    out
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
        // An exact answer is held in its limit and one more bit. Past a
        // few thousand bits the solvers are not asked at all (owner,
        // 2026-10-03: a width may be 2^23 bits): layer 2, or a check.
        Ty::Int(IntW::Fixed { signed, bits }) => {
            let (signed, bits) = crate::resolve::storage(*signed, *bits);
            (bits <= 4096).then_some((bits, signed))
        }
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
    if let Ty::Bounded { bound: Bound::Int { lo, hi, .. }, .. } = ty {
        return Some((*lo, *hi));
    }
    if let Ty::Bounded { bound: Bound::Len { lo, hi, .. }, .. } = ty {
        return Some((*lo as i128, *hi as i128));
    }
    match ty.base() {
        // Past what `i128` holds the range is not written down: the
        // solvers then know only the width.
        Ty::Int(IntW::Fixed { signed, bits }) => crate::resolve::int_range_exact(*signed, *bits),
        Ty::Int(IntW::Free(_)) => Some((i64::MIN as i128, i64::MAX as i128)),
        Ty::Bool => Some((0, 1)),
        Ty::Duration => Some((0, i64::MAX as i128)),
        _ => None,
    }
}
