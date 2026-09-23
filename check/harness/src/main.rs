//! `check` — check claude's differential harness (design/verification-session.md).
//!
//!   check gen    --seed N [--size S] [--inputs K]         print the program for a seed
//!   check one    --seed N [--size S] [--inputs K]         run one seed through both engines, verbose
//!   check run    --cases N [--from S] [--size S] [--inputs K] [--jobs J] [--keep-going] [--no-shrink]
//!   check shrink --seed N [--size S] [--inputs K]         shrink one seed's finding
//!   check file   <prog.cpls>                              compare one hand-written program
//!   check corpus                                          re-run every past finding in check/corpus/

mod compare;
mod engines;

use check_gen::rng::Rng;
use check_gen::shrink::{candidates, well_scoped};
use check_gen::{generate, Config, Program};
use compare::{check, Env, Verdict};
use engines::Tools;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn work() -> PathBuf {
    repo().join("check/work")
}

struct Args {
    cmd: String,
    rest: Vec<String>,
    flags: BTreeMap<String, String>,
}

fn parse() -> Args {
    let mut it = std::env::args().skip(1);
    let cmd = it.next().unwrap_or_default();
    let mut flags = BTreeMap::new();
    let mut rest = vec![];
    let v: Vec<String> = it.collect();
    let mut i = 0;
    while i < v.len() {
        if let Some(f) = v[i].strip_prefix("--") {
            if matches!(f, "keep-going" | "no-shrink" | "pow") {
                flags.insert(f.to_string(), "1".into());
            } else {
                flags.insert(f.to_string(), v.get(i + 1).cloned().unwrap_or_default());
                i += 1;
            }
        } else {
            rest.push(v[i].clone());
        }
        i += 1;
    }
    Args { cmd, rest, flags }
}

impl Args {
    fn num(&self, k: &str, d: u64) -> u64 {
        self.flags.get(k).map(|v| v.parse().unwrap_or_else(|_| die(&format!("--{k} wants a number")))).unwrap_or(d)
    }
    fn has(&self, k: &str) -> bool {
        self.flags.contains_key(k)
    }
    fn config(&self) -> Config {
        // Known CCA bugs the generator steers around until they are fixed
        // (`--avoid none` to aim straight at them).
        let avoid = self.flags.get("avoid").cloned().unwrap_or(String::new());
        check_gen::ast::NARROW_64.store(avoid.contains("u64-high"), std::sync::atomic::Ordering::Relaxed);
        Config {
            size: self.num("size", 3) as u32,
            inputs: self.num("inputs", 0) as u32,
            risk: self.num("risk", 5),
            pow: self.has("pow"),
            range_edge: !avoid.contains("range-edge"),
            width_from_context: !avoid.contains("shl-width"),
        }
    }
}

fn die(m: &str) -> ! {
    eprintln!("check: {m}");
    std::process::exit(2)
}

/// Input sets for a program: environment-variable lengths, chosen from the
/// seed — edges first, then random (provisional; see README).
fn envs(p: &Program) -> Vec<Env> {
    if p.inputs.is_empty() {
        return vec![];
    }
    const EDGES: [usize; 14] = [0, 1, 2, 3, 7, 10, 127, 128, 255, 256, 1000, 32767, 32768, 65536];
    let mut rng = Rng::new(p.seed ^ 0x1D5E_7C0D_E5E7_0000);
    let mut sets = vec![];
    for k in 0..4 {
        let env: Env = p
            .inputs
            .iter()
            .map(|i| {
                let n = if k == 0 { 0 } else if rng.chance(70) { *rng.pick(&EDGES) } else { rng.below(70_000) as usize };
                (i.var.clone(), "a".repeat(n))
            })
            .collect();
        sets.push(env);
    }
    sets
}

