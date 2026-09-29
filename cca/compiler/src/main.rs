use std::path::Path;
use std::process::exit;

use cca::diag::Diagnostic;
use cca::source::SourceMap;

fn usage() -> ! {
    eprintln!("usage: cca parse <file.cpls>…      parse only, report the first refusal");
    eprintln!("       cca check <file.cpls | limb-dir>…   parse and check");
    eprintln!("       cca mir <file.cpls | limb-dir>       check, then print the flow graph");
    eprintln!("       cca precog <file.cpls | limb-dir>    check, lower, prove, and report");
    eprintln!("       cca build <file.cpls | limb-dir> [-o out]   …then generate, assemble and link");
    eprintln!("       cca run <file.cpls | limb-dir> [-- words…]   build, then run it here with its words; its exit is cca's");
    eprintln!("       cca test <file.cpls | limb-dir>      run every `test fn`, each on its own, and report");
    eprintln!("       cca version                  the compiler's version and the Unicode version it pins");
    exit(2)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else { usage() };
    match cmd.as_str() {
        // "One Unicode version per build, recorded in the lock file beside the
        // compiler version" (reproducible-builds.md, item 7; owner,
        // 2026-09-24): what a lock file will carry, in one place.
        "version" => {
            let (a, b, c) = unicode_segmentation::UNICODE_VERSION;
            println!("cca {}", env!("CARGO_PKG_VERSION"));
            println!("unicode {}.{}.{}", a, b, c);
        }
        "parse" => {
            let mut failed = false;
            for f in &args[1..] {
                if let Err(d) = parse_one(Path::new(f)) {
                    failed = true;
                    eprint!("{}", d);
                }
            }
            if failed {
                exit(1);
            }
        }
        "check" => {
            let mut failed = false;
            for f in &args[1..] {
                if let Err(d) = check_one(Path::new(f)) {
                    failed = true;
                    eprint!("{}", d);
                }
            }
            if failed {
                exit(1);
            }
        }
        "mir" => {
            let mut failed = false;
            for f in &args[1..] {
                if let Err(d) = mir_one(Path::new(f)) {
                    failed = true;
                    eprint!("{}", d);
                }
            }
            if failed {
                exit(1);
            }
        }
        "precog" => {
            let mut failed = false;
            // `--json`: a line per site, for the Precog oracle (report.rs).
            let json = args[1..].iter().any(|a| a == "--json");
            for f in args[1..].iter().filter(|a| *a != "--json") {
                if let Err(d) = precog_one(Path::new(f), json) {
                    failed = true;
                    eprint!("{}", d);
                }
            }
            if failed {
                exit(1);
            }
        }
        "build" => {
            let mut files = Vec::new();
            let mut out = None;
            let mut assert_proofs = false;
            let mut final_build = false;
            let mut i = 1;
            while i < args.len() {
                if args[i] == "--assert-proofs" {
                    assert_proofs = true;
                    i += 1;
                } else if args[i] == "--final" {
                    // `synthia final` (core-features.md): no run-time checks.
                    final_build = true;
                    i += 1;
                } else if args[i] == "-o" && i + 1 < args.len() {
                    out = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    files.push(args[i].clone());
                    i += 1;
                }
            }
            let mut failed = false;
            for f in &files {
                let dest = out.clone().unwrap_or_else(|| Path::new(f).file_stem().unwrap().to_string_lossy().to_string());
                if let Err(d) = build_one(Path::new(f), Path::new(&dest), assert_proofs, final_build, false) {
                    failed = true;
                    eprint!("{}", d);
                }
            }
            if failed {
                exit(1);
            }
        }
        // `cca run` (owner, 2026-09-29: "CCA doesn't need to depend on
        // Synthia"): the build `cca build` makes, run on this machine with
        // this terminal's input and output. What CCA says while building
        // goes to stderr, so what the program prints is all of stdout. How
        // `--input`, a program's own arguments and a cross target's runner
        // work is open, so none is taken. Past `--`, the words are the
        // program's own, its `argv` (owner, 2026-09-29), as the oracle has it.
        "run" => {
            let mut file = None;
            let mut assert_proofs = false;
            let mut final_build = false;
            let split = args.iter().position(|a| a == "--").unwrap_or(args.len());
            let words: Vec<String> = args.get(split + 1..).map(|w| w.to_vec()).unwrap_or_default();
            for a in &args[1..split] {
                match a.as_str() {
                    "--assert-proofs" => assert_proofs = true,
                    "--final" => final_build = true,
                    f if file.is_none() && !f.starts_with('-') => file = Some(f.to_string()),
                    other => {
                        eprintln!("error: `cca run` takes one program and `--final` or `--assert-proofs`; `{}` is not one of them — the program's own words go after `--` (`--input` is not decided yet)", other);
                        exit(2);
                    }
                }
            }
            let Some(file) = file else { usage() };
            let path = Path::new(&file);
            let dir = std::env::temp_dir().join(format!("cca-run-{}", std::process::id()));
            if let Err(e) = std::fs::create_dir_all(&dir) {
                eprintln!("error: cannot make {}: {}", dir.display(), e);
                exit(1);
            }
            let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "program".into());
            let exe = dir.join(&stem);
            if let Err(d) = build_one(path, &exe, assert_proofs, final_build, true) {
                let _ = std::fs::remove_dir_all(&dir);
                eprint!("{}", d);
                exit(1);
            }
            let status = std::process::Command::new(&exe).args(&words).status();
            let _ = std::fs::remove_dir_all(&dir);
            match status {
                Ok(s) => {
                    use std::os::unix::process::ExitStatusExt;
                    // A signal is reported as a shell does: 128 + its number.
                    exit(s.code().unwrap_or_else(|| 128 + s.signal().unwrap_or(0)));
                }
                Err(e) => {
                    eprintln!("error: cannot run {}: {}", exe.display(), e);
                    exit(1);
                }
            }
        }
        // `cca test` (owner, 2026-09-29, `testing.md`): every `test fn`,
        // in the order written, each in a run of its own, so a check that
        // fires ends that test only. The report is the oracle's.
        "test" => {
            let mut file = None;
            let mut assert_proofs = false;
            for a in &args[1..] {
                match a.as_str() {
                    "--assert-proofs" => assert_proofs = true,
                    f if file.is_none() && !f.starts_with('-') => file = Some(f.to_string()),
                    other => {
                        eprintln!("error: `cca test` takes one program or Limb and `--assert-proofs`; `{}` is not one of them", other);
                        exit(2);
                    }
                }
            }
            let Some(file) = file else { usage() };
            let path = Path::new(&file);
            let dir = std::env::temp_dir().join(format!("cca-test-{}", std::process::id()));
            if let Err(e) = std::fs::create_dir_all(&dir) {
                eprintln!("error: cannot make {}: {}", dir.display(), e);
                exit(1);
            }
            let exe = dir.join("tests");
            let names = match build_opts(path, &exe, assert_proofs, false, true, true) {
                Ok(n) => n,
                Err(d) => {
                    let _ = std::fs::remove_dir_all(&dir);
                    eprint!("{}", d);
                    exit(1);
                }
            };
            let mut passed = 0;
            for (k, name) in names.iter().enumerate() {
                let out = std::process::Command::new(&exe).arg(k.to_string()).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::inherit()).stderr(std::process::Stdio::piped()).output();
                let verdict = match out {
                    Ok(o) => test_verdict(&o.status, &String::from_utf8_lossy(&o.stderr)),
                    Err(e) => Some(format!("it could not be run: {}", e)),
                };
                match verdict {
                    None => {
                        passed += 1;
                        println!("test {} ... ok", name);
                    }
                    Some(why) => println!("test {} ... FAILED: {}", name, why),
                }
            }
            let _ = std::fs::remove_dir_all(&dir);
            println!("{} tests: {} passed, {} failed", names.len(), passed, names.len() - passed);
            exit(if passed == names.len() { 0 } else { 1 });
        }
        _ => usage(),
    }
}

