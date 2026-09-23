//! The whole pipeline, all the way to a binary that runs.
//!
//! These programs are CCA's own (`tests/programs/`), not the language's
//! examples: `examples/` belongs to the design session, and every one of
//! them needs either `std::` or a heap type, neither of which has a shape
//! yet. What is here is the base language without a heap — which is
//! exactly what the generated code covers today, so the test says what is
//! true rather than what is hoped.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/programs")
}

fn cca() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cca"))
}

/// Where cargo put the runtime. The linker needs it, and a missing one is
/// a setup problem worth saying out loud rather than skipping over.
fn runtime() -> Option<PathBuf> {
    let exe = cca();
    for d in exe.parent()?.ancestors().take(4) {
        for profile in ["debug", "release"] {
            let p = d.join(profile).join("libcca_runtime.a");
            if p.is_file() {
                return Some(p);
            }
        }
        let p = d.join("libcca_runtime.a");
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

/// `name want` lines of `expected.txt`.
fn programs() -> Vec<(String, String)> {
    let expected = std::fs::read_to_string(dir().join("expected.txt")).unwrap();
    expected
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let (n, w) = l.split_once(' ').unwrap();
            (n.to_string(), w.trim().to_string())
        })
        .collect()
}

fn have_leaks() -> bool {
    Command::new("leaks").arg("--help").output().map(|o| o.status.success()).unwrap_or(false)
}

/// Every program builds, exits as `expected.txt` says, and gives back
/// everything it took.
///
/// The last is the test that catches a heap bug. An exit code is happy to
/// be right while a list's buffer, a map's table or a built text is never
/// freed — or is freed twice, which is worse and usually still exits 0.
/// macOS has `leaks`; where it does not exist the test says so rather
/// than passing quietly. One build serves both: they used to be two tests
/// each building every program.
#[test]
fn programs_build_run_and_give_back() {
    assert!(runtime().is_some(), "the runtime is not built — run `cargo build -p cca-runtime` first; the linker needs libcca_runtime.a");
    let leaks = have_leaks();
    if !leaks {
        eprintln!("note: no `leaks` on this machine, so nothing checked what the programs freed");
    }
    let out_dir = std::env::temp_dir().join("cca-codegen-tests");
    let _ = std::fs::create_dir_all(&out_dir);
    let results = common::par_map(&programs(), |(name, want)| -> Vec<String> {
        let mut failures = Vec::new();
        let src = dir().join(format!("{}.cpls", name));
        let bin = out_dir.join(name);
        let build = Command::new(cca()).arg("build").arg(&src).arg("-o").arg(&bin).output().unwrap();
        if !build.status.success() {
            return vec![format!("{}: would not build:\n{}", name, indent(&String::from_utf8_lossy(&build.stderr)))];
        }
        let run = Command::new(&bin).output().unwrap();
        let code = run.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&run.stderr).to_string();
        match want.as_str() {
            "trap" => {
                if code != 101 {
                    failures.push(format!("{}: a check should have fired (exit 101), and it exited {}", name, code));
                } else if !stderr.starts_with("check:") || !stderr.contains("line ") {
                    failures.push(format!("{}: the check fired without naming its site:\n{}", name, indent(&stderr)));
                }
            }
            _ => {
                let want: i32 = want.parse().unwrap();
                if code != want {
                    failures.push(format!("{}: exited {}, and it should exit {}\n{}", name, code, want, indent(&stderr)));
                }
            }
        }
        // A program that stops on a check never reaches the end, so what
        // it was still holding is not a leak.
        if leaks && want != "trap" {
            let r = Command::new("leaks").arg("--atExit").arg("--").arg(&bin).output().unwrap();
            let text = String::from_utf8_lossy(&r.stdout).to_string();
            if !text.contains("0 leaks for 0 total leaked bytes") {
                let said = text.lines().filter(|l| l.contains("leak")).collect::<Vec<_>>().join("\n");
                failures.push(format!("{}: did not give back everything it took: {}", name, said));
            }
        }
        failures
    });
    let failures: Vec<String> = results.into_iter().flatten().collect();
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

