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

mod common;

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

/// Check, then lower to MIR and verify the graph.
fn lower(path: &Path) -> Result<(), String> {
    let mut sources = cca::source::SourceMap::default();
    let items = cca::load::load(path, &mut sources).map_err(|d| d.render(&sources))?;
    let mut checker = cca::check::Checker::new(items);
    checker.std_stubs = true;
    let program = checker.check_program().map_err(|d| d.render(&sources))?;
    let mir = cca::mir::build::build(program);
    cca::mir::verify::program(&mir).map_err(|p| p.join("\n"))
}

/// The whole pipeline, up to and including Precog.
fn prove(path: &Path) -> Result<(), String> {
    let mut sources = cca::source::SourceMap::default();
    let items = cca::load::load(path, &mut sources).map_err(|d| d.render(&sources))?;
    let mut checker = cca::check::Checker::new(items);
    checker.std_stubs = true;
    let program = checker.check_program().map_err(|d| d.render(&sources))?;
    let mut mir = cca::mir::build::build(program);
    cca::mir::verify::program(&mir).map_err(|p| p.join("\n"))?;
    let owned = cca::mir::ownership::check(&mut mir, &sources);
    if !owned.problems.is_empty() {
        return Err(owned.problems.join(""));
    }
    let analysis = cca::precog::analyse(&mut mir);
    let refusals = cca::precog::refusals(&analysis, &mir, &sources);
    if refusals.is_empty() {
        Ok(())
    } else {
        Err(refusals.join(""))
    }
}

/// No example may be refused by Precog: they are the language's own
/// programs, and every one of them is meant to build.
///
/// Except where Precog honestly cannot prove what a `check` asks of it yet:
/// those are listed in `examples_unproven.txt` with the reason, and the test
/// insists each is *still* refused — the day Precog proves one, the suite
/// says so and the line goes.
#[test]
fn examples_are_not_refused_by_precog() {
    let pending = load_pending();
    let unproven = load_list("examples_unproven.txt");
    let failures: Vec<String> = common::par_map(&targets(&repo().join("examples")), |src| {
        let name = name_of(src);
        if pending.contains_key(&name) {
            return None;
        }
        match (unproven.get(&name), prove(src)) {
            (None, Err(msg)) => Some(format!("{}:\n{}", name, indent(&msg))),
            (Some(why), Ok(())) => Some(format!("{}: Precog proves it now, and tests/examples_unproven.txt still lists it — drop that line.\n  It said: {}", name, why)),
            _ => None,
        }
    })
    .into_iter()
    .flatten()
    .collect();
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn examples_lower_to_a_well_formed_graph() {
    let pending = load_pending();
    let failures: Vec<String> = common::par_map(&targets(&repo().join("examples")), |src| {
        let name = name_of(src);
        if pending.contains_key(&name) {
            return None;
        }
        lower(src).err().map(|msg| format!("{}:\n{}", name, indent(&msg)))
    })
    .into_iter()
    .flatten()
    .collect();
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

fn targets(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "cpls").unwrap_or(false) || p.is_dir())
        // A folder that only holds Limbs used as dependencies
        // (`examples/limbs/`) is not a program of its own.
        .filter(|p| !(p.is_dir() && !p.join("limb.cpls").is_file() && std::fs::read_dir(p).map(|rd| rd.filter_map(|e| e.ok()).any(|e| e.path().join("limb.cpls").is_file())).unwrap_or(false)))
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
    let pending = load_pending();
    let failures: Vec<String> = common::par_map(&targets(&repo().join("examples")), |src| {
        let name = name_of(src);
        match (pending.get(&name), check(src)) {
            (None, Err(msg)) => Some(format!("{}:\n{}", name, indent(&msg))),
            (Some(why), Ok(())) => Some(format!(
                "{}: checks now, and tests/examples_pending.txt still lists it — drop that line.\n  It said: {}",
                name, why
            )),
            _ => None,
        }
    })
    .into_iter()
    .flatten()
    .collect();
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

/// Examples CCA cannot check yet, with the reason. One entry per example,
/// `name :: why`, continued on indented lines.
fn load_pending() -> HashMap<String, String> {
    load_list("examples_pending.txt")
}

/// `name :: why` lines, a line without `::` continuing the one above.
fn load_list(file: &str) -> HashMap<String, String> {
    let text = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join(file)).unwrap();
    let mut out: HashMap<String, String> = HashMap::new();
    let mut last: Option<String> = None;
    for line in text.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((n, why)) = line.split_once(" :: ") {
            last = Some(n.trim().to_string());
            out.insert(n.trim().to_string(), why.trim().to_string());
        } else if let Some(n) = &last {
            let e = out.get_mut(n).unwrap();
            e.push(' ');
            e.push_str(line.trim());
        }
    }
    out
}

