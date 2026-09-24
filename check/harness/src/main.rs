//! `check` — check claude's differential harness (design/verification-session.md).
//!
//!   check gen    --seed N [--size S] [--inputs K]         print the program for a seed
//!   check one    --seed N [--size S] [--inputs K]         run one seed through both engines, verbose
//!   check run    --cases N [--from S] [--size S] [--inputs K] [--jobs J] [--keep-going] [--no-shrink]
//!   check shrink --seed N [--size S] [--inputs K]         shrink one seed's finding
//!   check file   <prog.cpls>                              compare one hand-written program
//!   check corpus                                          re-run every past finding in check/corpus/

mod claims;
mod compare;
mod engines;
mod json;

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
            if matches!(f, "keep-going" | "no-shrink" | "pow" | "edge" | "exhaust" | "target") {
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
        let avoid = self.flags.get("avoid").cloned().unwrap_or("exact-bits".into());
        check_gen::ast::NARROW_64.store(avoid.contains("u64-high"), std::sync::atomic::Ordering::Relaxed);
        check_gen::ast::PRINT_THROUGH_TO.store(avoid.contains("mixed-width"), std::sync::atomic::Ordering::Relaxed);
        Config {
            size: self.num("size", 3) as u32,
            inputs: self.num("inputs", if self.has("exhaust") { 2 } else { 0 }) as u32,
            risk: self.num("risk", 5),
            pow: !avoid.contains("pow"),
            range_edge: !avoid.contains("range-edge"),
            width_from_context: !avoid.contains("shl-width"),
            ranges: !avoid.contains("ranges"),
            edge: self.has("edge"),
            sized_bits: avoid.contains("exact-bits"),
            recursion: !avoid.contains("recursion"),
            exhaust: if self.has("exhaust") { self.num("domain", EXHAUST_CAP).max(2) } else { 0 },
        }
    }
}

fn die(m: &str) -> ! {
    eprintln!("check: {m}");
    std::process::exit(2)
}

/// Attack (a): at most this many input combinations per program
/// (provisional, Kespar's number; `--domain N` changes it).
const EXHAUST_CAP: u64 = 4096;

