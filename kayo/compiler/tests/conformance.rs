//! CCA against the oracle's tests (`design/compiler-session.md`).
//!
//! - every `examples/*.nrv` must load and check;
//! - every `oracle/tests/reject/*.nrv` must be refused, with a message in
//!   CCA's own words. The `.err` is the *oracle's* one-liner, not a
//!   template, so the phrase CCA must keep saying is recorded in
//!   `reject_phrases.txt` — reviewed once against the `.err`, and the test
//!   fails when it stops appearing. A program marked `!pending` is one CCA
//!   cannot refuse yet (it needs Precog); the test insists it is *not*
//!   refused, so the line has to be updated when it starts being. One
//!   marked `!unbuilt <ruling>; then: <the line it had>` is written in a
//!   spelling a ruling CCA has not built yet: any refusal passes, and the
//!   day the line after `then:` holds again the test says to drop the mark.
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

/// Load and check one file or Kit directory; `Ok(())` or the message.
fn check(path: &Path) -> Result<(), String> {
    let mut sources = kayo::source::SourceMap::default();
    let items = kayo::load::load(path, &mut sources).map_err(|d| d.render(&sources))?;
    let mut checker = kayo::check::Checker::new(items);
    checker.std_stubs = true;
    checker.check_program().map(|_| ()).map_err(|d| d.render(&sources))
}

/// Check, then lower to MIR and verify the graph.
fn lower(path: &Path) -> Result<(), String> {
    let mut sources = kayo::source::SourceMap::default();
    let items = kayo::load::load(path, &mut sources).map_err(|d| d.render(&sources))?;
    let mut checker = kayo::check::Checker::new(items);
    checker.std_stubs = true;
    let program = checker.check_program().map_err(|d| d.render(&sources))?;
    let mir = kayo::mir::build::build(program);
    kayo::mir::verify::program(&mir).map_err(|p| p.join("\n"))
}