fn main() {
    let a = parse();
    let tools = || Tools::find(&repo()).unwrap_or_else(|e| die(&e));
    match a.cmd.as_str() {
        "gen" => {
            let p = generate(a.num("seed", 1), &a.config());
            print!("{}", p.render());
        }
        "one" => {
            let t = tools();
            let p = generate(a.num("seed", 1), &a.config());
            let dir = work().join("one");
            let v = check(&t, &p.render(), &dir, &envs(&p));
            println!("program: {}", dir.join("prog.cpls").display());
            report(&v);
        }
        "shrink" => {
            let t = tools();
            let p = generate(a.num("seed", 1), &a.config());
            let dir = work().join("shrink");
            match check(&t, &p.render(), &dir.join("0"), &envs(&p)) {
                Verdict::Finding(f) => {
                    let small = shrink(&t, p, f.kind, &f.env, &dir, a.num("jobs", default_jobs()) as usize);
                    print!("{}", small.render());
                }
                v => {
                    println!("seed {} has no finding", a.num("seed", 1));
                    report(&v);
                }
            }
        }
        "file" => {
            let t = tools();
            let path = a.rest.first().unwrap_or_else(|| die("file wants a .cpls path"));
            let (src, env) = read_case(Path::new(path));
            let v = check(&t, &src, &work().join("file"), &env);
            report(&v);
        }
        "corpus" => corpus(&tools()),
        "run" => run(&tools(), &a),
        _ => {
            eprintln!("usage: check gen|one|run|shrink|file|corpus  (see check/README.md)");
            std::process::exit(2)
        }
    }
}

fn default_jobs() -> u64 {
    std::thread::available_parallelism().map(|n| n.get() as u64).unwrap_or(4)
}

fn report(v: &Verdict) {
    match v {
        Verdict::Agree { oracle_stopped: None } => println!("agree: both ran and printed the same"),
        Verdict::Agree { oracle_stopped: Some(m) } => println!("agree: both stopped at the same site ({m})"),
        Verdict::NotYet(m) => println!("not yet in CCA: {m}"),
        Verdict::SlowBuild => println!("cca build ran past its limit ({} s)", engines::BUILD_LIMIT.as_secs()),
        Verdict::Finding(f) => println!("FINDING ({})\n{}", f.kind, f.detail),
    }
}

/// A corpus file is a program; `# env NAME LENGTH` lines at the top give an
/// input set (the variable is set to that many characters).
fn read_case(path: &Path) -> (String, Vec<Env>) {
    let src = std::fs::read_to_string(path).unwrap_or_else(|e| die(&format!("{}: {e}", path.display())));
    let mut env = Env::new();
    for l in src.lines() {
        if let Some(rest) = l.strip_prefix("# env ") {
            let mut w = rest.split_whitespace();
            if let (Some(k), Some(n)) = (w.next(), w.next().and_then(|n| n.parse::<usize>().ok())) {
                env.push((k.to_string(), "a".repeat(n)));
            }
        }
    }
    let envs = if env.is_empty() { vec![] } else { vec![env] };
    (src, envs)
}

fn corpus(t: &Tools) {
    let dir = repo().join("check/corpus");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map_or(false, |x| x == "cpls")).collect();
    files.sort();
    let (mut ok, mut bad) = (0, 0);
    for f in &files {
        let (src, env) = read_case(f);
        let name = f.file_stem().unwrap().to_string_lossy().to_string();
        let v = check(t, &src, &work().join("corpus").join(&name), &env);
        match &v {
            Verdict::Finding(x) => {
                bad += 1;
                println!("STILL FAILS  {name}  ({})", x.kind);
            }
            Verdict::NotYet(m) => println!("not yet      {name}  ({m})"),
            Verdict::SlowBuild => println!("slow build   {name}"),
            Verdict::Agree { .. } => {
                ok += 1;
                println!("agrees       {name}");
            }
        }
    }
    println!("{} corpus programs: {ok} agree, {bad} still fail", files.len());
}