/// Input sets for a program: environment-variable lengths. In exhaustive
/// mode every combination of every input's domain; otherwise chosen from
/// the seed — edges first, then random (provisional; see README).
fn envs(p: &Program) -> Vec<Env> {
    if p.inputs.is_empty() {
        return vec![];
    }
    if p.inputs.iter().all(|i| i.domain.is_some()) {
        let mut sets: Vec<Env> = vec![vec![]];
        for i in &p.inputs {
            let m = i.domain.unwrap() as usize;
            sets = sets.into_iter().flat_map(|e| (0..m).map(move |n| {
                let mut e = e.clone();
                e.push((i.var.clone(), "a".repeat(n)));
                e
            })).collect();
        }
        return sets;
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

/// Input sets aimed by Z3 at the program's sites (attack (c)): every
/// distinct failing input it found, at most `cap`, after the usual ones.
fn aimed_envs(p: &Program, dir: &Path, cap: usize) -> (Vec<Env>, usize, usize) {
    let mut sets = envs(p);
    let Some((sites, answers)) = engines::aim(p, dir) else { return (sets, 0, 0) };
    let found = answers.iter().filter(|a| matches!(a, check_gen::solve::Answer::Fails(_))).count();
    let mut seen = std::collections::BTreeSet::new();
    for a in answers.into_iter().filter_map(|a| if let check_gen::solve::Answer::Fails(v) = a { Some(v) } else { None }) {
        if a.len() != p.inputs.len() || !seen.insert(a.clone()) {
            continue;
        }
        if seen.len() > cap {
            break;
        }
        sets.push(p.inputs.iter().zip(&a).map(|(i, n)| (i.var.clone(), "a".repeat(*n as usize))).collect());
    }
    (sets, sites.len(), found)
}

fn main() {
    engines::reap_leftovers();
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
                    let small = shrink(&t, p, &f.sig, f.mode, &f.env, &dir, a.num("jobs", default_jobs()).min(JOBS_CAP) as usize);
                    print!("{}{}", env_header(&f.env), small.render());
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
        "aim" => {
            let p = generate(a.num("seed", 1), &a.config());
            match engines::aim(&p, &work().join("aim")) {
                None => println!("too big to translate, or Z3 gave no answer"),
                Some((sites, answers)) => {
                    use check_gen::solve::Answer;
                    for (k, s) in sites.iter().enumerate() {
                        match answers.get(k) {
                            Some(Answer::Fails(v)) => println!("FAILS    {:?}  {}", v, s.what),
                            Some(Answer::Never) => println!("never    {}", s.what),
                            Some(Answer::Unknown) => println!("unknown  {}", s.what),
                            None => println!("unasked  {}", s.what),
                        }
                    }
                    println!("{} sites; script at {}", sites.len(), work().join("aim/aim.smt2").display());
                }
            }
        }
        "claims" => {
            let t = tools();
            let p = generate(a.num("seed", 1), &a.config());
            let dir = work().join("claims");
            match claims::check_claims(&t, &p, &dir) {
                Err(e) => println!("seed {}: {e}", p.seed),
                Ok(o) => {
                    println!(
                        "seed {}: {} proven claims: {} with nothing to ask (literals, the printer), {} matched to my tree, {} not; {} justified by the simple intervals (b){}; {} solver questions (c), {} given up, {} not reached; {} solver answers the exact run did not confirm",
                        p.seed,
                        o.proven,
                        o.literal,
                        o.matched,
                        o.unmatched.len(),
                        o.justified,
                        if o.intervals_complete { "" } else { " — (b) could not follow the whole program" },
                        o.asked,
                        o.unknown,
                        o.unasked,
                        o.unconfirmed
                    );
                    for c in o.unmatched.iter().take(5) {
                        println!("  unmatched: line {}:{}..{} {} {}", c.line, c.col, c.end_col, c.kind, c.construct);
                    }
                    for r in &o.refuted {
                        let c = &r.claim;
                        println!(
                            "REFUTED  line {}:{}..{} {} `{}` (layer {}, in {}): {} — inputs {:?}  [{}]",
                            c.line, c.col, c.end_col, c.kind, c.construct, c.layer, c.function, r.test, r.inputs, r.site
                        );
                        if r.inputs.iter().all(|v| *v <= check_gen::solve::INPUT_MAX) {
                            let env: Env = p.inputs.iter().zip(&r.inputs).map(|(i, n)| (i.var.clone(), "a".repeat(*n as usize))).collect();
                            let v = check(&t, &p.render(), &dir.join("run"), &[env]);
                            print!("    run on that input: ");
                            report(&v);
                        } else {
                            println!("    that input is too long to set as an environment variable; the claim stands refuted by the solver only");
                        }
                    }
                }
            }
        }
        "claims-run" => claims_run(&tools(), &a),
        "evalsite" => {
            // What the exact evaluator saw at every site on one line.
            let p = generate(a.num("seed", 1), &a.config());
            let (_, spans) = p.render_spans();
            let vals: Vec<u64> = a.flags.get("values").map(|v| v.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or_default();
            let r = check_gen::eval::run(&p, &vals);
            let line = a.num("line", 0) as u32;
            for ((n, st, kind), seen) in &r.seen {
                let at = spans.expr.get(n).filter(|x| x.0 == line).map(|x| (x.1, x.2)).or(spans.stmt.get(st).filter(|x| x.0 == line).map(|x| (x.1, x.3)));
                if let Some((c0, c1)) = at {
                    println!("{line}:{c0}..{c1} {kind}: {:?}", seen);
                }
            }
            println!("stopped at: {:?}; exhausted: {}", r.stopped.map(|x| x.2), r.exhausted);
        }
        "evaltest" => {
            // check_gen::eval against the oracle: stops or not, and where.
            let t = tools();
            let (mut same, mut differ) = (0, 0);
            for seed in a.num("from", 1)..a.num("from", 1) + a.num("cases", 20) {
                let p = generate(seed, &a.config());
                let (text, spans) = p.render_spans();
                let dir = work().join("evaltest");
                std::fs::create_dir_all(&dir).unwrap();
                let f = dir.join("prog.cpls");
                std::fs::write(&f, &text).unwrap();
                let sets = if p.inputs.is_empty() { vec![vec![]] } else { envs(&p) };
                for env in sets {
                    let vals: Vec<u64> = env.iter().map(|(_, v)| v.len() as u64).collect();
                    let o = engines::oracle(&t, &f, &dir, &env);
                    let r = check_gen::eval::run(&p, &vals);
                    let eline = r.stopped.and_then(|(n, s, _)| spans.expr.get(&n).map(|x| x.0).or(spans.stmt.get(&s).map(|x| x.0)));
                    let oline = match &o {
                        engines::Oracle::Stopped { line, .. } => Some(line.unwrap_or(0)),
                        _ => None,
                    };
                    if r.exhausted {
                        continue;
                    }
                    if eline == oline {
                        same += 1;
                    } else {
                        differ += 1;
                        println!("seed {seed} inputs {vals:?}: oracle stops at {oline:?}, eval at {eline:?} ({:?})", r.stopped.map(|x| x.2));
                    }
                }
            }
            println!("{same} runs agree, {differ} differ");
        }
        "corpus" => corpus(&tools()),
        "run" => run(&tools(), &a),
        _ => {
            eprintln!("usage: check gen|one|run|shrink|file|corpus  (see check/README.md)");
            std::process::exit(2)
        }
    }
}

/// `# env NAME LENGTH` lines, the corpus's way of recording an input set.
fn env_header(env: &Env) -> String {
    env.iter().map(|(k, v)| format!("# env {k} {}\n", v.len())).collect()
}

/// This session's share of the machine: 3 cores (owner, 2026-09-23 — ten
/// cores split three ways between the heavy sessions, one for the desktop).
/// Every `cca build` and oracle run counts; `--jobs` may lower it.
const JOBS_CAP: u64 = 3;

fn default_jobs() -> u64 {
    JOBS_CAP
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
/// A step is kept only if it shows the same finding: the same kind and the
/// same messages from both engines (numbers aside), not merely the same kind.
fn shrink(t: &Tools, p: Program, sig: &str, mode: engines::Mode, env: &Env, dir: &Path, jobs: usize) -> Program {
    let kind = sig.split(':').next().unwrap_or("");
    engines::set_shrinking(true);
    let modes = [mode];
    check_gen::shrink::STRUCTURAL_ONLY.store(sig.contains("cca build: check:"), std::sync::atomic::Ordering::Relaxed);
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
                        let modes = &modes;
                        big_stack().spawn_scoped(s, move || match compare::check_modes(t, &c.render(), &d, envs, modes) {
                            Verdict::Finding(f) if f.sig == sig => Some(f.detail),
                            _ => None,
                        })
                    })
                    .collect();
                hs.into_iter().map(|h| h.expect("spawn").join().unwrap()).collect()
            });
            // Earliest success first; each is confirmed alone before it is kept.
            for (k, detail) in results.iter().enumerate().filter_map(|(k, r)| r.as_ref().map(|d| (k, d))) {
                let d = dir.join("confirm");
                let again = compare::check_modes(t, &batch[k].render(), &d, &envs, &modes);
                if matches!(&again, Verdict::Finding(f) if f.sig == sig) {
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
            None => {
                engines::set_shrinking(false);
                return cur;
            }
        }
    }
}

