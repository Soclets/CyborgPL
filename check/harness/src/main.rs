//! `check` — check claude's differential harness (design/verification-session.md).
//!
//!   check gen    --seed N [--size S] [--inputs K]         print the program for a seed
//!   check one    --seed N [--size S] [--inputs K]         run one seed through both engines, verbose
//!   check run    --cases N [--from S] [--size S] [--inputs K] [--jobs J] [--keep-going] [--no-shrink]
//!   check shrink --seed N [--size S] [--inputs K]         shrink one seed's finding
//!   check file   <prog.zg>                               compare one hand-written program
//!   check corpus                                          re-run every past finding in check/corpus/

mod claims;
mod compare;
mod engines;
mod json;
mod tv;

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
    /// For the evaluator and the solver: no deci or text, which they do not
    /// model (a program with them would be skipped whole).
    fn config_modelled(&self) -> Config {
        Config { deci: false, brain16: false, mutator: false, ..self.config() }
    }

    fn config(&self) -> Config {
        // Known CCA bugs the generator steers around until they are fixed
        // (`--avoid none` to aim straight at them).
        let avoid = self.flags.get("avoid").cloned().unwrap_or("std".into());
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
            lists: !avoid.contains("lists"),
            item_through_to: avoid.contains("item-width"),
            data_through_to: avoid.contains("data-width"),
            borrows: !avoid.contains("borrows"),
            borrow_wrong: self.num("borrow-wrong", 250),
            enums: !avoid.contains("enums"),
            strict: if avoid.contains("strict") { 0 } else { self.num("strict", 200) },
            simd: !avoid.contains("simd"),
            simd_lanes: !avoid.contains("simd-lanes"),
            simd_wrap: !avoid.contains("simd-wrap"),
            deci: !avoid.split(',').any(|a| a == "deci"),
            deci_useful: !avoid.contains("deci-useful"),
            no_cheat: !avoid.contains("no-cheat"),
            brain16: !avoid.contains("brain16"),
            brain_mod: !avoid.contains("brain-mod"),
            roundings: !avoid.contains("rounding"),
            mutator: !avoid.contains("mutator"),
            ieee: !avoid.contains("ieee"),
            remove: !avoid.contains("remove"),
            shadow: !avoid.contains("shadow"),
            shadow_wrong: self.num("shadow-wrong", 50),
            std_lib: !avoid.contains("std") && !avoid.contains("cord"),
            iterators: !avoid.contains("iterators"),
            print: !avoid.contains("print"),
            written: !avoid.contains("written"),
            consts: !avoid.contains("consts"),
            new_words: !avoid.contains("new-words"),
            edge_landings: !avoid.contains("edge-landings"),
        }
    }
}

