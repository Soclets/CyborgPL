//! Running the two engines from the outside: the oracle binary and
//! `cca build` + the binary it writes. Nothing here reads either engine's
//! source; only their output, exit codes and messages.

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub struct Tools {
    pub oracle: PathBuf,
    pub cca: PathBuf,
}

impl Tools {
    /// The binaries built in this repository (`cargo build --release` in
    /// `oracle/` and `kayo/`). `cca` is `KAYO\n`, the compiler (CCA, Synapse, then Zomp, until
    /// the owner's rename, 2026-10-09).
    pub fn find(repo: &Path) -> Result<Tools, String> {
        let oracle = repo.join("oracle/target/release/oracle");
        let cca = repo.join("kayo/target/release/kayo");
        for (p, dir) in [(&oracle, "oracle"), (&cca, "kayo")] {
            if !p.exists() {
                return Err(format!("{} not found; run `cargo build --release` in {dir}/", p.display()));
            }
        }
        Ok(Tools { oracle, cca })
    }

    /// The oracle alone, for the commands that never run the compiler
    /// (evaltest): they work while `kayo/` is not built.
    pub fn find_oracle(repo: &Path) -> Result<Tools, String> {
        let oracle = repo.join("oracle/target/release/oracle");
        if !oracle.exists() {
            return Err(format!("{} not found; run `cargo build --release` in oracle/", oracle.display()));
        }
        Ok(Tools { oracle, cca: repo.join("kayo/target/release/kayo") })
    }
}

/// Wall-clock limits — a safety net against a hang, never a verdict of
/// agreement (provisional; see check/README.md). A build may legitimately be
/// slow (compile time is not a concern, design/core-features.md), so it gets
/// longer and running past it is counted, not reported as a finding.
pub const RUN_LIMIT: Duration = Duration::from_secs(20);
pub const BUILD_LIMIT: Duration = Duration::from_secs(120);
/// Z3 on one program's claims (the Precog oracle, attack (c)): it runs in
/// the background, so it gets longer than a build.
pub const CLAIM_LIMIT: Duration = Duration::from_secs(600);
/// While shrinking, a candidate slower than this is simply not kept.
pub const SHRINK_BUILD_LIMIT: Duration = Duration::from_secs(30);

static SHRINKING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_shrinking(on: bool) {
    SHRINKING.store(on, Ordering::SeqCst);
}

pub fn build_limit() -> Duration {
    if SHRINKING.load(Ordering::SeqCst) {
        SHRINK_BUILD_LIMIT
    } else {
        BUILD_LIMIT
    }
}

pub struct Run {
    pub stdout: Vec<u8>,
    pub stderr: String,
    /// None: killed at the time limit, or ended by a signal.
    pub code: Option<i32>,
    pub timed_out: bool,
}

static SERIAL: AtomicU64 = AtomicU64::new(0);

/// Runs `cmd` in a process group of its own. A program run (`clean`) sees
/// only the harness's own variables, so a seed replays exactly whatever the
/// shell has set; a build keeps the environment (it needs the linker on PATH).
///
/// Each run writes to fresh files, and the whole group is killed when the
/// run ends: `cca build` starts solver processes, and one left behind (say by
/// a build stopped at its limit) once kept writing into the next run's
/// stdout, which showed up as NUL bytes and a false "output" finding.
pub fn run(cmd: &mut Command, dir: &Path, env: &[(String, String)], clean: bool, limit: Duration) -> Run {
    let n = SERIAL.fetch_add(1, Ordering::SeqCst);
    let out_path = dir.join(format!("stdout.{n}"));
    let err_path = dir.join(format!("stderr.{n}"));
    let out = File::create(&out_path).expect("stdout file");
    let err = File::create(&err_path).expect("stderr file");
    if clean {
        cmd.env_clear();
    }
    for (k, v) in env {
        cmd.env(k, v);
    }
    let mut child = cmd.process_group(0).stdin(Stdio::null()).stdout(out).stderr(err).spawn().expect("spawn");
    let pgid = child.id();
    // Note the group, so a harness that is itself killed does not leave it
    // running: the next harness start kills whatever is listed.
    let mark = live_dir().join(pgid.to_string());
    let _ = fs::write(&mark, std::process::id().to_string());
    let start = Instant::now();
    let mut timed_out = false;
    let status = loop {
        if let Some(s) = child.try_wait().expect("wait") {
            break Some(s);
        }
        if start.elapsed() > limit {
            timed_out = true;
            break None;
        }
        std::thread::sleep(Duration::from_millis(2));
    };
    // Whatever is left of the group — the child itself on a timeout, or
    // anything it started — goes now.
    kill_group(pgid);
    let _ = fs::remove_file(&mark);
    if timed_out {
        let _ = child.wait();
    }
    let r = Run {
        stdout: fs::read(&out_path).unwrap_or_default(),
        stderr: String::from_utf8_lossy(&fs::read(&err_path).unwrap_or_default()).into_owned(),
        code: status.and_then(|s| s.code()),
        timed_out,
    };
    let _ = fs::remove_file(&out_path);
    let _ = fs::remove_file(&err_path);
    r
}

