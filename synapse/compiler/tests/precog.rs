//! Layer 3, on its own terms.
//!
//! The danger with handing a site to a solver is not a wrong answer: it
//! is a question that was never asked. A query whose assumptions
//! contradict each other comes back `unsat` whatever the program does,
//! and `unsat` is how layer 3 says *proven* — so an impossible question
//! deletes a check that was needed. These tests ask the queries about
//! themselves, and then ask layer 3 for answers it should and should not
//! give.
//!
//! Every test here needs a solver. With none on the machine there is
//! nothing to test and the test says so rather than passing quietly.

use brainstem::mir::{Verdict, Program};
use brainstem::cortex::smt;

fn build(src: &str) -> Program {
    let dir = std::env::temp_dir().join("cca-precog-tests");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(format!("t{}.nrv", hash(src)));
    std::fs::write(&path, src).unwrap();
    let mut sources = brainstem::source::SourceMap::default();
    let items = brainstem::load::load(&path, &mut sources).unwrap_or_else(|d| panic!("{}", d.render(&sources)));
    let mut checker = brainstem::check::Checker::new(items);
    checker.std_stubs = true;
    let program = checker.check_program().unwrap_or_else(|d| panic!("{}", d.render(&sources)));
    let mut mir = brainstem::mir::build::build(program);
    let _ = brainstem::mir::ownership::check(&mut mir, &sources);
    brainstem::cortex::analyse(&mut mir);
    mir
}