#[test]
fn reject_programs_are_refused_in_ccas_own_words() {
    let phrases = load_phrases();
    let dir = repo().join("oracle/tests/reject");
    let all = targets(&dir);
    let seen: Vec<String> = all.iter().map(|p| name_of(p)).collect();
    let mut failures: Vec<String> = common::par_map(&all, |src| -> Option<String> {
        let name = name_of(src);
        let Some((want, print)) = phrases.get(&name) else {
            return Some(format!("{}: no line in tests/reject_phrases.txt — add one saying what CCA should say", name));
        };
        // The oracle's words moved since the phrase was reviewed: the phrase
        // may no longer say the same thing (owner, 2026-09-24).
        let now = fingerprint(&dir.join(format!("{}.err", name)));
        if print.as_deref() != Some(now.as_str()) {
            return Some(format!("{}: the oracle's .err is not the one this phrase was reviewed against — re-check this phrase against the new .err, then record [fnv:{}]\n  .err now: {}\n  phrase:   {}", name, now, oracle_err(&dir, &name), want));
        }
        let want = want.clone();
        let got = check(src).and_then(|()| prove(src));
        if let Some(why) = want.strip_prefix("!pending") {
            return got.as_ref().err().map(|e| format!("{}: CCA now refuses this, which is progress — replace the `!pending` line in tests/reject_phrases.txt with a phrase of the message.{}\n{}", name, if why.trim().is_empty() { String::new() } else { format!(" It said:{}", why) }, indent(e)));
        }
        if let Some(why) = want.strip_prefix("!by-design") {
            // CCA is right to accept this one; refusing it would be the bug.
            return got.as_ref().err().map(|e| format!("{}: CCA refuses this, and it should build —{}\n{}", name, why, indent(e)));
        }
        match got {
            Ok(()) => Some(format!("{}: accepted, and it must be refused (the oracle says: {})", name, oracle_err(&dir, &name))),
            Err(msg) => {
                let first = msg.lines().next().unwrap_or("").trim_start_matches("error: ").to_string();
                (!first.contains(want.as_str())).then(|| format!("{}: expected a message containing\n    {}\n  got\n{}", name, want, indent(&msg)))
            }
        }
    })
    .into_iter()
    .flatten()
    .collect();
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

/// Each line: the phrase, and the fingerprint of the `.err` it was
/// reviewed against (`name [fnv:HASH] :: phrase`).
fn load_phrases() -> HashMap<String, (String, Option<String>)> {
    let text = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/reject_phrases.txt")).unwrap();
    text.lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| {
            let (n, p) = l.split_once(" :: ").unwrap_or_else(|| panic!("bad line in reject_phrases.txt: {}", l));
            let n = n.trim();
            let (name, print) = match n.split_once(" [fnv:") {
                Some((name, rest)) => (name.trim(), Some(rest.trim_end_matches(']').to_string())),
                None => (n, None),
            };
            (name.to_string(), (p.trim().to_string(), print))
        })
        .collect()
}

/// FNV-1a 64 of a file's bytes: the `.err` a phrase was reviewed against.
fn fingerprint(path: &Path) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in std::fs::read(path).unwrap_or_default() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", h)
}

fn indent(s: &str) -> String {
    s.lines().map(|l| format!("    {}", l)).collect::<Vec<_>>().join("\n")
}