fn live_dir() -> std::path::PathBuf {
    let d = Path::new(env!("CARGO_MANIFEST_DIR")).join("../work/live");
    let _ = fs::create_dir_all(&d);
    d
}

fn kill_group(pgid: u32) {
    let _ = Command::new("/bin/kill").args(["-9", "--", &format!("-{pgid}")]).stdout(Stdio::null()).stderr(Stdio::null()).status();
}

/// Kill every engine process group whose harness is gone (stopped before it
/// could clean up). Groups of a harness still running are left alone.
pub fn reap_leftovers() {
    let Ok(rd) = fs::read_dir(live_dir()) else { return };
    for e in rd.flatten() {
        let owner = fs::read_to_string(e.path()).unwrap_or_default();
        let alive = !owner.is_empty()
            && Command::new("/bin/kill").args(["-0", owner.trim()]).stdout(Stdio::null()).stderr(Stdio::null()).status().map_or(false, |s| s.success());
        if alive {
            continue;
        }
        if let Ok(pgid) = e.file_name().to_string_lossy().parse::<u32>() {
            kill_group(pgid);
        }
        let _ = fs::remove_file(e.path());
    }
}

/// Attack (c): ask Z3 about every site of a program (check_gen::solve).
/// Returns, per site, a failing input when Z3 found one. `None` overall when
/// the program is too big to translate or Z3 did not answer.
pub fn aim(p: &check_gen::Program, dir: &Path) -> Option<(Vec<check_gen::solve::Site>, Vec<check_gen::solve::Answer>)> {
    let q = check_gen::solve::encode(p)?;
    fs::create_dir_all(dir).ok()?;
    let f = dir.join("aim.smt2");
    fs::write(&f, &q.script).ok()?;
    let r = run(Command::new("z3").arg("-smt2").arg(&f), dir, &[("PATH".into(), "/opt/homebrew/bin:/usr/bin:/bin".into())], false, BUILD_LIMIT);
    // On a time-out, the answers given so far still count.
    let out = String::from_utf8_lossy(&r.stdout).into_owned();
    let answers = check_gen::solve::parse_answers(&out, q.sites.len(), q.inputs.len());
    Some((q.sites, answers))
}