/// How one test's run ended, in the report's words: `None` when it passed.
/// A `must` that does not hold said `line N: …` and ended 1; a check (or a
/// proof, when proofs are asserted) says `check: …` / `proof failed: …`
/// and `where: file, line N, column M`.
fn test_verdict(status: &std::process::ExitStatus, stderr: &str) -> Option<String> {
    use std::os::unix::process::ExitStatusExt;
    match status.code() {
        Some(0) => None,
        Some(1) => Some(stderr.lines().find(|l| l.starts_with("line ")).unwrap_or("a `must` did not hold").to_string()),
        Some(code @ (101 | 104)) => {
            let what = stderr.lines().find_map(|l| l.strip_prefix("check: ").or_else(|| l.strip_prefix("proof failed: "))).unwrap_or("a check fired");
            let line = stderr.lines().find_map(|l| l.strip_prefix("where: ")).and_then(|w| w.split(", line ").nth(1)).and_then(|w| w.split(',').next());
            let what = if code == 104 { format!("proof failed: {}", what) } else { what.to_string() };
            Some(match line {
                Some(n) => format!("line {}: {}", n, what),
                None => what,
            })
        }
        Some(code) => Some(format!("it ended with exit code {}", code)),
        None => Some(format!("it was stopped by signal {}", status.signal().unwrap_or(0))),
    }
}

