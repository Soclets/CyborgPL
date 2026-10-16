//! One program, one or more input sets, both engines, one verdict.
//!
//! What counts as agreement (check/README.md):
//! - both ran: stdout byte for byte, and both exit 0;
//! - both stopped at run time: the same line, and the same stdout before it;
//! - CCA refused to compile at line L and the oracle stopped at line L — a
//!   check that fails on known values is a compile error at the line
//!   (design/precog.md, layer 1), and a program the oracle refuses is one
//!   CCA should refuse too.
//! Anything else is a finding. A "not yet" from CCA is coverage, not a finding.

use crate::engines::*;
use std::fmt::Write as _;
use std::path::Path;

pub type Env = Vec<(String, String)>;

#[derive(Clone, Debug)]
pub struct Finding {
    pub kind: &'static str,
    /// Which build showed it.
    pub mode: Mode,
    /// The kind plus both engines' messages with numbers removed: what
    /// findings are grouped by, so one root cause shows as one line.
    pub sig: String,
    pub env: Env,
    pub detail: String,
}

#[derive(Clone, Debug)]
pub enum Verdict {
    Agree { oracle_stopped: Option<String> },
    /// `cca build` ran past its limit: counted, not a finding.
    SlowBuild,
    NotYet(String),
    Finding(Finding),
}

fn show(b: &[u8]) -> String {
    let s = String::from_utf8_lossy(b);
    if s.len() > 400 {
        format!("{}… ({} bytes)", &s[..400], b.len())
    } else {
        s.into_owned()
    }
}

fn describe_env(env: &Env) -> String {
    if env.is_empty() {
        return "no inputs".into();
    }
    env.iter().map(|(k, v)| format!("{k} = {} characters", v.len())).collect::<Vec<_>>().join(", ")
}

/// Both builds, unless `modes` says otherwise.
pub const BOTH: [Mode; 2] = [Mode::Normal, Mode::Asserted];

pub fn check(tools: &Tools, src: &str, dir: &Path, envs: &[Env]) -> Verdict {
    check_modes(tools, src, dir, envs, &BOTH)
}

/// A generated program: its expectations first, then the comparison. A
/// program built with a borrow mistake on purpose must be refused by both
/// engines (where each points is not compared: which line a borrow conflict
/// is reported at is open with the owner, 2026-09-29). A program whose
/// borrows are right by construction must not be refused for a borrow.
pub fn check_program(tools: &Tools, p: &check_gen::ast::Program, dir: &Path, envs: &[Env]) -> Verdict {
    let src = p.render();
    if !p.expect_refused {
        let v = check(tools, &src, dir, envs);
        if !p.bfuncs.is_empty() {
            let refused_borrow = |m: &str| ["borrow", "given away", "cannot be changed", "lives", "dies"].iter().any(|w| m.contains(w));
            let hit = match &v {
                Verdict::Agree { oracle_stopped: Some(m) } if refused_borrow(m) => Some(format!("oracle: {m}")),
                Verdict::Finding(f) if f.detail.lines().any(|l| l.starts_with("cca build") && refused_borrow(l)) => Some(f.detail.clone()),
                _ => None,
            };
            if let Some(d) = hit {
                let kind = "refused-right-borrow";
                return Verdict::Finding(Finding { kind, mode: Mode::Normal, sig: format!("{kind}: {}", crate::normalise(d.lines().last().unwrap_or(""))), env: vec![], detail: d });
            }
        }
        return v;
    }
    std::fs::create_dir_all(dir).unwrap();
    let file = dir.join("prog.cy");
    std::fs::write(&file, &src).unwrap();
    let env: Env = envs.first().cloned().unwrap_or_default();
    let mut d = String::new();
    let o = oracle(tools, &file, dir, &env);
    let _ = writeln!(d, "a mistake on purpose (a borrow, or a plain let over a visible name); oracle: {}", oracle_text(&o));
    let b = cca_build(tools, &file, dir, Mode::Normal);
    match &b {
        Build::Refused { line, msg } => {
            let _ = writeln!(d, "cca build: refused at line {} — {msg}", opt(*line));
        }
        Build::Built(_) => {
            let _ = writeln!(d, "cca build: built it");
        }
        Build::NotYet { msg } => return Verdict::NotYet(msg.clone()),
        Build::Timeout => return Verdict::SlowBuild,
    }
    let kind = match (&o, &b) {
        (Oracle::Ran { .. }, Build::Built(_)) => "both-accepted-mistake",
        (Oracle::Ran { .. }, _) => "oracle-accepted-mistake",
        (_, Build::Built(_)) => "cca-accepted-mistake",
        (Oracle::Timeout, _) => "oracle-timeout",
        // both refused it, as they should: say so, not "both ran"
        _ => return Verdict::Agree { oracle_stopped: Some(format!("a mistake on purpose, refused by both: {}", oracle_text(&o))) },
    };
    Verdict::Finding(Finding { kind, mode: Mode::Normal, sig: kind.to_string(), env, detail: d })
}