/// Worker threads get a big stack: generating, rendering and dropping a deep
/// program recurses, and the 2 MiB default once overflowed and took the
/// whole harness down (seeds 301–600, 2026-09-23).
fn big_stack() -> std::thread::Builder {
    std::thread::Builder::new().stack_size(64 << 20)
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
    let jobs = a.num("jobs", default_jobs()).min(JOBS_CAP);
    let keep_going = a.has("keep-going");
    let target = a.has("target");
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
            big_stack().spawn_scoped(s, move || {
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
                    let sets = if target { aimed_envs(&p, &dir, 32).0 } else { envs(&p) };
                    let v = check(t, &p.render(), &dir, &sets);
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
            })
            .expect("spawn");
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
            let small = shrink(t, p, &f.sig, f.mode, &f.env, &work().join("shrink"), jobs as usize);
            let fd = work().join("findings").join(seed.to_string());
            std::fs::write(fd.join("shrunk.cpls"), format!("{}{}", env_header(&f.env), small.render())).unwrap();
            let v = check(t, &small.render(), &fd.join("shrunk"), &if f.env.is_empty() { vec![] } else { vec![f.env.clone()] });
            println!("\n=== {k}, seed {seed}, shrunk to {}:", fd.join("shrunk.cpls").display());
            report(&v);
        }
    }
}