/// A guarded value keeps no check, and the proof reaches LLVM.
#[test]
fn a_proven_site_hands_its_fact_to_llvm() {
    let ir = ir_of("proven_add");
    assert!(!ir.contains("with.overflow"), "a site Precog proved still carries a run-time check:\n{}", ir);
    // `nsw`, or `nuw` where Precog also proved the answer is never negative.
    assert!(ir.contains("nsw") || ir.contains("nuw"), "a proven addition reached LLVM without the fact that it cannot wrap:\n{}", ir);
}

/// Exact math at the machine's width where Precog proves it fits: no
/// 128-bit multiply, no 65-bit remainder (a library call) in the loop.
#[test]
fn exact_math_works_at_64_bits_where_it_fits() {
    let ir = ir_of("exact_at_64");
    assert!(!ir.contains("mul nsw i128") && !ir.contains("mul i128"), "a proven 64-bit product was done at 128 bits:\n{}", ir);
    assert!(!ir.contains("rem i65") && !ir.contains("rem i128"), "a proven 64-bit remainder was done wider:\n{}", ir);
}

/// A program whose values are all known has the answers in it, not the
/// working: layer 1 ran it (`precog.md`).
#[test]
fn a_known_program_is_already_the_answer() {
    let ir = ir_of("arithmetic");
    assert!(!ir.contains("with.overflow"), "a known program still carries a run-time check:\n{}", ir);
    assert!(!ir.contains(" add ") && !ir.contains("add nsw"), "a known program still does its arithmetic at run time:\n{}", ir);
}

/// A list owns its buffer, so a list that goes out of scope gives it
/// back. This is the one thing a leak check catches and a test of the
/// exit code never would.
#[test]
fn a_list_gives_its_memory_back() {
    let ir = ir_of("lists");
    assert!(ir.contains("cyborg_alloc"), "a list literal reached LLVM without allocating:\n{}", ir);
    assert!(ir.contains("cy_drop_"), "a list went out of scope and nothing gave its memory back:\n{}", ir);
    assert!(ir.contains("cyborg_free"), "nothing in the generated code frees anything:\n{}", ir);
}

/// `copy` is "a second, independent value; an allocation"
/// (`ownership-syntax.md`) — not a second handle onto one buffer, which
/// would be freed twice.
#[test]
fn copying_a_list_copies_what_it_owns() {
    let ir = ir_of("list_copy");
    assert!(ir.contains("cy_copy_"), "`copy` of a list did not copy the buffer it owns:\n{}", ir);
}

