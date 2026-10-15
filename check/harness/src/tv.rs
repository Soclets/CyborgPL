//! Translation validation, the LLVM half (design/verification-session.md,
//! item 4; Alive2 borrowed, owner 2026-09-22 and 2026-09-30; tried again on
//! Zomp, owner 2026-10-15: "(c)", then "(a)").
//!
//! `zomp build --emit-llvm-tv <dir>` (c530643c) writes the module as Zomp
//! hands it to LLVM (`tv-before.ll`) and after Zomp's own LLVM 22 passes with
//! inlining off (`tv-after.ll`): every defined function `noinline` and
//! external, the same functions and signatures on both sides, and what the
//! passes inferred taken off again. Alive2 checks each function against its
//! own rewrite, with the flags and facts Zomp added from its prover's proofs,
//! which LLVM's rewrites rely on.
//!
//! Alive2 is Homebrew's 21.0 (built on LLVM 23), which cannot check LLVM 22's
//! `llvm.lifetime.start/end(ptr)` with no size: those calls are taken out of
//! both files here (compiler claude: on this side, so `tv-after.ll` stays
//! what the real pipeline makes). `main` is judged but never counted as
//! incorrect: LLVM drops its store of argc when nothing reads it (compiler
//! claude, 2026-09-30), not a miscompile.

use crate::engines::{self, Tools};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

const ALIVE: &str = "/opt/homebrew/bin/alive-tv";
const ALIVE_LIMIT: Duration = Duration::from_secs(600);

#[derive(Default, Debug, Clone)]
pub struct Counts {
    pub correct: u64,
    pub incorrect: u64,
    pub unproved: u64,
    pub errors: u64,
}

pub struct Outcome {
    pub counts: Counts,
    /// Each incorrect transformation: the function, and Alive2's report of it.
    pub incorrect: Vec<(String, String)>,
    /// Alive2's `ERROR:` lines (why a function was not proved, or could not
    /// be checked), one per function.
    pub reasons: Vec<String>,
    /// alive-tv stopped before its summary; the counts are what it reached.
    pub stopped: bool,
    /// Incorrect reports in functions that call one of the module's own
    /// functions: LLVM's interprocedural passes use a callee's body even when
    /// it is `noinline` (a callee that returns its argument, one that ignores
    /// an argument, so `poison` is passed), which Alive2, checking the caller
    /// alone, reads as wrong (seeds 3, 32, 35, 50, 58, 2026-10-15). Not judged.
    pub leaning: Vec<(String, String)>,
}

/// The functions in `ir` that call a function the module itself defines.
fn callers_of_own(ir: &str) -> Vec<String> {
    let name_of = |l: &str| l.split('@').nth(1).and_then(|l| l.split('(').next()).unwrap_or("").to_string();
    let defined: Vec<String> = ir.lines().filter(|l| l.starts_with("define")).map(name_of).collect();
    let mut out = vec![];
    let mut cur: Option<String> = None;
    for l in ir.lines() {
        if l.starts_with("define") {
            cur = Some(name_of(l));
        } else if l.starts_with('}') {
            cur = None;
        } else if let Some(c) = &cur {
            if l.contains("call ") && defined.iter().any(|d| l.contains(&format!("@{d}("))) && !out.contains(c) {
                out.push(c.clone());
            }
        }
    }
    out
}

/// What `validate` could not do: Alive2 could not read the IR (LLVM 23's
/// parser on Zomp's LLVM 22 IR), or anything else.
pub enum Fail {
    Unread(String),
    Other(String),
}

/// The verdicts, from the summary, or from the functions' own blocks when
/// alive-tv stopped before its summary (after a timeout, say).
fn counts(out: &str) -> Counts {
    if out.contains("Summary:") {
        let n = |what: &str| out.lines().find(|l| l.trim_end().ends_with(what)).and_then(|l| l.split_whitespace().next()).and_then(|v| v.parse().ok()).unwrap_or(0);
        return Counts { correct: n("correct transformations"), incorrect: n("incorrect transformations"), unproved: n("failed-to-prove transformations"), errors: n("Alive2 errors") };
    }
    let mut c = Counts::default();
    for block in out.split("----------------------------------------") {
        if block.contains("Transformation seems to be correct!") {
            c.correct += 1;
        } else if block.contains("Transformation doesn't verify!") {
            c.incorrect += 1;
        } else if block.contains("ERROR: ") {
            c.unproved += 1;
        }
    }
    c
}

