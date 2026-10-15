//! Translation validation, the LLVM half (design/verification-session.md,
//! item 4; Alive2 borrowed, owner 2026-09-22 and 2026-09-30): `cca build
//! --emit-llvm` writes the module as CCA hands it to LLVM (`before.ll`) and
//! as the backend gets it (`after.ll`), and Alive2 checks that each function
//! LLVM rewrote still means what it meant — with the flags and facts CCA
//! added from Precog's proofs, which LLVM's rewrites rely on.
//!
//! Two checks per program:
//! - **per function**: every function made external (so no pass may change
//!   a signature for the whole program's sake), then LLVM's function-level
//!   O3 passes with no inlining or attribute inference, each function
//!   checked against its own rewrite. The meaningful one.
//! - **CCA's own pipeline**: `before.ll` against `after.ll`, where most code
//!   was inlined into `main` and is not paired. Only a function that calls
//!   none of the module's own functions is judged there: where a callee was
//!   inlined, Alive2 sees an opaque call in the source and the callee's
//!   body (with whatever it may do) in the target, which always reads as
//!   "source more defined" (seed 27, 2026-09-30) and says nothing either
//!   way. `main` is left out too: LLVM drops its store of argc when nothing
//!   reads it (compiler claude, 2026-09-30), not a miscompile.

use crate::engines::{self, Tools};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

const OPT: &str = "/opt/homebrew/opt/llvm/bin/opt";
const ALIVE: &str = "/opt/homebrew/bin/alive-tv";
/// LLVM's O3 function passes, without inlining or attribute inference
/// (Alive2 21 cannot read `nofree`), and without instcombine's fixpoint check.
const FUNCTION_PASSES: &str = "function(sroa,early-cse<memssa>,simplifycfg,instcombine<no-verify-fixpoint>,correlated-propagation,jump-threading,reassociate,loop-mssa(licm),loop-rotate,loop(indvars,loop-idiom,loop-deletion),loop-unroll<O3>,sccp,gvn,dse,adce,instcombine<no-verify-fixpoint>,simplifycfg)";
const ALIVE_LIMIT: Duration = Duration::from_secs(600);

#[derive(Default, Debug, Clone)]
pub struct Counts {
    pub correct: u64,
    pub incorrect: u64,
    pub unproved: u64,
    pub errors: u64,
}

pub struct Outcome {
    pub per_function: Counts,
    pub pipeline: Counts,
    /// Each incorrect transformation: which check, and Alive2's report of it.
    pub incorrect: Vec<(String, String)>,
    /// Alive2's `ERROR:` lines on the pipeline check (why a function was
    /// not proved, or could not be checked), counted.
    pub reasons: Vec<String>,
}

fn counts(out: &str) -> Counts {
    let n = |what: &str| out.lines().find(|l| l.trim_end().ends_with(what)).and_then(|l| l.split_whitespace().next()).and_then(|v| v.parse().ok()).unwrap_or(0);
    Counts { correct: n("correct transformations"), incorrect: n("incorrect transformations"), unproved: n("failed-to-prove transformations"), errors: n("Alive2 errors") }
}

/// The reports of the transformations Alive2 found incorrect, one per
/// function, leaving out the functions named in `skip`.
fn incorrect_reports(out: &str, skip: &[&str]) -> Vec<String> {
    let mut v = vec![];
    for block in out.split("----------------------------------------") {
        if !block.contains("Transformation doesn't verify!") {
            continue;
        }
        let name = block.lines().find(|l| l.starts_with("define")).and_then(|l| l.split('@').nth(1)).and_then(|l| l.split('(').next()).unwrap_or("");
        if skip.contains(&name) {
            continue;
        }
        v.push(block.trim().to_string());
    }
    v
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
/// parser on Zomp's LLVM 22 IR, owner 2026-10-15), or anything else.
pub enum Fail {
    Unread(String),
    Other(String),
}

fn alive(dir: &Path, src: &Path, tgt: &Path) -> Result<String, Fail> {
    let r = engines::run(Command::new(ALIVE).arg("--smt-to=20000").arg(src).arg(tgt), dir, &[], false, ALIVE_LIMIT);
    if r.timed_out {
        return Err(Fail::Other("alive-tv past its time limit".into()));
    }
    let out = String::from_utf8_lossy(&r.stdout).into_owned() + &r.stderr;
    // No summary: it stopped before checking anything, which is a file it
    // could not read (the first line naming an error says why).
    if !out.contains("Summary:") {
        let why = out.lines().find(|l| l.to_lowercase().contains("error")).or(out.lines().find(|l| !l.trim().is_empty())).unwrap_or("no output");
        return Err(Fail::Unread(why.trim().to_string()));
    }
    Ok(out)
}

/// `per_function` runs the per-function check too; off since 2026-10-15
/// (owner): Homebrew's `opt` is LLVM 23's optimiser, not Zomp's (LLVM 22).
pub fn validate(t: &Tools, src: &str, dir: &Path, per_function: bool) -> Result<Outcome, Fail> {
    let io = |e: std::io::Error| Fail::Other(e.to_string());
    std::fs::create_dir_all(dir).map_err(io)?;
    let file = dir.join("prog.zg");
    std::fs::write(&file, src).map_err(io)?;
    let ir = dir.join("ir");
    let _ = std::fs::remove_dir_all(&ir);
    let r = engines::run(Command::new(&t.cca).arg("build").arg("--emit-llvm").arg(&ir).arg(&file).arg("-o").arg(dir.join("prog")), dir, &[], false, engines::build_limit());
    if r.code != Some(0) || !ir.join("before.ll").exists() {
        return Err(Fail::Other(format!("zomp build did not write the IR: {}", r.stderr.lines().next().unwrap_or(""))));
    }
    let before = std::fs::read_to_string(ir.join("before.ll")).map_err(io)?;
    let a = if per_function {
        let ext = ir.join("external.ll");
        std::fs::write(&ext, before.replace("\ndefine internal ", "\ndefine ")).map_err(io)?;
        let fnl = ir.join("functions.ll");
        let o = engines::run(Command::new(OPT).arg(format!("-passes={FUNCTION_PASSES}")).arg(&ext).arg("-S").arg("-o").arg(&fnl), dir, &[], false, engines::build_limit());
        if o.code != Some(0) {
            return Err(Fail::Other(format!("opt: {}", o.stderr.lines().next().unwrap_or(""))));
        }
        alive(dir, &ext, &fnl)?
    } else {
        String::new()
    };
    let b = alive(dir, &ir.join("before.ll"), &ir.join("after.ll"))?;
    let mut incorrect: Vec<(String, String)> = incorrect_reports(&a, &[]).into_iter().map(|r| ("per function".to_string(), r)).collect();
    let mut skip = callers_of_own(&before);
    skip.push("main".into());
    let skip: Vec<&str> = skip.iter().map(|s| s.as_str()).collect();
    incorrect.extend(incorrect_reports(&b, &skip).into_iter().map(|r| ("Zomp's pipeline".to_string(), r)));
    let mut pipeline = counts(&b);
    // `main`'s known false report is not counted as incorrect
    pipeline.incorrect = incorrect.iter().filter(|x| x.0 == "Zomp's pipeline").count() as u64;
    let reasons = b.lines().filter_map(|l| l.trim().strip_prefix("ERROR: ")).map(|r| r.to_string()).collect();
    Ok(Outcome { per_function: counts(&a), pipeline, incorrect, reasons })
}