/// `quiet`: what CCA says along the way (Precog's notes, the tally) goes to
/// stderr, for `cca run`, whose stdout is the program's.
fn build_one(path: &Path, out: &Path, assert_proofs: bool, final_build: bool, quiet: bool) -> Result<(), String> {
    build_opts(path, out, assert_proofs, final_build, quiet, false).map(|_| ())
}

/// `tests`: the test binary (`cca test`), and the names of its tests in
/// the order its first argument numbers them.
fn build_opts(path: &Path, out: &Path, assert_proofs: bool, final_build: bool, quiet: bool, tests: bool) -> Result<Vec<String>, String> {
    let say = |s: String| {
        if quiet {
            eprint!("{}", s);
        } else {
            print!("{}", s);
        }
    };
    let mut sources = SourceMap::default();
    let items = cca::load::load(path, &mut sources).map_err(|d| d.render(&sources))?;
    let mut checker = cca::check::Checker::new(items);
    checker.std_stubs = std::env::var("CCA_STD_STUBS").is_ok();
    let program = checker.check_program().map_err(|d| d.render(&sources))?;
    let mut mir = cca::mir::build::build(program);
    cca::mir::verify::program(&mir).map_err(|p| format!("error: the flow graph is not well formed — a bug in CCA:\n  {}\n", p.join("\n  ")))?;
    let owned = cca::mir::ownership::check(&mut mir, &sources);
    if !owned.problems.is_empty() {
        return Err(owned.problems.join(""));
    }
    let analysis = cca::precog::analyse_opts(&mut mir, !assert_proofs);
    for n in cca::precog::notes(&analysis, &mir, &sources) {
        say(format!("\n{}", n));
    }
    let refusals = cca::precog::refusals_opts(&analysis, &mir, &sources, final_build);
    if !refusals.is_empty() {
        return Err(refusals.join(""));
    }
    let ctx = cca::codegen::inkwell::context::Context::create();
    let name = path.file_stem().unwrap().to_string_lossy().to_string();
    let test_names: Vec<String> = mir.functions.iter().filter(|f| f.is_test).map(|f| f.name.clone()).collect();
    let lowered = cca::codegen::lower_for(&ctx, &mir, &sources, &name, assert_proofs, tests).map_err(|problems| {
        let mut s = String::new();
        for p in problems {
            let mut d = cca::diag::Diagnostic::error(format!("not yet: {}", p.what));
            if let Some(sp) = p.span {
                d = d.at(sp);
            }
            d = d.why("CCA can check, prove and report this program; what it cannot do yet is write the machine code for that part");
            s.push_str(&d.render(&sources));
        }
        s
    })?;
    if std::env::var("CCA_IR").is_ok() {
        println!("{}", lowered.module.print_to_string().to_string());
    }
    let obj = out.with_extension("o");
    cca::codegen::write_object(&lowered.module, &obj).map_err(|e| format!("error: {}\n", e))?;
    cca::codegen::link(&obj, out).map_err(|e| format!("error: {}\n", e))?;
    let _ = std::fs::remove_file(&obj);
    // `nocheck` sites carry no check: counted apart, not as "checked".
    let proven = analysis.sites.iter().filter(|r| matches!(r.verdict, cca::mir::Verdict::Proven { .. })).count();
    let trusted = analysis.sites.iter().filter(|r| matches!(r.verdict, cca::mir::Verdict::Trusted)).count();
    let checked = analysis.sites.len() - proven - trusted;
    if trusted > 0 {
        say(format!("{} sites: {} proven, {} checked at run time, {} nocheck\n", analysis.sites.len(), proven, checked, trusted));
    } else {
        say(format!("{} sites: {} proven, {} checked at run time\n", analysis.sites.len(), proven, checked));
    }
    if !quiet {
        println!("wrote {}", out.display());
    }
    Ok(test_names)
}