/// A CCA binary's threads: `--threads`, but never more than the share left
/// by the jobs running beside it (3 jobs: 1 thread; `--jobs 1`: up to 3).
fn set_threads(a: &Args, jobs: u64) {
    let room = (JOBS_CAP / jobs.max(1)).max(1);
    let t = a.num("threads", room).min(room).max(1);
    engines::THREADS.store(t as usize, std::sync::atomic::Ordering::Relaxed);
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
            let v = compare::check_program(&t, &p, &dir, &envs(&p));
            println!("program: {}", dir.join("prog.zg").display());
            report(&v);
        }
        "shrink" => {
            let t = tools();
            let p = generate(a.num("seed", 1), &a.config());
            let dir = work().join("shrink");
            match compare::check_program(&t, &p, &dir.join("0"), &envs(&p)) {
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
            let path = a.rest.first().unwrap_or_else(|| die("file wants a .zg path"));
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
            let p = generate(a.num("seed", 1), &a.config_modelled());
            let dir = work().join("claims");
            match claims::check_claims(&t, &p, &dir) {
                Err(e) => println!("seed {}: {e}", p.seed),
                Ok(o) => {
                    println!(
                        "seed {}: {} proven claims: {} with nothing to ask (literals, the printer), {} matched to my tree ({} ambiguous), {} not; {} justified by the simple intervals (b){}; {} solver questions (c), {} given up, {} not reached; {} solver answers the exact run did not confirm",
                        p.seed,
                        o.proven,
                        o.literal,
                        o.matched,
                        o.ambiguous,
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
        "tv" => tv_run(&tools(), &a),
        "same" => same_run(&tools(), &a),
        "evalsite" => {
            // What the exact evaluator saw at every site on one line.
            let p = generate(a.num("seed", 1), &a.config_modelled());
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
                let p = generate(seed, &a.config_modelled());
                if p.expect_refused || p.unevaluable {
                    continue;
                }
                let (text, spans) = p.render_spans();
                let dir = work().join("evaltest");
                std::fs::create_dir_all(&dir).unwrap();
                let f = dir.join("prog.zg");
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
                        let msg = match &o {
                            engines::Oracle::Stopped { msg, .. } => msg.lines().next().unwrap_or("").to_string(),
                            _ => String::new(),
                        };
                        println!("seed {seed} inputs {vals:?}: oracle stops at {oline:?}, eval at {eline:?} ({:?}) {msg}", r.stopped.map(|x| x.2));
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
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().map_or(false, |x| x == "zg")).collect();
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
/// How long one shrink may take (provisional, 2026-09-29).
const SHRINK_LIMIT: std::time::Duration = std::time::Duration::from_secs(15 * 60);

fn shrink(t: &Tools, p: Program, sig: &str, mode: engines::Mode, env: &Env, dir: &Path, jobs: usize) -> Program {
    let kind = sig.split(':').next().unwrap_or("");
    engines::set_shrinking(true);
    let modes = [mode];
    // A refused `check`, or anything in a strict file, leans on the
    // generator's guards: simplifying an expression can remove one and make
    // the refusal right (seed 2875: the `and.then` guard went).
    check_gen::shrink::STRUCTURAL_ONLY.store(p.strict || sig.contains("cca build: check:"), std::sync::atomic::Ordering::Relaxed);
    let envs: Vec<Env> = if env.is_empty() { vec![] } else { vec![env.clone()] };
    let oracle_class = |src: &str, d: &Path| -> bool {
        std::fs::create_dir_all(d).unwrap();
        let f = d.join("prog.zg");
        std::fs::write(&f, src).unwrap();
        matches!(engines::oracle(t, &f, d, env), engines::Oracle::Ran { .. })
    };
    let want_ran = oracle_class(&p.render(), &dir.join("o"));
    let mut cur = p;
    let mut cur_len = cur.render().len();
    // A time limit, so a program that makes an engine crash slowly cannot
    // keep a core busy for hours (seed 6241, 2026-09-29: 79 minutes).
    let deadline = std::time::Instant::now() + SHRINK_LIMIT;
    loop {
        if std::time::Instant::now() > deadline {
            eprintln!("shrink: stopped at the {} min limit; kept the smallest so far", SHRINK_LIMIT.as_secs() / 60);
            engines::set_shrinking(false);
            return cur;
        }
        let cands: Vec<Program> = candidates(&cur).into_iter().filter(|c| c.render().len() < cur_len && well_scoped(c)).collect();
        let mut next = None;
        let mut i = 0;
        while i < cands.len() && next.is_none() && std::time::Instant::now() < deadline {
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
    let p = d.join(format!("{n}.zg"));
    std::fs::write(&p, src).unwrap();
    std::fs::write(d.join(format!("{n}.txt")), note).unwrap();
    p
}

fn run(t: &Tools, a: &Args) {
    let cfg = a.config();
    let cases = a.num("cases", 100);
    let from = a.num("from", 1);
    let jobs = a.num("jobs", default_jobs()).min(JOBS_CAP);
    set_threads(a, jobs);
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
                    let v = compare::check_program(t, &p, &dir, &sets);
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
                        Verdict::SlowBuild => {
                            // Kept for compiler claude: compile time is not a
                            // finding, but a slow build is worth a look.
                            let sd = work().join("slow");
                            std::fs::create_dir_all(&sd).unwrap();
                            std::fs::write(sd.join(format!("{seed}.zg")), p.render()).unwrap();
                            tl.slow.push(seed)
                        }
                        Verdict::Finding(f) => {
                            println!("seed {seed}: FINDING ({})", f.kind);
                            let fd = work().join("findings").join(seed.to_string());
                            std::fs::create_dir_all(&fd).unwrap();
                            std::fs::write(fd.join("prog.zg"), p.render()).unwrap();
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
            if k.contains("borrow") {
                // Found against the program's own expectation, which a
                // shrinking step would not keep up to date.
                continue;
            }
            let seed = seeds[0];
            let (_, f) = tl.findings.iter().find(|x| x.0 == seed).unwrap();
            let p = generate(seed, &cfg);
            let small = shrink(t, p, &f.sig, f.mode, &f.env, &work().join("shrink"), jobs as usize);
            let fd = work().join("findings").join(seed.to_string());
            std::fs::write(fd.join("shrunk.zg"), format!("{}{}", env_header(&f.env), small.render())).unwrap();
            let v = check(t, &small.render(), &fd.join("shrunk"), &if f.env.is_empty() { vec![] } else { vec![f.env.clone()] });
            println!("\n=== {k}, seed {seed}, shrunk to {}:", fd.join("shrunk.zg").display());
            report(&v);
        }
    }
}

/// The Precog oracle's claim check over many programs: every proven claim,
/// (b) first, then (c); every refuted claim printed as it is found.
fn claims_run(t: &Tools, a: &Args) {
    let cfg = a.config_modelled();
    let cases = a.num("cases", 20);
    let from = a.num("from", 1);
    let jobs = a.num("jobs", default_jobs()).min(JOBS_CAP);
    let next = AtomicU64::new(from);
    #[derive(Default)]
    struct T {
        programs: u64,
        failed: u64,
        /// Why programs could not be checked, and how many each.
        why: BTreeMap<String, u64>,
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
        ambiguous: usize,
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
                        let o = match r {
                            Ok(o) => o,
                            Err(e) => {
                                tl.failed += 1;
                                *tl.why.entry(e).or_default() += 1;
                                continue;
                            }
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
                        tl.ambiguous += o.ambiguous;
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
        "\n{} programs ({} could not be checked); {} proven claims: {} with nothing to ask, {} matched ({} of them ambiguous, not asked), {} not matched; (b) justified {} ({} programs where (b) could not follow everything); (c) asked {} questions: {} answered, {} given up by Z3, {} not reached in time; REFUTED {} (confirmed by running the program exactly; {} solver answers not confirmed)",
        tl.programs, tl.failed, tl.proven, tl.nothing, tl.matched, tl.ambiguous, tl.unmatched, tl.justified, tl.incomplete_b, tl.asked, tl.asked - tl.unknown - tl.unasked, tl.unknown, tl.unasked, tl.refuted, tl.unconfirmed
    );
    for (why, n) in &tl.why {
        println!("  not checked: {n:4}  {why}");
    }
}

/// Two runs of `cca precog --json` on one program must give the same
/// report (CCA c8cd316: a query that depended on hash-set order proved in
/// some builds and not in others).
fn same_run(t: &Tools, a: &Args) {
    let cfg = a.config();
    let cases = a.num("cases", 50);
    let from = a.num("from", 1);
    let jobs = a.num("jobs", default_jobs()).min(JOBS_CAP);
    let next = AtomicU64::new(from);
    let differ = Mutex::new(vec![]);
    std::thread::scope(|s| {
        for w in 0..jobs {
            let (next, differ, cfg) = (&next, &differ, &cfg);
            big_stack()
                .spawn_scoped(s, move || loop {
                    let seed = next.fetch_add(1, Ordering::SeqCst);
                    if seed >= from + cases {
                        return;
                    }
                    let p = generate(seed, cfg);
                    let dir = work().join(format!("same{w}"));
                    let _ = std::fs::create_dir_all(&dir);
                    let f = dir.join("prog.zg");
                    std::fs::write(&f, p.render()).unwrap();
                    let once = || engines::analyze(&t, &f, &dir, engines::build_limit());
                    let (x, y) = (once(), once());
                    if x.timed_out || y.timed_out {
                        continue;
                    }
                    if x.stdout != y.stdout {
                        let keep = work().join("same").join(seed.to_string());
                        let _ = std::fs::create_dir_all(&keep);
                        let _ = std::fs::write(keep.join("prog.zg"), p.render());
                        let _ = std::fs::write(keep.join("first.json"), &x.stdout);
                        let _ = std::fs::write(keep.join("second.json"), &y.stdout);
                        println!("seed {seed}: the two reports differ; kept in {}", keep.display());
                        differ.lock().unwrap().push(seed);
                    }
                })
                .expect("spawn");
        }
    });
    let d = differ.into_inner().unwrap();
    println!("{cases} programs from seed {from}: {} gave two different reports {:?}", d.len(), d);
}

/// Translation validation over many programs (check/src/tv.rs): every
/// incorrect transformation printed as it is found, its files kept in
/// work/tv/<seed>/.
fn tv_run(t: &Tools, a: &Args) {
    let cfg = a.config();
    let cases = a.num("cases", 20);
    let from = a.num("from", 1);
    let jobs = a.num("jobs", default_jobs()).min(JOBS_CAP);
    let next = AtomicU64::new(from);
    #[derive(Default)]
    struct T {
        programs: u64,
        failed: u64,
        f: tv::Counts,
        p: tv::Counts,
        incorrect: u64,
    }
    let tally = Mutex::new(T::default());
    std::thread::scope(|s| {
        for w in 0..jobs {
            let (next, tally, cfg) = (&next, &tally, &cfg);
            big_stack()
                .spawn_scoped(s, move || loop {
                    let seed = next.fetch_add(1, Ordering::SeqCst);
                    if seed >= from + cases {
                        return;
                    }
                    let p = generate(seed, cfg);
                    let dir = work().join(format!("tv{w}"));
                    let r = tv::validate(t, &p.render(), &dir);
                    let mut tl = tally.lock().unwrap();
                    tl.programs += 1;
                    match r {
                        Err(e) => {
                            tl.failed += 1;
                            eprintln!("seed {seed}: {e}");
                        }
                        Ok(o) => {
                            let add = |c: &mut tv::Counts, d: &tv::Counts| {
                                c.correct += d.correct;
                                c.incorrect += d.incorrect;
                                c.unproved += d.unproved;
                                c.errors += d.errors;
                            };
                            add(&mut tl.f, &o.per_function);
                            add(&mut tl.p, &o.pipeline);
                            tl.incorrect += o.incorrect.len() as u64;
                            if !o.incorrect.is_empty() {
                                let keep = work().join("tv").join(seed.to_string());
                                let _ = std::fs::create_dir_all(&keep);
                                let _ = std::process::Command::new("cp").arg("-R").arg(dir.join("prog.zg")).arg(dir.join("ir")).arg(&keep).status();
                                for (which, report) in &o.incorrect {
                                    println!("seed {seed}: INCORRECT ({which}), files in {}\n{}", keep.display(), report.lines().take(12).collect::<Vec<_>>().join("\n"));
                                }
                            }
                        }
                    }
                })
                .expect("spawn");
        }
    });
    let tl = tally.into_inner().unwrap();
    println!(
        "\n{} programs ({} not validated); per function: {} correct, {} incorrect, {} not proved, {} Alive2 errors; CCA's pipeline: {} correct, {} incorrect, {} not proved, {} Alive2 errors",
        tl.programs, tl.failed, tl.f.correct, tl.f.incorrect, tl.f.unproved, tl.f.errors, tl.p.correct, tl.p.incorrect, tl.p.unproved, tl.p.errors
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
