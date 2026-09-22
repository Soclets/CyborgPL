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

/// What the generated code proves away is not a matter of taste: a program
/// whose values are all known has no checks left in it at all.
#[test]
fn a_known_program_keeps_no_checks() {
    let src = dir().join("arithmetic.cpls");
    let out = std::env::temp_dir().join("cca-codegen-tests/ir-probe");
    let _ = std::fs::create_dir_all(out.parent().unwrap());
    let r = Command::new(cca()).arg("build").arg(&src).arg("-o").arg(&out).env("CCA_IR", "1").output().unwrap();
    let ir = String::from_utf8_lossy(&r.stdout).to_string();
    assert!(r.status.success(), "would not build:\n{}", String::from_utf8_lossy(&r.stderr));
    assert!(!ir.contains("with.overflow"), "a program Precog proved still carries a run-time check:\n{}", ir);
    // And the proof went to LLVM, which is the point of having it.
    assert!(ir.contains("nsw"), "a proven addition reached LLVM without the fact that it cannot wrap:\n{}", ir);
}

fn indent(s: &str) -> String {
    s.lines().map(|l| format!("    {}", l)).collect::<Vec<_>>().join("\n")
}