/// Greedy: take the first smaller candidate that still shows the same kind of
/// finding under the same inputs; repeat until none does. A candidate must
/// first be well scoped and make the oracle behave as it did in the finding
/// (ran, or stopped) — both cheap — before CCA builds it. Survivors are tried
/// `jobs` at a time; the earliest success wins, so the result does not depend
/// on which thread finished first.
fn shrink(t: &Tools, p: Program, kind: &str, env: &Env, dir: &Path, jobs: usize) -> Program {
    let envs: Vec<Env> = if env.is_empty() { vec![] } else { vec![env.clone()] };
    let oracle_class = |src: &str, d: &Path| -> bool {
        std::fs::create_dir_all(d).unwrap();
        let f = d.join("prog.cpls");
        std::fs::write(&f, src).unwrap();
        matches!(engines::oracle(t, &f, d, env), engines::Oracle::Ran { .. })
    };
    let want_ran = oracle_class(&p.render(), &dir.join("o"));
    let mut cur = p;
    let mut cur_len = cur.render().len();
    loop {
        let cands: Vec<Program> = candidates(&cur).into_iter().filter(|c| c.render().len() < cur_len && well_scoped(c)).collect();
        let mut next = None;
        let mut i = 0;
        while i < cands.len() && next.is_none() {
            let mut batch = vec![];
            while i < cands.len() && batch.len() < jobs.max(1) {
                if oracle_class(&cands[i].render(), &dir.join("o")) == want_ran {
                    batch.push(cands[i].clone());
                }
                i += 1;
            }
            let results: Vec<Option<String>> = std::thread::scope(|s| {
                let hs: Vec<_> = batch
                    .iter()
                    .enumerate()
                    .map(|(k, c)| {
                        let d = dir.join(format!("w{k}"));
                        let envs = &envs;
                        s.spawn(move || match check(t, &c.render(), &d, envs) {
                            Verdict::Finding(f) if f.kind == kind => Some(f.detail),
                            _ => None,
                        })
                    })
                    .collect();
                hs.into_iter().map(|h| h.join().unwrap()).collect()
            });
            // Earliest success first; each is confirmed alone before it is kept.
            for (k, detail) in results.iter().enumerate().filter_map(|(k, r)| r.as_ref().map(|d| (k, d))) {
                let d = dir.join("confirm");
                let again = check(t, &batch[k].render(), &d, &envs);
                if matches!(&again, Verdict::Finding(f) if f.kind == kind) {
                    next = Some(batch[k].clone());
                    break;
                }
                let note = format!("in parallel:\n{detail}\nalone:\n{again:?}\n");
                eprintln!("shrink: a step showed `{kind}` in parallel but not alone; kept at {}", keep_flaky(&batch[k].render(), &note).display());
            }
        }
        match next {
            Some(n) => {
                cur_len = n.render().len();
                cur = n;
            }
            None => return cur,
        }
    }
}

/// A program whose verdict changed between two runs: kept for a look.
fn keep_flaky(src: &str, note: &str) -> PathBuf {
    let d = work().join("flaky");
    std::fs::create_dir_all(&d).unwrap();
    let n = std::fs::read_dir(&d).map(|r| r.count()).unwrap_or(0);
    let p = d.join(format!("{n}.cpls"));
    std::fs::write(&p, src).unwrap();
    std::fs::write(d.join(format!("{n}.txt")), note).unwrap();
    p
}

