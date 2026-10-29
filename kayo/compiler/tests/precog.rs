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

use kayo::mir::{Verdict, Program};
use kayo::precog::smt;

fn build(src: &str) -> Program {
    let dir = std::env::temp_dir().join("cca-precog-tests");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(format!("t{}.cy", hash(src)));
    std::fs::write(&path, src).unwrap();
    let mut sources = kayo::source::SourceMap::default();
    let items = kayo::load::load(&path, &mut sources).unwrap_or_else(|d| panic!("{}", d.render(&sources)));
    let mut checker = kayo::check::Checker::new(items);
    checker.std_stubs = true;
    let program = checker.check_program().unwrap_or_else(|d| panic!("{}", d.render(&sources)));
    let mut mir = kayo::mir::build::build(program);
    let _ = kayo::mir::ownership::check(&mut mir, &sources);
    kayo::precog::analyse(&mut mir);
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

/// The one that bit: a `u32` holds 0..4294967295, and 4294967295
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
                let Some(q) = smt::encode(&p, fi, kayo::mir::SiteId(si as u32), &Default::default()) else { continue };
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
        // A `u32` straight from C, so every value the type holds is
        // possible — `to.u32[getpid[]]` would not do: Precog knows that
        // one is at most 2^31 - 1, and then `'x' + 1` really is safe.
        "%extern.C proc getuid [] -> u32;\n\
         fn bump [$x: u32] -> u32:\n\
         \x20   let $y: u32 := $x + 1;\n\
         \x20   ret $y;\n\
         ;fn bump\n\
         MAIN:\n\
         \x20   no_check let $u := getuid();\n\
         \x20   let $r := bump($u);\n\
         ;MAIN\n",
    );
    let ns = notes(&p);
    assert!(
        ns.iter().any(|n| n.contains("fails")),
        "adding 1 to a `u32` that may already be 4294967295 was not caught:\n{}",
        ns.join("\n")
    );
    assert!(
        !ns.iter().any(|n| n.contains("proven by layer 3")),
        "a site was proven by layer 3 that cannot be proven:\n{}",
        ns.join("\n")
    );
}

/// A const array's length is known while building, to layer 3 too, and
/// `array.length` is the length a read's bounds ask about (std claude,
/// 2026-10-21: a search by halves in a const table left its reads checked,
/// the array 7 long and 0 long at once). Every site here is proven.
#[test]
fn a_const_arrays_length_is_known_to_layer_3() {
    assert!(have_a_solver(), "no solver on this machine, so layer 3 cannot be tested at all");
    let p = build(
        "%extern.C proc getuid [] -> u32;\n\
         const $keys: arr[u32] := [65, 66, 67, 223, 913, 931];\n\
         fn look [$c: u32] -> u32:\n\
         \x20   let $n := arr.len($keys);\n\
         \x20   var $p: u64[max: 6] = 0;\n\
         \x20   if $p + 4 <= $n and.then $keys[pos: $p + 4] <= $c:\n\
         \x20       change $p to $p + 4;\n\
         \x20   ;if\n\
         \x20   if $p + 2 <= $n and.then $keys[pos: $p + 2] <= $c:\n\
         \x20       change $p to $p + 2;\n\
         \x20   ;if\n\
         \x20   if $p + 1 <= $n and.then $keys[pos: $p + 1] <= $c:\n\
         \x20       change $p to $p + 1;\n\
         \x20   ;if\n\
         \x20   if $p >= 1 and.then $keys[pos: $p] == $c:\n\
         \x20       ret $p;\n\
         \x20   ;if\n\
         \x20   ret 0;\n\
         ;fn look\n\
         MAIN:\n\
         \x20   no_check let $u := getuid();\n\
         \x20   let $r := look($u);\n\
         ;MAIN\n",
    );
    let ns = notes(&p);
    assert!(
        ns.iter().all(|n| n.contains("proven")),
        "a guarded read of a const array, or the change it allows, was left unproven:\n{}",
        ns.join("\n")
    );
}

/// What layer 3 is for: two values layer 2 only knows separately, and a
/// guard that relates them.
#[test]
fn layer_3_proves_what_layer_2_can_only_guess_at() {
    assert!(have_a_solver(), "no solver on this machine, so layer 3 cannot be tested at all");
    let p = build(
        "%extern.C proc getpid [] -> i32;\n\
         MAIN:\n\
         \x20   no_check let $a: u8 := to.u8(getpid());\n\
         \x20   no_check let $b: u8 := to.u8(getpid());\n\
         \x20   if $a < $b:\n\
         \x20       let $gap: u8 := $b - $a;\n\
         \x20   ;if\n\
         ;MAIN\n",
    );
    let ns = notes(&p);
    assert!(
        ns.iter().any(|n| n.contains("proven by layer 3")),
        "`'b' - 'a'` under `'a' < 'b'` is exactly what layer 3 is for, and it was not proven:\n{}",
        ns.join("\n")
    );
}