/// The whole pipeline, up to and including Precog.
fn prove(path: &Path) -> Result<(), String> {
    let mut sources = kayo::source::SourceMap::default();
    let items = kayo::load::load(path, &mut sources).map_err(|d| d.render(&sources))?;
    let mut checker = kayo::check::Checker::new(items);
    checker.std_stubs = true;
    let program = checker.check_program().map_err(|d| d.render(&sources))?;
    let mut mir = kayo::mir::build::build(program);
    kayo::mir::verify::program(&mir).map_err(|p| p.join("\n"))?;
    let owned = kayo::mir::ownership::check(&mut mir, &sources);
    if !owned.problems.is_empty() {
        return Err(owned.problems.join(""));
    }
    let analysis = kayo::precog::analyse(&mut mir);
    let refusals = kayo::precog::refusals(&analysis, &mir, &sources);
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
///
/// An example waiting on a `cord` function Synapse does not build
/// (`examples_unbuilt.txt`) is left to the day it builds: its `cord` calls
/// are testing stand-ins whose values Precog cannot know, so what it proves
/// of it means little, and a loop over a stand-in's list of no known length
/// is followed its full 100 000 trips each time it is reached
/// (`corner_shop` took seven minutes so).
#[test]
fn examples_are_not_refused_by_precog() {
    let pending = load_pending();
    let unproven = load_list("examples_unproven.txt");
    let on_stand_ins: std::collections::HashSet<String> = load_list("examples_unbuilt.txt").into_iter().filter(|(_, why)| why.contains("std::")).map(|(n, _)| n).collect();
    let failures: Vec<String> = common::par_map(&targets(&repo().join("examples")), |src| {
        let name = name_of(src);
        if pending.contains_key(&name) || (on_stand_ins.contains(&name) && !unproven.contains_key(&name)) {
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
        // A folder is a program when it holds Nerve source or a manifest;
        // one of files a test reads (`incl_assets/`) is not.
        .filter(|p| p.extension().map(|x| x == "cy").unwrap_or(false) || (p.is_dir() && std::fs::read_dir(p).map(|rd| rd.filter_map(|e| e.ok()).any(|e| e.path().extension().is_some_and(|x| x == "cy" || x == "cyr" || x == "nrv" || x == "nrp"))).unwrap_or(false)))
        // A folder that only holds kits used as dependencies
        // (`examples/kits/`) is not a program of its own.
        .filter(|p| !(p.is_dir() && !p.join("kit.cy").is_file() && std::fs::read_dir(p).map(|rd| rd.filter_map(|e| e.ok()).any(|e| e.path().join("kit.cy").is_file())).unwrap_or(false)))
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
        if let Some(rest) = want.strip_prefix("!unbuilt") {
            let (ruling, then) = rest.split_once("; then: ").unwrap_or((rest, ""));
            if !then.is_empty() && verdict(&name, then, &got, &dir).is_none() {
                return Some(format!("{}: CCA knows the spelling now ({}) and does what the line after `then:` asks — drop the `!unbuilt` mark in tests/reject_phrases.txt", name, ruling.trim()));
            }
            if got.is_ok() && !then.starts_with("!by-design") {
                return Some(format!("{}: accepted, and it must be refused (the oracle says: {})", name, oracle_err(&dir, &name)));
            }
            return None;
        }
        verdict(&name, &want, &got, &dir)
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

/// One reject program's line against what CCA said: `None` when it holds.
fn verdict(name: &str, want: &str, got: &Result<(), String>, dir: &Path) -> Option<String> {
    if let Some(why) = want.strip_prefix("!pending") {
        return got.as_ref().err().map(|e| format!("{}: CCA now refuses this, which is progress — replace the `!pending` line in tests/reject_phrases.txt with a phrase of the message.{}\n{}", name, if why.trim().is_empty() { String::new() } else { format!(" It said:{}", why) }, indent(e)));
    }
    if let Some(why) = want.strip_prefix("!by-design") {
        // CCA is right to accept this one; refusing it would be the bug.
        return got.as_ref().err().map(|e| format!("{}: CCA refuses this, and it should build —{}\n{}", name, why, indent(e)));
    }
    match got {
        Ok(()) => Some(format!("{}: accepted, and it must be refused (the oracle says: {})", name, oracle_err(dir, name))),
        Err(msg) => {
            let first = msg.lines().next().unwrap_or("").trim_start_matches("error: ").to_string();
            (!first.contains(want)).then(|| format!("{}: expected a message containing\n    {}\n  got\n{}", name, want, indent(msg)))
        }
    }
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

/// Every example built and run, its output and exit the oracle's (`.out`,
/// `.exit`), since Synapse builds `cord::print` and `cord::exit` (owner,
/// 2026-10-09: "`b`, not the entire fucking neuron"). One Synapse cannot
/// build yet is listed in `examples_unbuilt.txt` with words of its refusal;
/// the test insists it is still refused so, and the day it builds, says to
/// drop the line.
#[test]
fn examples_run_as_the_oracle_prints() {
    let unbuilt = load_list("examples_unbuilt.txt");
    let unordered = load_list("examples_unordered.txt");
    let synapse = PathBuf::from(env!("CARGO_BIN_EXE_kayo"));
    let dir = repo().join("examples");
    let all: Vec<PathBuf> = targets(&dir).into_iter().filter(|p| dir.join(format!("{}.out", name_of(p))).is_file()).collect();
    let seen: Vec<String> = all.iter().map(|p| name_of(p)).collect();
    let mut failures: Vec<String> = common::par_map(&all, |src| -> Option<String> {
        let name = name_of(src);
        let want = std::fs::read(dir.join(format!("{}.out", name))).unwrap();
        let code = std::fs::read_to_string(dir.join(format!("{}.exit", name))).ok().and_then(|t| t.trim().rsplit(':').next().and_then(|n| n.trim().parse::<i32>().ok())).unwrap_or(0);
        let words: Vec<String> = std::fs::read_to_string(dir.join(format!("{}.args", name))).map(|t| t.lines().filter(|l| !l.is_empty()).map(str::to_string).collect()).unwrap_or_default();
        let mut cmd = Command::new(&synapse);
        cmd.arg("run").arg(src).stdin(std::process::Stdio::null());
        if !words.is_empty() {
            cmd.arg("--").args(&words);
        }
        let r = cmd.output().unwrap();
        let err = String::from_utf8_lossy(&r.stderr);
        let refused = r.stdout.is_empty() && err.lines().any(|l| l.starts_with("error:")) && r.status.code() != Some(code);
        match (unbuilt.get(&name), refused) {
            (Some(why), true) => (!err.contains(why.as_str())).then(|| format!("{}: still not built, but not for the reason tests/examples_unbuilt.txt gives (`{}`):\n{}", name, why, indent(&err))),
            (Some(_), false) => Some(format!("{}: builds now — drop its line in tests/examples_unbuilt.txt (it {} the oracle)", name, if r.stdout == want && r.status.code() == Some(code) { "runs as" } else { "differs from" })),
            (None, true) => Some(format!("{}: refused to build:\n{}", name, indent(&err))),
            (None, false) => {
                // Lines whose order the tasks' timing decides, compared as a set.
                let same = if unordered.contains_key(&name) {
                    let lines = |b: &[u8]| {
                        let mut v: Vec<String> = String::from_utf8_lossy(b).lines().map(str::to_string).collect();
                        v.sort();
                        v
                    };
                    lines(&r.stdout) == lines(&want)
                } else {
                    r.stdout == want
                };
                if !same || r.status.code() != Some(code) {
                    Some(format!("{}: exit {:?} (the oracle's {}), and its output\n{}\n  the oracle's\n{}", name, r.status.code(), code, indent(&String::from_utf8_lossy(&r.stdout)), indent(&String::from_utf8_lossy(&want))))
                } else {
                    None
                }
            }
        }
    })
    .into_iter()
    .flatten()
    .collect();
    for name in unbuilt.keys() {
        if !seen.contains(name) {
            failures.push(format!("{}: listed in tests/examples_unbuilt.txt, but there is no such example with a .out", name));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

use std::process::Command;

/// The examples in `examples_cyborgpl_std.txt`, each built in a Neuron that
/// lists the Nerve cord (`cord/`, by cord's ID and its path; owner,
/// 2026-10-14), print what the oracle prints.
#[test]
fn examples_run_against_the_cyborgpl_std() {
    let synapse = PathBuf::from(env!("CARGO_BIN_EXE_kayo"));
    let text = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/examples_cyborgpl_std.txt")).unwrap();
    let cord = repo().join("std");
    // The sweep to CyborgPL's names (2026-10-15): `cord/` becomes `std/`, std
    // claude's, after KAYO\n's part; until then there is no CyborgPL std to run.
    if !cord.join("CyJet.cyr").is_file() {
        eprintln!("note: no std/ with a CyJet.cyr yet (cord/ is std claude's to rename): examples against the CyborgPL std not run");
        return;
    }
    let mut failures = Vec::new();
    for name in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let dir = std::env::temp_dir().join(format!("cca-nerve-std/{}", name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(repo().join(format!("examples/{}.cy", name)), dir.join("kit.cy")).unwrap();
        let manifest = format!(
            "cyjet::manifest[\n    name = \"{n}\",\n    prefix = \"{n}\",\n    version = \"0.1.0\",\n    dependencies = [\n        cyjet::dep[id = \"fff4302dffc4a894a2bab6baf724eb21dfc05a2ad9818509d8001013d6761936\", path = \"{c}\"],\n    ],\n    targets = [cyjet::target[name = \"aarch64-macos\"],]\n];\n",
            n = name,
            c = cord.display()
        );
        std::fs::write(dir.join("CyJet.cyr"), manifest).unwrap();
        let want = std::fs::read(repo().join(format!("examples/{}.out", name))).unwrap();
        let r = Command::new(&synapse).arg("run").arg(&dir).stdin(std::process::Stdio::null()).output().unwrap();
        if r.stdout != want || r.status.code() != Some(0) {
            failures.push(format!("{}: exit {:?}, and against the CyborgPL std it printed\n{}\n  the oracle's\n{}\n{}", name, r.status.code(), indent(&String::from_utf8_lossy(&r.stdout)), indent(&String::from_utf8_lossy(&want)), indent(&String::from_utf8_lossy(&r.stderr))));
        }
    }
    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}