/// With proofs asserted, every program does exactly what it does without:
/// an assertion that trips on a correct proof is a false alarm of the most
/// expensive kind — the verification session treats every `proof failed`
/// as a wrong proof. Nothing here is folded and nothing is promised, so
/// this also runs what the ordinary build worked out at compile time.
#[test]
fn programs_run_the_same_with_proofs_asserted() {
    assert!(runtime().is_some(), "the runtime is not built");
    let out_dir = std::env::temp_dir().join("cca-asserted-tests");
    let _ = std::fs::create_dir_all(&out_dir);
    let results = common::par_map(&programs(), |(name, want)| -> Option<String> {
        let bin = out_dir.join(name);
        let build = Command::new(cca()).arg("build").arg("--assert-proofs").arg(dir().join(format!("{}.cpls", name))).arg("-o").arg(&bin).output().unwrap();
        if !build.status.success() {
            return Some(format!("{}: would not build with proofs asserted:\n{}", name, indent(&String::from_utf8_lossy(&build.stderr))));
        }
        let run = Command::new(&bin).output().unwrap();
        let code = run.status.code().unwrap_or(-1);
        let want_code = if want == "trap" { 101 } else { want.parse().unwrap() };
        (code != want_code).then(|| format!("{}: exited {} with proofs asserted, and {} without\n{}", name, code, want_code, indent(&String::from_utf8_lossy(&run.stderr))))
    });
    let failures: Vec<String> = results.into_iter().flatten().collect();
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

fn final_build(path: &Path) -> std::process::Output {
    let out = std::env::temp_dir().join(format!("cca-final-tests/{}", path.file_stem().unwrap().to_string_lossy()));
    let _ = std::fs::create_dir_all(out.parent().unwrap());
    Command::new(cca()).arg("build").arg("--final").arg(path).arg("-o").arg(&out).output().unwrap()
}

/// The final build accepts loops whose facts are counted (owner,
/// 2026-09-23): a running sum is at most its trips times its largest step
/// (`bench/lists.cpls`), and a list appended once a trip is as long as the
/// counter has come (`bench/sieve.cpls`) — and recursion proven by
/// induction on its arguments (`bench/fib.cpls`). And it still refuses
/// where the count does not fit, where the appends are not one a trip,
/// where fib goes one past what fits, where a function recurses upward,
/// and where the counter may not move on a trip.
#[test]
fn final_build_proves_counted_loops_and_no_more() {
    let bench = Path::new(env!("CARGO_MANIFEST_DIR")).join("../bench");
    for name in ["lists", "sieve", "fib"] {
        let r = final_build(&bench.join(format!("{}.cpls", name)));
        assert!(r.status.success(), "the final build refused bench/{}.cpls:\n{}", name, String::from_utf8_lossy(&r.stderr));
    }
    let tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/final");
    // Where each must be refused: the line that really can go wrong.
    for (name, phrase) in [
        ("acc_overflows", "acc_overflows.cpls, line 12"),
        ("append_on_a_branch", "append_on_a_branch.cpls, line 18"),
        ("fib_93", "fib_93.cpls, line 8"),
        ("recursion_upward", "recursion_upward.cpls, line 9"),
        ("counter_on_a_branch", "counter_on_a_branch.cpls, line 19"),
    ] {
        let r = final_build(&tests.join(format!("{}.cpls", name)));
        let err = String::from_utf8_lossy(&r.stderr);
        assert!(!r.status.success() && err.contains(phrase), "{}: the final build should refuse it ({}), and said:\n{}", name, phrase, err);
    }
}

/// An unproven narrowing is a check in the working build, not a silent
/// truncation (`conversion.md`: "a runtime check unless Precog proves the
/// fit"). It used to be neither checked nor counted.
#[test]
fn an_unproven_narrowing_is_checked() {
    let ir = ir_of("narrow_checked");
    assert!(ir.contains("the value can be outside what a `uint8` holds"), "a narrowing Precog did not prove reached the binary with no check:\n{}", ir);
}

/// `--assert-proofs` (`precog.md`, attack (d)): every proof becomes a run-
/// time assertion, and none is promised to LLVM, so a wrong proof is a
/// `proof failed:` exit 104 rather than silent undefined behaviour.
#[test]
fn proofs_can_be_asserted() {
    let src = dir().join("proven_add.cpls");
    let out = std::env::temp_dir().join("cca-codegen-tests/ir-asserted");
    let _ = std::fs::create_dir_all(out.parent().unwrap());
    let r = Command::new(cca()).arg("build").arg("--assert-proofs").arg(&src).arg("-o").arg(&out).env("CCA_IR", "1").output().unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let ir = String::from_utf8_lossy(&r.stdout).to_string();
    assert!(!ir.contains(" nsw") && !ir.contains(" nuw"), "a proof was promised to LLVM in the proof-checking build:\n{}", ir);
    assert!(ir.contains("proof failed:"), "a proven site was not asserted:\n{}", ir);
}

fn ir_of(name: &str) -> String {
    let src = dir().join(format!("{}.cpls", name));
    let out = std::env::temp_dir().join(format!("cca-codegen-tests/ir-{}", name));
    let _ = std::fs::create_dir_all(out.parent().unwrap());
    let r = Command::new(cca()).arg("build").arg(&src).arg("-o").arg(&out).env("CCA_IR", "1").output().unwrap();
    assert!(r.status.success(), "{} would not build:\n{}", name, String::from_utf8_lossy(&r.stderr));
    String::from_utf8_lossy(&r.stdout).to_string()
}

fn indent(s: &str) -> String {
    s.lines().map(|l| format!("    {}", l)).collect::<Vec<_>>().join("\n")
}
