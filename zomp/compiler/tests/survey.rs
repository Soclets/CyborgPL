//! Where CCA stands against the language's own programs: a report, not a
//! pass/fail. `cargo test --release --test survey -- --ignored --nocapture`.
//!
//! Every example is checked, lowered and proven with `std::` stubbed (as the
//! conformance tests do), then taken to machine code, and whatever code
//! generation cannot do yet is listed — std's own gaps apart from the base
//! language's. An example that does not use `std::` is also built, run, and
//! its exit code compared with its `.exit` (0 when there is none).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf()
}

enum Stage {
    Check(String),
    Lower(String),
    Prove(String),
    Codegen(Vec<String>),
    Ok,
}

fn stage(path: &Path) -> Stage {
    let mut sources = ground::source::SourceMap::default();
    let items = match ground::load::load(path, &mut sources) {
        Ok(i) => i,
        Err(d) => return Stage::Check(d.render(&sources)),
    };
    let mut checker = ground::check::Checker::new(items);
    checker.std_stubs = true;
    let program = match checker.check_program() {
        Ok(p) => p,
        Err(d) => return Stage::Check(d.render(&sources)),
    };
    let mut mir = ground::mir::build::build(program);
    if let Err(p) = ground::mir::verify::program(&mir) {
        return Stage::Lower(p.join("\n"));
    }
    let owned = ground::mir::ownership::check(&mut mir, &sources);
    if !owned.problems.is_empty() {
        return Stage::Prove(owned.problems.join(""));
    }
    let analysis = ground::cortex::analyse(&mut mir);
    let refusals = ground::cortex::refusals(&analysis, &mir, &sources);
    if !refusals.is_empty() {
        return Stage::Prove(refusals.join(""));
    }
    let ctx = ground::codegen::inkwell::context::Context::create();
    let stage = match ground::codegen::lower_opts(&ctx, &mir, &sources, "survey", false) {
        Ok(_) => Stage::Ok,
        Err(problems) => Stage::Codegen(problems.into_iter().map(|p| p.what).collect()),
    };
    stage
}

#[test]
#[ignore]
fn survey() {
    let dir = repo().join("examples");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map_or(false, |x| x == "zg") || p.is_dir()).collect();
    files.sort();
    let mut base_gaps: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut std_only = Vec::new();
    let mut builds = Vec::new();
    let mut other = Vec::new();
    let mut no_std_total = 0;
    let mut no_std_pass = Vec::new();
    let mut no_std_fail = Vec::new();
    for f in &files {
        let name = f.file_stem().unwrap().to_string_lossy().to_string();
        let text = if f.is_dir() { String::new() } else { std::fs::read_to_string(f).unwrap_or_default() };
        let uses_std = f.is_dir() || text.contains("std::");
        match stage(f) {
            Stage::Codegen(whats) => {
                let base: Vec<String> = whats.iter().filter(|w| !w.contains("std")).cloned().collect();
                if base.is_empty() {
                    std_only.push(name.clone());
                }
                for w in base {
                    base_gaps.entry(w).or_default().push(name.clone());
                }
            }
            Stage::Ok => builds.push(name.clone()),
            Stage::Check(m) | Stage::Lower(m) | Stage::Prove(m) => other.push(format!("{}: {}", name, m.lines().next().unwrap_or(""))),
        }
        if !uses_std {
            no_std_total += 1;
            let bin = std::env::temp_dir().join(format!("cca-survey-{}", name));
            let out = std::process::Command::new(env!("CARGO_BIN_EXE_zomp")).arg("build").arg(f).arg("-o").arg(&bin).output().unwrap();
            if !out.status.success() {
                no_std_fail.push(format!("{}: does not build", name));
                continue;
            }
            let run = std::process::Command::new(&bin).output().unwrap();
            let want: i32 = std::fs::read_to_string(dir.join(format!("{}.exit", name))).ok().and_then(|s| s.trim().trim_start_matches("exit:").trim().parse().ok()).unwrap_or(0);
            let want_out = std::fs::read_to_string(dir.join(format!("{}.out", name))).unwrap_or_default();
            let got_out = String::from_utf8_lossy(&run.stdout).to_string();
            if run.status.code() == Some(want) && got_out == want_out {
                no_std_pass.push(name.clone());
            } else {
                no_std_fail.push(format!("{}: exit {:?} (want {}), output {}", name, run.status.code(), want, if got_out == want_out { "same" } else { "differs" }));
            }
        }
    }
    println!("\n=== {} examples", files.len());
    println!("builds all the way to machine code (std stubbed): {}", builds.len());
    println!("stopped only by std::: {}", std_only.len());
    println!("stopped before code generation: {}", other.len());
    for o in &other {
        println!("  {}", o);
    }
    println!("\n=== base-language gaps in code generation (feature: examples)");
    for (w, names) in &base_gaps {
        println!("  {}  [{}]: {}", w, names.len(), names.join(", "));
    }
    println!("\n=== examples without std:: : {}, run and match: {}", no_std_total, no_std_pass.len());
    for n in &no_std_fail {
        println!("  {}", n);
    }
}