/// The reports of the transformations Alive2 found incorrect, one per
/// function, with the function's name.
fn incorrect_reports(out: &str) -> Vec<(String, String)> {
    let mut v = vec![];
    for block in out.split("----------------------------------------") {
        if !block.contains("Transformation doesn't verify!") {
            continue;
        }
        let name = block.lines().find(|l| l.starts_with("define")).and_then(|l| l.split('@').nth(1)).and_then(|l| l.split('(').next()).unwrap_or("");
        v.push((name.to_string(), block.trim().to_string()));
    }
    v
}

/// The module without `llvm.lifetime.start/end` calls and declarations.
fn strip_lifetimes(ir: &str) -> String {
    ir.lines().filter(|l| !l.contains("@llvm.lifetime.start") && !l.contains("@llvm.lifetime.end")).map(|l| format!("{l}\n")).collect()
}

/// alive-tv's output, and whether it stopped before its summary.
fn alive(dir: &Path, src: &Path, tgt: &Path) -> Result<(String, bool), Fail> {
    let r = engines::run(Command::new(ALIVE).arg("--smt-to=20000").arg(src).arg(tgt), dir, &[], false, ALIVE_LIMIT);
    if r.timed_out {
        return Err(Fail::Other("alive-tv past its time limit".into()));
    }
    let out = String::from_utf8_lossy(&r.stdout).into_owned() + &r.stderr;
    // No summary and no function reached: it stopped before checking
    // anything, which is a file it could not read (the first line naming an
    // error says why). With functions but no summary, it stopped part way.
    if !out.contains("Summary:") && !out.lines().any(|l| l.starts_with("define ")) {
        let why = out.lines().find(|l| l.to_lowercase().contains("error")).or(out.lines().find(|l| !l.trim().is_empty())).unwrap_or("no output");
        return Err(Fail::Unread(why.trim().to_string()));
    }
    let stopped = !out.contains("Summary:");
    Ok((out, stopped))
}

pub fn validate(t: &Tools, src: &str, dir: &Path) -> Result<Outcome, Fail> {
    let io = |e: std::io::Error| Fail::Other(e.to_string());
    std::fs::create_dir_all(dir).map_err(io)?;
    let file = dir.join("prog.zg");
    std::fs::write(&file, src).map_err(io)?;
    let ir = dir.join("ir");
    let _ = std::fs::remove_dir_all(&ir);
    let r = engines::run(Command::new(&t.cca).arg("build").arg(&file).arg("-o").arg(dir.join("prog")).arg("--emit-llvm-tv").arg(&ir), dir, &[], false, engines::build_limit());
    if r.code != Some(0) || !ir.join("tv-before.ll").exists() || !ir.join("tv-after.ll").exists() {
        return Err(Fail::Other(format!("zomp build did not write the IR: {}", r.stderr.lines().next().unwrap_or(""))));
    }
    let (src_ll, tgt_ll) = (ir.join("before.ll"), ir.join("after.ll"));
    for (from, to) in [("tv-before.ll", &src_ll), ("tv-after.ll", &tgt_ll)] {
        let text = std::fs::read_to_string(ir.join(from)).map_err(io)?;
        std::fs::write(to, strip_lifetimes(&text)).map_err(io)?;
    }
    let (out, stopped) = alive(dir, &src_ll, &tgt_ll)?;
    let callers = callers_of_own(&std::fs::read_to_string(&src_ll).map_err(io)?);
    let (leaning, incorrect): (Vec<(String, String)>, Vec<(String, String)>) =
        incorrect_reports(&out).into_iter().filter(|(name, _)| name != "main").partition(|(name, _)| callers.contains(name));
    let mut c = counts(&out);
    c.incorrect = incorrect.len() as u64;
    let reasons = out.lines().filter_map(|l| l.trim().strip_prefix("ERROR: ")).map(|r| r.to_string()).collect();
    Ok(Outcome { counts: c, incorrect, reasons, stopped, leaning })
}