pub fn check_modes(tools: &Tools, src: &str, dir: &Path, envs: &[Env], modes: &[Mode]) -> Verdict {
    std::fs::create_dir_all(dir).unwrap();
    let file = dir.join("prog.cy");
    std::fs::write(&file, src).unwrap();
    let envs: Vec<Env> = if envs.is_empty() { vec![vec![]] } else { envs.to_vec() };
    let mut builds = vec![];
    for m in modes {
        match cca_build(tools, &file, dir, *m) {
            Build::NotYet { msg } => return Verdict::NotYet(msg),
            Build::Timeout => return Verdict::SlowBuild,
            b => builds.push((*m, b)),
        }
    }
    let mut stopped = None;
    for env in &envs {
        let o = oracle(tools, &file, dir, env);
        if let Oracle::Stopped { msg, .. } = &o {
            stopped = Some(msg.clone());
        }
        for (mode, build) in &builds {
            if let Some(f) = compare_one(&o, build, *mode, dir, env) {
                return Verdict::Finding(f);
            }
        }
    }
    Verdict::Agree { oracle_stopped: stopped }
}

fn compare_one(o: &Oracle, build: &Build, mode: Mode, dir: &Path, env: &Env) -> Option<Finding> {
    let tag = if mode == Mode::Asserted { " --assert-proofs" } else { "" };
    let mut d = String::new();
    let _ = writeln!(d, "inputs: {}", describe_env(env));
    let _ = writeln!(d, "oracle: {}", oracle_text(o));
    let finding = |kind: &'static str, d: String| {
        let msgs: Vec<String> = d
            .lines()
            .skip(1)
            .map(|l| {
                let (who, rest) = l.split_once(": ").unwrap_or((l, ""));
                let what = rest.split(" — ").nth(1).unwrap_or(rest.split(',').next().unwrap_or(""));
                format!("{who}: {}", crate::normalise(what.split("; stdout").next().unwrap_or("")))
            })
            .collect();
        Some(Finding { kind, mode, sig: format!("{kind}: {}", msgs.join(" | ")), env: env.clone(), detail: d })
    };
    match build {
        Build::NotYet { .. } | Build::Timeout => unreachable!(),
        Build::Refused { line, msg } => {
            let _ = writeln!(d, "cca build{tag}: refused at line {} — {msg}", opt(*line));
            match o {
                Oracle::Stopped { line: m, .. } if m == line && line.is_some() => None,
                // CCA refuses a site that fails whenever it is reached; the
                // oracle stopped at an earlier line first (a run-time check
                // CCA keeps, since that one may never run) and so never got
                // there. Provisional (2026-09-29, seed 4091): earlier by line
                // number, which a loop or a call can get wrong.
                Oracle::Stopped { line: Some(m), .. } if msg.starts_with("error:") && line.map_or(false, |l| *m < l) => None,
                Oracle::Stopped { .. } => finding("different-site", d),
                Oracle::Ran { .. } => finding("cca-refused-oracle-ran", d),
                Oracle::Timeout => finding("oracle-timeout", d),
            }
        }
        Build::Built(bin) => {
            let e = cca_run(bin, dir, env);
            let _ = writeln!(d, "cca binary{tag}: {}", exec_text(&e));
            if let Exec::Stopped { code: Some(104), .. } = e {
                // A proof CCA made was wrong on these inputs: the worst kind.
                return finding("proof-failed", d);
            }
            match (o, &e) {
                (Oracle::Ran { out: a }, Exec::Ran { out: b }) => (a != b).then(|| ()).and_then(|_| finding("output", d)),
                (Oracle::Stopped { out: a, line: m, .. }, Exec::Stopped { out: b, line: l, .. }) => {
                    if m != l || m.is_none() {
                        finding("different-site", d)
                    } else if a != b {
                        finding("output-before-stop", d)
                    } else {
                        None
                    }
                }
                (Oracle::Stopped { .. }, Exec::Ran { .. }) => finding("oracle-stopped-cca-ran", d),
                (Oracle::Ran { .. }, Exec::Stopped { .. }) => finding("cca-stopped-oracle-ran", d),
                (Oracle::Timeout, Exec::Timeout) => None,
                (Oracle::Timeout, _) => finding("oracle-timeout", d),
                (_, Exec::Timeout) => finding("cca-run-timeout", d),
            }
        }
    }
}

fn opt(l: Option<u32>) -> String {
    l.map(|l| l.to_string()).unwrap_or("?".into())
}

pub fn oracle_text(o: &Oracle) -> String {
    match o {
        Oracle::Ran { out } => format!("ran, exit 0, stdout {:?}", show(out)),
        Oracle::Stopped { out, line, msg } => format!("stopped at line {} — {msg}; stdout before it {:?}", opt(*line), show(out)),
        Oracle::Timeout => "timed out".into(),
    }
}

pub fn exec_text(e: &Exec) -> String {
    match e {
        Exec::Ran { out } => format!("ran, exit 0, stdout {:?}", show(out)),
        Exec::Stopped { out, code, line, msg } => {
            format!("stopped (exit {}) at line {} — {msg}; stdout before it {:?}", code.map(|c| c.to_string()).unwrap_or("signal".into()), opt(*line), show(out))
        }
        Exec::Timeout => "timed out".into(),
    }
}