fn hash(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

fn have_a_solver() -> bool {
    !smt::Portfolio::find().members.is_empty()
}

/// Every note Precog left, one per site.
fn notes(p: &Program) -> Vec<String> {
    let mut out = Vec::new();
    for f in &p.functions {
        for s in &f.sites {
            out.push(format!("{:?} {}", s.verdict, describe(&s.verdict)));
        }
    }
    out
}

fn describe(v: &Verdict) -> String {
    match v {
        Verdict::Proven { how } => format!("proven by {}", how),
        Verdict::Fails { counterexample } => format!("fails: {}", counterexample),
        Verdict::Trusted => "trusted".into(),
        Verdict::Unproven => "unproven".into(),
    }
}

/// The one that bit: a `uint32` holds 0..4294967295, and 4294967295
/// written as 32 bits *is* −1, so `v >= 0 and v <= 4294967295` asked at
/// the value's own width says `v >= 0 and v <= -1` — which nothing
/// satisfies. Every query about a full-width unsigned value came back
/// `unsat`, and every one of them read as a proof.
#[test]
fn a_query_can_always_be_satisfied_by_something() {
    assert!(have_a_solver(), "no solver on this machine, so layer 3 cannot be tested at all");
    let portfolio = smt::Portfolio::find();
    let mut asked = 0;
    for src in PROGRAMS {
        let p = build(src);
        for (fi, f) in p.functions.iter().enumerate() {
            for (si, _) in f.sites.iter().enumerate() {
                let Some(q) = smt::encode(&p, fi, brainstem::mir::SiteId(si as u32), &Default::default()) else { continue };
                asked += 1;
                assert_eq!(
                    portfolio.ask_text(&q.assumptions),
                    smt::Answer::Sat(String::new()),
                    "a query about this site cannot be satisfied by any input at all, so every answer it gives is `unsat` and every `unsat` reads as a proof:\n{}",
                    q.assumptions
                );
            }
        }
    }
    assert!(asked > 0, "no site was put to layer 3, so nothing was tested");
}

/// A value that fills its type is not proven to stay inside it.
#[test]
fn a_full_width_unsigned_value_is_not_proven_safe() {
    assert!(have_a_solver(), "no solver on this machine, so layer 3 cannot be tested at all");
    let p = build(
        // A `uint32` straight from C, so every value the type holds is
        // possible — `to.uint32[getpid[]]` would not do: Precog knows that
        // one is at most 2^31 - 1, and then `'x' + 1` really is safe.
        "extern.C fn getuid [] -> uint32;\n\
         fn bump ['x': uint32] -> uint32 {\n\
         \x20   let 'y': uint32 = 'x' plus 1;\n\
         \x20   return 'y';\n\
         }\n\
         MAIN {\n\
         \x20   no_check let 'u' := getuid[];\n\
         \x20   let 'r' := bump['u'];\n\
         }\n",
    );
    let ns = notes(&p);
    assert!(
        ns.iter().any(|n| n.contains("fails")),
        "adding 1 to a `uint32` that may already be 4294967295 was not caught:\n{}",
        ns.join("\n")
    );
    assert!(
        !ns.iter().any(|n| n.contains("proven by layer 3")),
        "a site was proven by layer 3 that cannot be proven:\n{}",
        ns.join("\n")
    );
}

/// What layer 3 is for: two values layer 2 only knows separately, and a
/// guard that relates them.
#[test]
fn layer_3_proves_what_layer_2_can_only_guess_at() {
    assert!(have_a_solver(), "no solver on this machine, so layer 3 cannot be tested at all");
    let p = build(
        "extern.C fn getpid [] -> int32;\n\
         MAIN {\n\
         \x20   no_check let 'a': uint8 = to.uint8[getpid[]];\n\
         \x20   no_check let 'b': uint8 = to.uint8[getpid[]];\n\
         \x20   if 'a' < 'b' {\n\
         \x20       let 'gap': uint8 = 'b' minus 'a';\n\
         \x20   }\n\
         }\n",
    );
    let ns = notes(&p);
    assert!(
        ns.iter().any(|n| n.contains("proven by layer 3")),
        "`'b' - 'a'` under `'a' < 'b'` is exactly what layer 3 is for, and it was not proven:\n{}",
        ns.join("\n")
    );
}

/// Programs whose sites are worth putting to a solver: every width, both
/// signs, a range, a length, a loop.
///
/// The first one matters most: its unknowns are *parameters*, so the
/// query has to say what each type holds. A value derived from a call
/// does not test that — the call's own type is what gets written down.
const PROGRAMS: &[&str] = &[
    "extern.C fn getpid [] -> int32;\n\
     fn a8 ['x': uint8] -> uint8 { let 'y': uint8 = 'x' plus 1; return 'y'; }\n\
     fn a16 ['x': uint16] -> uint16 { let 'y': uint16 = 'x' plus 1; return 'y'; }\n\
     fn a32 ['x': uint32] -> uint32 { let 'y': uint32 = 'x' plus 1; return 'y'; }\n\
     fn a64 ['x': uint64] -> uint64 { let 'y': uint64 = 'x' plus 1; return 'y'; }\n\
     fn b8 ['x': int8] -> int8 { let 'y': int8 = 'x' plus 1; return 'y'; }\n\
     fn b64 ['x': int64] -> int64 { let 'y': int64 = 'x' plus 1; return 'y'; }\n\
     MAIN {\n\
     \x20   no_check let 'p' := getpid[];\n\
     \x20   no_check let 'u8': uint8 = to.uint8['p'];\n\
     \x20   no_check let 'u16': uint16 = to.uint16['p'];\n\
     \x20   no_check let 'u32': uint32 = to.uint32['p'];\n\
     \x20   no_check let 'u64': uint64 = to.uint64['p'];\n\
     \x20   no_check let 'i8': int8 = to.int8['p'];\n\
     \x20   no_check let 'i64': int64 = to.int64['p'];\n\
     \x20   let 'r1' := a8['u8'];\n\
     \x20   let 'r2' := a16['u16'];\n\
     \x20   let 'r3' := a32['u32'];\n\
     \x20   let 'r4' := a64['u64'];\n\
     \x20   let 'r5' := b8['i8'];\n\
     \x20   let 'r6' := b64['i64'];\n\
     }\n",
    "extern.C fn getpid [] -> int32;\n\
     MAIN {\n\
     \x20   no_check let 'a': uint8 = to.uint8[getpid[]];\n\
     \x20   no_check let 'b': uint16 = to.uint16[getpid[]];\n\
     \x20   no_check let 'c': uint32 = to.uint32[getpid[]];\n\
     \x20   no_check let 'd': uint64 = to.uint64[getpid[]];\n\
     \x20   let 'w': uint8 = 'a' plus 1;\n\
     \x20   let 'x': uint16 = 'b' plus 1;\n\
     \x20   let 'y': uint32 = 'c' plus 1;\n\
     \x20   let 'z': uint64 = 'd' plus 1;\n\
     }\n",
    "extern.C fn getpid [] -> int32;\n\
     MAIN {\n\
     \x20   no_check let 'a': int8 = to.int8[getpid[]];\n\
     \x20   no_check let 'b': int16 = to.int16[getpid[]];\n\
     \x20   no_check let 'c': int32 = getpid[];\n\
     \x20   no_check let 'd': int64 = to.int64[getpid[]];\n\
     \x20   let 'w': int8 = 'a' minus 1;\n\
     \x20   let 'x': int16 = 'b' times 2;\n\
     \x20   let 'y': int32 = 'c' plus 'c';\n\
     \x20   let 'z': int64 = 'd' plus 1;\n\
     }\n",
    "extern.C fn getpid [] -> int32;\n\
     MAIN {\n\
     \x20   no_check let 'n': int32 = getpid[];\n\
     \x20   let 'r': int32[min 1, max 10] = 'n';\n\
     \x20   let 'q': int32 = 100 div.floor 'n';\n\
     \x20   let 'm': int32 = 100 mod.floor 'n';\n\
     }\n",
    "extern.C fn getpid [] -> int32;\n\
     MAIN {\n\
     \x20   no_check let 'n': int32 = getpid[];\n\
     \x20   let mut 'total': int32 = 0;\n\
     \x20   for 'i': int32 from 1 to 10 by 1 {\n\
     \x20       change 'total' = 'total' plus 'i' plus 'n';\n\
     \x20   }\n\
     }\n",
];