/// The first `line N` in a message.
pub fn line_of(msg: &str) -> Option<u32> {
    let i = msg.find("line ")?;
    let digits: String = msg[i + 5..].chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

#[derive(Clone, Debug, PartialEq)]
pub enum Oracle {
    Ran { out: Vec<u8> },
    /// Refused or stopped at a failing check: the oracle does not say which,
    /// so the harness never relies on the difference.
    Stopped { out: Vec<u8>, line: Option<u32>, msg: String },
    Timeout,
}

pub fn oracle(t: &Tools, file: &Path, dir: &Path, env: &[(String, String)]) -> Oracle {
    let r = run(Command::new(&t.oracle).arg(file), dir, env, true, RUN_LIMIT);
    if r.timed_out {
        return Oracle::Timeout;
    }
    match r.code {
        Some(0) => Oracle::Ran { out: r.stdout },
        _ => {
            let first = r.stderr.lines().next().unwrap_or("").to_string();
            // drop the file name the oracle puts in front
            let msg = first.split_once(": ").map(|(_, m)| m.to_string()).unwrap_or(first.clone());
            Oracle::Stopped { out: r.stdout, line: line_of(&first), msg }
        }
    }
}

/// Messages by which CCA says a construct is not built yet: coverage, not a finding.
const NOT_YET: [&str; 5] = ["not yet", "is not available yet", "is not built yet", "CCA carries whole numbers up to", "has no shape in the generated code"];

#[derive(Clone, Debug, PartialEq)]
pub enum Build {
    Built(PathBuf),
    NotYet { msg: String },
    Refused { line: Option<u32>, msg: String },
    Timeout,
}

/// `Asserted` is `cca build --assert-proofs` (design/precog.md attack (d)):
/// every proven site keeps a run-time check that exits 104 with
/// `proof failed:` if the proof was wrong.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Normal,
    Asserted,
}

pub fn cca_build(t: &Tools, file: &Path, dir: &Path, mode: Mode) -> Build {
    let bin = dir.join(if mode == Mode::Asserted { "prog-asserted" } else { "prog" });
    let _ = fs::remove_file(&bin);
    let mut cmd = Command::new(&t.cca);
    cmd.arg("build");
    if mode == Mode::Asserted {
        cmd.arg("--assert-proofs");
    }
    let r = run(cmd.arg(file).arg("-o").arg(&bin), dir, &[], false, build_limit());
    if r.timed_out {
        return Build::Timeout;
    }
    if r.code == Some(0) && bin.exists() {
        return Build::Built(bin);
    }
    let text = r.stderr.clone();
    // "(not yet known)" is kayo naming a type it could not work out: a
    // refusal, not a missing feature (2026-10-16)
    if NOT_YET.iter().any(|m| text.replace("(not yet known)", "").contains(m)) {
        let first = text.lines().find(|l| NOT_YET.iter().any(|m| l.contains(m))).unwrap_or("").to_string();
        return Build::NotYet { msg: first };
    }
    let msg = text.lines().find(|l| l.starts_with("error") || l.starts_with("check")).unwrap_or(text.lines().next().unwrap_or("")).to_string();
    let line = text.lines().find(|l| l.starts_with("where:")).and_then(line_of);
    Build::Refused { line, msg }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Exec {
    Ran { out: Vec<u8> },
    /// Exited non-zero: a runtime check fired (exit 101 with `check:` and
    /// `where: …, line N`), or something else went wrong.
    Stopped { out: Vec<u8>, code: Option<i32>, line: Option<u32>, msg: String },
    Timeout,
}

/// Threads a CCA binary may use for its parallel `for` (CCA 34d9c18), set
/// by the harness so that jobs × threads stays within the three-core share.
pub static THREADS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);

pub fn cca_run(bin: &Path, dir: &Path, env: &[(String, String)]) -> Exec {
    let mut env = env.to_vec();
    env.push(("CYBORG_THREADS".into(), THREADS.load(Ordering::Relaxed).to_string()));
    let r = run(&mut Command::new(bin), dir, &env, true, RUN_LIMIT);
    if r.timed_out {
        return Exec::Timeout;
    }
    if r.code == Some(0) {
        return Exec::Ran { out: r.stdout };
    }
    let line = r.stderr.lines().find(|l| l.starts_with("where:")).and_then(line_of);
    let msg = r.stderr.lines().next().unwrap_or("").to_string();
    Exec::Stopped { out: r.stdout, code: r.code, line, msg }
}

/// `kayo prove --json <file>`: Precog, the compiler's prover (Precog, then Cortex,
/// then nameless 2026-10-14, Precog again 2026-10-16; `prove`, owner 2026-10-15).
pub fn analyze(t: &Tools, file: &Path, dir: &Path, limit: Duration) -> Run {
    run(Command::new(&t.cca).arg("prove").arg("--json").arg(file), dir, &[], false, limit)
}
