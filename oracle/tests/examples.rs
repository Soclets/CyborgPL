//! Every `examples/*.zg` must produce its `.out` (and its `.exit` code, if
//! present). Every `tests/reject/*.zg` must fail with a message containing
//! its `.err`. Run: `cargo test` in oracle/.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn oracle() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_oracle"))
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

/// `.zg` files, plus Limb directories (a directory holding `strike.zg`);
/// a directory's expected `.out` / `.err` sits beside it, named after it.
fn cpls_files(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(dir).unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "zg").unwrap_or(false) || p.join("strike.zg").is_file())
        .collect();
    v.sort();
    v
}

#[test]
fn examples_produce_expected_output() {
    let dir = root().join("examples");
    let mut failures = Vec::new();
    for src in cpls_files(&dir) {
        let want = fs::read_to_string(src.with_extension("out")).unwrap_or_else(|_| panic!("{} has no .out", src.display()));
        let want_code: i32 = fs::read_to_string(src.with_extension("exit")).ok()
            .and_then(|s| s.trim().strip_prefix("exit:").and_then(|n| n.parse().ok())).unwrap_or(0);
        // a `.args` beside it: the words after `--`, one per line, for MAIN's argc/argv (owner, 2026-09-29)
        let words: Vec<String> = fs::read_to_string(src.with_extension("args")).map(|t| t.lines().map(str::to_string).collect()).unwrap_or_default();
        let out = Command::new(oracle()).arg(&src).args(if words.is_empty() { vec![] } else { std::iter::once("--".to_string()).chain(words).collect() }).output().unwrap();
        let got = String::from_utf8_lossy(&out.stdout).to_string();
        let code = out.status.code().unwrap_or(-1);
        // a `.test-out` beside it: its `test fn`s, run by `oracle --test` (owner, 2026-09-29), print that
        if let Ok(want_tests) = fs::read_to_string(src.with_extension("test-out")) {
            let t = Command::new(oracle()).arg("--test").arg(&src).output().unwrap();
            let got_tests = String::from_utf8_lossy(&t.stdout).to_string();
            if got_tests != want_tests {
                failures.push(format!("{} (--test):\n  got:\n{got_tests}\n  want:\n{want_tests}\n  stderr: {}", src.file_name().unwrap().to_string_lossy(), String::from_utf8_lossy(&t.stderr)));
            }
        }
        if got != want || code != want_code {
            failures.push(format!("{}:\n  exit {code} (want {want_code})\n  stderr: {}\n  got:\n{got}\n  want:\n{want}",
                src.file_name().unwrap().to_string_lossy(), String::from_utf8_lossy(&out.stderr), ));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn rejected_programs_fail_with_the_right_message() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/reject");
    let mut failures = Vec::new();
    for src in cpls_files(&dir) {
        let want = fs::read_to_string(src.with_extension("err")).unwrap_or_else(|_| panic!("{} has no .err", src.display()));
        let want = want.trim();
        let out = Command::new(oracle()).arg(&src).output().unwrap();
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        let code = out.status.code().unwrap_or(-1);
        if code != 1 {
            failures.push(format!("{}: expected rejection (exit 1), got exit {code}; stderr: {stderr}", src.file_name().unwrap().to_string_lossy()));
        } else if !stderr.contains(want) {
            failures.push(format!("{}: expected error containing `{want}`, got: {stderr}", src.file_name().unwrap().to_string_lossy()));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

/// A file named bare, run from its own folder (`cd dir && oracle t.zg`), finds what it includes there
/// too (compiler claude, 2026-10-10: the empty path reached Clang as its input).
#[test]
fn a_bare_file_name_includes_from_its_folder() {
    let dir = root().join("examples");
    let out = Command::new(oracle()).current_dir(&dir).arg("c_headers.zg").output().unwrap();
    let want = fs::read_to_string(dir.join("c_headers.out")).unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout), want, "stderr: {}", String::from_utf8_lossy(&out.stderr));
}