fn precog_one(path: &Path, json: bool) -> Result<(), String> {
    let mut sources = SourceMap::default();
    let items = cca::load::load(path, &mut sources).map_err(|d| d.render(&sources))?;
    let mut checker = cca::check::Checker::new(items);
    checker.std_stubs = std::env::var("CCA_STD_STUBS").is_ok();
    let program = checker.check_program().map_err(|d| d.render(&sources))?;
    let mut mir = cca::mir::build::build(program);
    cca::mir::verify::program(&mir).map_err(|p| format!("error: the flow graph is not well formed — a bug in CCA:\n  {}\n", p.join("\n  ")))?;
    let owned = cca::mir::ownership::check(&mut mir, &sources);
    if !owned.problems.is_empty() {
        return Err(owned.problems.join(""));
    }
    if std::env::var("CCA_DROPS").is_ok() {
        eprintln!("drops: {} removed as already given away, {} left on some paths only", owned.drops_removed, owned.conditional_drops);
    }
    let analysis = cca::precog::analyse(&mut mir);
    if json {
        print!("{}", cca::precog::report::render_json(&analysis, &mir, &sources));
        return Ok(());
    }
    print!("{}", cca::precog::report::render(&analysis, &mir, &sources));
    for n in cca::precog::notes(&analysis, &mir, &sources) {
        print!("\n{}", n);
    }
    let refusals = cca::precog::refusals(&analysis, &mir, &sources);
    if !refusals.is_empty() {
        return Err(refusals.join(""));
    }
    Ok(())
}

fn mir_one(path: &Path) -> Result<(), String> {
    let mut sources = SourceMap::default();
    let items = cca::load::load(path, &mut sources).map_err(|d| d.render(&sources))?;
    let mut checker = cca::check::Checker::new(items);
    checker.std_stubs = std::env::var("CCA_STD_STUBS").is_ok();
    let program = checker.check_program().map_err(|d| d.render(&sources))?;
    let mir = cca::mir::build::build(program);
    let problems = cca::mir::verify::program(&mir);
    for (i, f) in mir.functions.iter().enumerate() {
        println!("# f{}{}", i, if Some(cca::mir::FnIdx(i as u32)) == mir.main { "  (MAIN)" } else { "" });
        print!("{}", cca::mir::print::function(f, &mir.items));
        println!();
    }
    if let Err(problems) = problems {
        return Err(format!("error: the flow graph is not well formed — this is a bug in CCA, not in the program:\n  {}\n", problems.join("\n  ")));
    }
    Ok(())
}

fn parse_one(path: &Path) -> Result<(), String> {
    let mut sources = SourceMap::default();
    let text = std::fs::read_to_string(path).map_err(|e| format!("error: cannot read {}: {}\n", path.display(), e))?;
    let id = sources.add(path, text);
    let render = |d: Box<Diagnostic>| d.render(&sources);
    let toks = cca::lexer::lex(id, &sources.get(id).text).map_err(render)?;
    cca::parser::parse_file(&toks).map_err(render)?;
    Ok(())
}

fn check_one(path: &Path) -> Result<(), String> {
    let mut sources = SourceMap::default();
    let items = cca::load::load(path, &mut sources).map_err(|d| d.render(&sources))?;
    let mut checker = cca::check::Checker::new(items);
    checker.std_stubs = std::env::var("CCA_STD_STUBS").is_ok();
    let program = checker.check_program().map_err(|d| d.render(&sources))?;
    for _ in 0..program.amp_count {
        println!("What are you? Rust programmer or Claude?");
    }
    Ok(())
}
