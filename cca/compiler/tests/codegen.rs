//! The whole pipeline, all the way to a binary that runs.
//!
//! These programs are CCA's own (`tests/programs/`), not the language's
//! examples: `examples/` belongs to the design session, and every one of
//! them needs either `std::` or a heap type, neither of which has a shape
//! yet. What is here is the base language without a heap — which is
//! exactly what the generated code covers today, so the test says what is
//! true rather than what is hoped.

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

#[test]
fn programs_build_and_run() {
    assert!(runtime().is_some(), "the runtime is not built — run `cargo build -p cca-runtime` first; the linker needs libcca_runtime.a");
    let expected = std::fs::read_to_string(dir().join("expected.txt")).unwrap();
    let out_dir = std::env::temp_dir().join("cca-codegen-tests");
    let _ = std::fs::create_dir_all(&out_dir);
    let mut failures = Vec::new();
    for line in expected.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, want) = line.split_once(' ').unwrap();
        let src = dir().join(format!("{}.cpls", name));
        let bin = out_dir.join(name);
        let build = Command::new(cca()).arg("build").arg(&src).arg("-o").arg(&bin).output().unwrap();
        if !build.status.success() {
            failures.push(format!("{}: would not build:\n{}", name, indent(&String::from_utf8_lossy(&build.stderr))));
            continue;
        }
        let run = Command::new(&bin).output().unwrap();
        let code = run.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&run.stderr).to_string();
        match want {
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
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

/// A guarded value keeps no check, and the proof reaches LLVM.
#[test]
fn a_proven_site_hands_its_fact_to_llvm() {
    let ir = ir_of("proven_add");
    assert!(!ir.contains("with.overflow"), "a site Precog proved still carries a run-time check:\n{}", ir);
    assert!(ir.contains("nsw"), "a proven addition reached LLVM without the fact that it cannot wrap:\n{}", ir);
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

/// Every program gives back everything it took.
///
/// This is the test that catches a heap bug. An exit code is happy to be
/// right while a list's buffer, a map's table or a built text is never
/// freed — or is freed twice, which is worse and usually still exits 0.
/// macOS has `leaks`; where it does not exist the test says so rather
/// than passing quietly.
#[test]
fn programs_give_back_what_they_took() {
    if Command::new("leaks").arg("--help").output().map(|o| !o.status.success()).unwrap_or(true) {
        eprintln!("note: no `leaks` on this machine, so nothing checked what the programs freed");
        return;
    }
    assert!(runtime().is_some(), "the runtime is not built");
    let expected = std::fs::read_to_string(dir().join("expected.txt")).unwrap();
    let out_dir = std::env::temp_dir().join("cca-codegen-tests");
    let _ = std::fs::create_dir_all(&out_dir);
    let mut leaking = Vec::new();
    for line in expected.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, want) = line.split_once(' ').unwrap();
        // A program that stops on a check never reaches the end, so what
        // it was still holding is not a leak.
        if want == "trap" {
            continue;
        }
        let src = dir().join(format!("{}.cpls", name));
        let bin = out_dir.join(name);
        let build = Command::new(cca()).arg("build").arg(&src).arg("-o").arg(&bin).output().unwrap();
        if !build.status.success() {
            continue;
        }
        let r = Command::new("leaks").arg("--atExit").arg("--").arg(&bin).output().unwrap();
        let text = String::from_utf8_lossy(&r.stdout).to_string();
        if !text.contains("0 leaks for 0 total leaked bytes") {
            let said = text.lines().filter(|l| l.contains("leak")).collect::<Vec<_>>().join("\n");
            leaking.push(format!("{}: {}", name, said));
        }
    }
    assert!(leaking.is_empty(), "these programs did not give back everything they took:\n{}", leaking.join("\n"));
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