/// The Precog oracle's claim check over many programs: every proven claim,
/// (b) first, then (c); every refuted claim printed as it is found.
fn claims_run(t: &Tools, a: &Args) {
    let cfg = a.config();
    let cases = a.num("cases", 20);
    let from = a.num("from", 1);
    let jobs = a.num("jobs", default_jobs()).min(JOBS_CAP);
    let next = AtomicU64::new(from);
    #[derive(Default)]
    struct T {
        programs: u64,
        failed: u64,
        proven: usize,
        nothing: usize,
        matched: usize,
        unmatched: usize,
        justified: usize,
        asked: usize,
        unknown: usize,
        unasked: usize,
        refuted: usize,
        unconfirmed: usize,
        incomplete_b: u64,
    }
    let tally = Mutex::new(T::default());
    std::thread::scope(|s| {
        for w in 0..jobs {
            let (next, tally, cfg) = (&next, &tally, &cfg);
            big_stack()
                .spawn_scoped(s, move || {
                    let dir = work().join(format!("cw{w}"));
                    loop {
                        let seed = next.fetch_add(1, Ordering::SeqCst);
                        if seed >= from + cases {
                            return;
                        }
                        let p = generate(seed, cfg);
                        let r = claims::check_claims(t, &p, &dir);
                        let mut tl = tally.lock().unwrap();
                        tl.programs += 1;
                        let Ok(o) = r else {
                            tl.failed += 1;
                            continue;
                        };
                        tl.proven += o.proven;
                        tl.nothing += o.literal;
                        tl.matched += o.matched;
                        tl.unmatched += o.unmatched.len();
                        tl.justified += o.justified;
                        tl.asked += o.asked;
                        tl.unknown += o.unknown;
                        tl.unasked += o.unasked;
                        tl.refuted += o.refuted.len();
                        tl.unconfirmed += o.unconfirmed;
                        tl.incomplete_b += (!o.intervals_complete) as u64;
                        drop(tl);
                        for r in &o.refuted {
                            let c = &r.claim;
                            let mut line = format!(
                                "seed {seed}: REFUTED line {}:{}..{} {} `{}` (layer {}, in {}): {} — inputs {:?} [{}]",
                                c.line, c.col, c.end_col, c.kind, c.construct, c.layer, c.function, r.test, r.inputs, r.site
                            );
                            if r.inputs.iter().all(|v| *v <= check_gen::solve::INPUT_MAX) {
                                let env: Env = p.inputs.iter().zip(&r.inputs).map(|(i, n)| (i.var.clone(), "a".repeat(*n as usize))).collect();
                                let v = check(t, &p.render(), &dir.join("run"), &[env]);
                                line += &format!("\n    run on that input: {}", match v {
                                    Verdict::Finding(f) => format!("FINDING ({})\n{}", f.kind, f.detail),
                                    Verdict::Agree { .. } => "both engines agree".into(),
                                    Verdict::NotYet(m) => format!("not yet: {m}"),
                                    Verdict::SlowBuild => "build too slow".into(),
                                });
                            } else {
                                line += "\n    input too long to set; refuted by the solver only";
                            }
                            println!("{line}");
                        }
                    }
                })
                .expect("spawn");
        }
    });
    let tl = tally.into_inner().unwrap();
    println!(
        "\n{} programs ({} could not be checked); {} proven claims: {} with nothing to ask, {} matched, {} not matched; (b) justified {} ({} programs where (b) could not follow everything); (c) asked {} questions: {} answered, {} given up by Z3, {} not reached in time; REFUTED {} (confirmed by running the program exactly; {} solver answers not confirmed)",
        tl.programs, tl.failed, tl.proven, tl.nothing, tl.matched, tl.unmatched, tl.justified, tl.incomplete_b, tl.asked, tl.asked - tl.unknown - tl.unasked, tl.unknown, tl.unasked, tl.refuted, tl.unconfirmed
    );
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