/// Spacer (owner, 2026-10-25, on by default while it is measured): two
/// counters that move together are a relation no shape says, so `$j - $i
/// + 1` after the loop is never 0 only by an invariant Spacer finds and
/// both solvers confirm. Started one apart, the same division is 0 on every
/// run, and no invariant may prove it.
#[test]
fn spacer_finds_the_invariant_no_shape_says_and_never_a_false_one() {
    assert!(have_a_solver(), "no solver on this machine, so layer 3 cannot be tested at all");
    if !smt::spacer_on() {
        return;
    }
    let program = |j0: &str| {
        format!(
            "fn gap [$n: i32] -> i32:\n\
             \x20   var $i: i32 = 0;\n\
             \x20   var $j: i32 = {};\n\
             \x20   while $i < $n:\n\
             \x20       change $i to $i + 1;\n\
             \x20       change $j to $j + 1;\n\
             \x20   ;while\n\
             \x20   ret 100 div.floor ($j - $i + 1);\n\
             ;fn gap\n\
             MAIN [$argc: i32, $argv: ptr[ptr[u8]]]:\n\
             \x20   std::print(gap($argc));\n\
             ;MAIN\n",
            j0
        )
    };
    let division = |p: &Program| {
        let f = p.functions.iter().find(|f| f.name == "gap").unwrap();
        f.sites.iter().find(|s| matches!(s.kind, kayo::mir::SiteKind::DivZero)).map(|s| describe(&s.verdict)).unwrap()
    };
    let good = division(&build(&program("0")));
    assert!(good.contains("Spacer"), "the division after the lock-step loop should be proven with Spacer's invariant: {}", good);
    let bad = division(&build(&program("-1")));
    assert!(!bad.starts_with("proven"), "the division is 0 on every run, and it was proven: {}", bad);
}

/// Programs whose sites are worth putting to a solver: every width, both
/// signs, a range, a length, a loop.
///
/// The first one matters most: its unknowns are *parameters*, so the
/// query has to say what each type holds. A value derived from a call
/// does not test that — the call's own type is what gets written down.
const PROGRAMS: &[&str] = &[
    "%extern.C proc getpid [] -> i32;\n\
     fn a8 [$x: u8] -> u8:\n    let $y: u8 := $x + 1;\n    ret $y;\n;fn a8\n\
     fn a16 [$x: u16] -> u16:\n    let $y: u16 := $x + 1;\n    ret $y;\n;fn a16\n\
     fn a32 [$x: u32] -> u32:\n    let $y: u32 := $x + 1;\n    ret $y;\n;fn a32\n\
     fn a64 [$x: u64] -> u64:\n    let $y: u64 := $x + 1;\n    ret $y;\n;fn a64\n\
     fn b8 [$x: i8] -> i8:\n    let $y: i8 := $x + 1;\n    ret $y;\n;fn b8\n\
     fn b64 [$x: i64] -> i64:\n    let $y: i64 := $x + 1;\n    ret $y;\n;fn b64\n\
     MAIN:\n\
     \x20   no_check let $p := getpid();\n\
     \x20   no_check let $u8: u8 := to.u8($p);\n\
     \x20   no_check let $u16: u16 := to.u16($p);\n\
     \x20   no_check let $u32: u32 := to.u32($p);\n\
     \x20   no_check let $u64: u64 := to.u64($p);\n\
     \x20   no_check let $i8: i8 := to.i8($p);\n\
     \x20   no_check let $i64: i64 := to.i64($p);\n\
     \x20   let $r1 := a8($u8);\n\
     \x20   let $r2 := a16($u16);\n\
     \x20   let $r3 := a32($u32);\n\
     \x20   let $r4 := a64($u64);\n\
     \x20   let $r5 := b8($i8);\n\
     \x20   let $r6 := b64($i64);\n\
     ;MAIN\n",
    "%extern.C proc getpid [] -> i32;\n\
     MAIN:\n\
     \x20   no_check let $a: u8 := to.u8(getpid());\n\
     \x20   no_check let $b: u16 := to.u16(getpid());\n\
     \x20   no_check let $c: u32 := to.u32(getpid());\n\
     \x20   no_check let $d: u64 := to.u64(getpid());\n\
     \x20   let $w: u8 := $a + 1;\n\
     \x20   let $x: u16 := $b + 1;\n\
     \x20   let $y: u32 := $c + 1;\n\
     \x20   let $z: u64 := $d + 1;\n\
     ;MAIN\n",
    "%extern.C proc getpid [] -> i32;\n\
     MAIN:\n\
     \x20   no_check let $a: i8 := to.i8(getpid());\n\
     \x20   no_check let $b: i16 := to.i16(getpid());\n\
     \x20   no_check let $c: i32 := getpid();\n\
     \x20   no_check let $d: i64 := to.i64(getpid());\n\
     \x20   let $w: i8 := $a - 1;\n\
     \x20   let $x: i16 := $b x 2;\n\
     \x20   let $y: i32 := $c + $c;\n\
     \x20   let $z: i64 := $d + 1;\n\
     ;MAIN\n",
    "%extern.C proc getpid [] -> i32;\n\
     MAIN:\n\
     \x20   no_check let $n: i32 := getpid();\n\
     \x20   let $r: i32[min: 1, max: 10] := $n;\n\
     \x20   let $q: i32 := 100 div.floor $n;\n\
     \x20   let $m: i32 := 100 mod.floor $n;\n\
     ;MAIN\n",
    "%extern.C proc getpid [] -> i32;\n\
     MAIN:\n\
     \x20   no_check let $n: i32 := getpid();\n\
     \x20   var $total: i32 = 0;\n\
     \x20   for $i: i32 [from: 1, to: 10, by: 1]:\n\
     \x20       change $total to $total + $i + $n;\n\
     \x20   ;for\n\
     ;MAIN\n",
];
