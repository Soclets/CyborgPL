//! CCA against the oracle's tests (`design/compiler-session.md`).
//!
//! - every `examples/*.cpls` must load and check;
//! - every `oracle/tests/reject/*.cpls` must be refused, with a message in
//!   CCA's own words. The `.err` is the *oracle's* one-liner, not a
//!   template, so the phrase CCA must keep saying is recorded in
//!   `reject_phrases.txt` — reviewed once against the `.err`, and the test
//!   fails when it stops appearing. A program marked `!pending` is one CCA
//!   cannot refuse yet (it needs Precog); the test insists it is *not*
//!   refused, so the line has to be updated when it starts being.
//!
//! Printing examples need `std::`, which does not exist yet (owner,
//! 2026-09-22). The checker has a testing-only stub table for them so the
//! front end can be tested now; nothing is generated for a `std::` call.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}

/// Load and check one file or Limb directory; `Ok(())` or the message.
fn check(path: &Path) -> Result<(), String> {
    let mut sources = cca::source::SourceMap::default();
    let items = cca::load::load(path, &mut sources).map_err(|d| d.render(&sources))?;
    let mut checker = cca::check::Checker::new(items);
    checker.std_stubs = true;
    checker.check_program().map(|_| ()).map_err(|d| d.render(&sources))
}

fn targets(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "cpls").unwrap_or(false) || p.is_dir())
        .collect();
    v.sort();
    v
}

fn name_of(p: &Path) -> String {
    if p.is_dir() {
        p.file_name().unwrap().to_string_lossy().to_string()
    } else {
        p.file_stem().unwrap().to_string_lossy().to_string()
    }
}

#[test]
fn examples_load_and_check() {
    let mut failures = Vec::new();
    for src in targets(&repo().join("examples")) {
        if let Err(msg) = check(&src) {
            failures.push(format!("{}:\n{}", name_of(&src), indent(&msg)));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn reject_programs_are_refused_in_ccas_own_words() {
    let phrases = load_phrases();
    let dir = repo().join("oracle/tests/reject");
    let mut failures = Vec::new();
    let mut seen = Vec::new();
    for src in targets(&dir) {
        let name = name_of(&src);
        seen.push(name.clone());
        let Some(want) = phrases.get(&name) else {
            failures.push(format!("{}: no line in tests/reject_phrases.txt — add one saying what CCA should say", name));
            continue;
        };
        let got = check(&src);
        if want == "!pending" {
            if got.is_err() {
                failures.push(format!("{}: CCA now refuses this, which is progress — replace the `!pending` line in tests/reject_phrases.txt with a phrase of the message:\n{}", name, indent(got.as_ref().unwrap_err())));
            }
            continue;
        }
        match got {
            Ok(()) => failures.push(format!("{}: accepted, and it must be refused (the oracle says: {})", name, oracle_err(&dir, &name))),
            Err(msg) => {
                let first = msg.lines().next().unwrap_or("").trim_start_matches("error: ").to_string();
                if !first.contains(want.as_str()) {
                    failures.push(format!("{}: expected a message containing\n    {}\n  got\n{}", name, want, indent(&msg)));
                }
            }
        }
    }
    for name in phrases.keys() {
        if !seen.contains(name) {
            failures.push(format!("{}: listed in tests/reject_phrases.txt, but there is no such reject test", name));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

fn oracle_err(dir: &Path, name: &str) -> String {
    std::fs::read_to_string(dir.join(format!("{}.err", name))).map(|s| s.trim().to_string()).unwrap_or_default()
}

fn load_phrases() -> HashMap<String, String> {
    let text = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/reject_phrases.txt")).unwrap();
    text.lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| {
            let (n, p) = l.split_once(" :: ").unwrap_or_else(|| panic!("bad line in reject_phrases.txt: {}", l));
            (n.trim().to_string(), p.trim().to_string())
        })
        .collect()
}

fn indent(s: &str) -> String {
    s.lines().map(|l| format!("    {}", l)).collect::<Vec<_>>().join("\n")
}