fn run(t: &Tools, a: &Args) {
    let cfg = a.config();
    let cases = a.num("cases", 100);
    let from = a.num("from", 1);
    let jobs = a.num("jobs", default_jobs());
    let keep_going = a.has("keep-going");
    let next = AtomicU64::new(from);
    let stop = AtomicBool::new(false);
    let done = AtomicU64::new(0);
    #[derive(Default)]
    struct Tally {
        agree: u64,
        agree_stopped: u64,
        slow: Vec<u64>,
        not_yet: BTreeMap<String, u64>,
        stops: BTreeMap<String, u64>,
        findings: Vec<(u64, compare::Finding)>,
    }
    let tally = Mutex::new(Tally::default());
    std::thread::scope(|s| {
        for w in 0..jobs {
            let (next, stop, done, tally, cfg) = (&next, &stop, &done, &tally, &cfg);
            s.spawn(move || {
                let dir = work().join(format!("w{w}"));
                loop {
                    if stop.load(Ordering::SeqCst) {
                        return;
                    }
                    let seed = next.fetch_add(1, Ordering::SeqCst);
                    if seed >= from + cases {
                        return;
                    }
                    let p = generate(seed, cfg);
                    let v = check(t, &p.render(), &dir, &envs(&p));
                    let mut tl = tally.lock().unwrap();
                    match v {
                        Verdict::Agree { oracle_stopped } => {
                            tl.agree += 1;
                            if let Some(m) = oracle_stopped {
                                tl.agree_stopped += 1;
                                *tl.stops.entry(normalise(&m)).or_default() += 1;
                            }
                        }
                        Verdict::NotYet(m) => *tl.not_yet.entry(normalise(&m)).or_default() += 1,
                        Verdict::SlowBuild => tl.slow.push(seed),
                        Verdict::Finding(f) => {
                            println!("seed {seed}: FINDING ({})", f.kind);
                            let fd = work().join("findings").join(seed.to_string());
                            std::fs::create_dir_all(&fd).unwrap();
                            std::fs::write(fd.join("prog.cpls"), p.render()).unwrap();
                            std::fs::write(fd.join("report.txt"), format!("seed {seed}, {:?}\n{}", cfg, f.detail)).unwrap();
                            tl.findings.push((seed, f));
                            if !keep_going {
                                stop.store(true, Ordering::SeqCst);
                            }
                        }
                    }
                    drop(tl);
                    let n = done.fetch_add(1, Ordering::SeqCst) + 1;
                    if n % 100 == 0 {
                        eprintln!("… {n} cases");
                    }
                }
            });
        }
    });
    let mut tl = tally.into_inner().unwrap();
    tl.findings.sort_by_key(|f| f.0);
    println!("\n{} cases (seeds from {from}), size {}, inputs {}", done.load(Ordering::SeqCst), cfg.size, cfg.inputs);
    println!("agree: {} ({} of them both stopping at the same site)", tl.agree, tl.agree_stopped);
    tl.slow.sort();
    println!("cca build past {} s: {} (seeds {:?})", engines::BUILD_LIMIT.as_secs(), tl.slow.len(), tl.slow.iter().take(10).collect::<Vec<_>>());
    let ny: u64 = tl.not_yet.values().sum();
    println!("not yet in CCA: {ny}");
    for (m, n) in top(&tl.not_yet, 8) {
        println!("    {n:6}  {m}");
    }
    if !tl.stops.is_empty() {
        println!("where agreeing runs stopped:");
        for (m, n) in top(&tl.stops, 8) {
            println!("    {n:6}  {m}");
        }
    }
    println!("findings: {}", tl.findings.len());
    let mut by_kind: BTreeMap<&str, Vec<u64>> = BTreeMap::new();
    let mut by_sig: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    for (seed, f) in &tl.findings {
        by_kind.entry(f.kind).or_default().push(*seed);
        by_sig.entry(f.sig.clone()).or_default().push(*seed);
    }
    let mut sigs: Vec<_> = by_sig.iter().collect();
    sigs.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(b.0)));
    for (sg, seeds) in sigs {
        println!("    {:4}  {sg}\n          seeds {:?}", seeds.len(), seeds.iter().take(10).collect::<Vec<_>>());
    }
    if !a.has("no-shrink") {
        for (k, seeds) in &by_kind {
            let seed = seeds[0];
            let (_, f) = tl.findings.iter().find(|x| x.0 == seed).unwrap();
            let p = generate(seed, &cfg);
            let small = shrink(t, p, k, &f.env, &work().join("shrink"), jobs as usize);
            let fd = work().join("findings").join(seed.to_string());
            std::fs::write(fd.join("shrunk.cpls"), small.render()).unwrap();
            let v = check(t, &small.render(), &fd.join("shrunk"), &if f.env.is_empty() { vec![] } else { vec![f.env.clone()] });
            println!("\n=== {k}, seed {seed}, shrunk to {}:", fd.join("shrunk.cpls").display());
            report(&v);
        }
    }
}

fn top(m: &BTreeMap<String, u64>, n: usize) -> Vec<(String, u64)> {
    let mut v: Vec<(String, u64)> = m.iter().map(|(k, v)| (k.clone(), *v)).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    v.truncate(n);
    v
}

/// A message with its numbers replaced, so messages can be counted by kind.
pub fn normalise(m: &str) -> String {
    let mut out = String::new();
    let mut in_num = false;
    for c in m.chars() {
        if c.is_ascii_digit() {
            if !in_num {
                out.push('N');
            }
            in_num = true;
        } else {
            in_num = false;
            out.push(c);
        }
    }
    out.chars().take(110).collect()
}
