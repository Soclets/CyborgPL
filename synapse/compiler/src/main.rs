use std::path::Path;
use std::process::exit;

use brainstem::diag::Diagnostic;
use brainstem::source::SourceMap;

fn usage() -> ! {
    eprintln!("usage: synapse parse <file.nrv>…      parse only, report the first refusal");
    eprintln!("       synapse check <file.nrv | neuron-dir>…   parse and check");
    eprintln!("       synapse mir <file.nrv | neuron-dir>       check, then print the flow graph");
    eprintln!("       synapse cortex <file.nrv | neuron-dir>  check, lower, prove, and report");
    eprintln!("       synapse build <file.nrv | neuron-dir> [-o out] [--target t] [--final] [--emit-llvm dir]   …then generate, assemble and link; --final refuses what is not proven and writes the debug info beside the program (out.dSYM, out.debug); dir/before.ll and dir/after.ll are the LLVM IR before and after LLVM's passes");
    eprintln!("       synapse run <file.nrv | neuron-dir> [-- words…]   build, then run it here with its words; its exit is synapse's");
    eprintln!("       synapse test <file.nrv | neuron-dir>      run every `test fn`, each on its own, and report");
    eprintln!("       synapse prepare [folder/] [<name>, …] [--prefix <prefix>, …] | --all   run those Neurons' build.nrv, in their own folders");
    eprintln!("       synapse generate <file.nrv | neuron-dir>…   write each collection template's members under it");
    eprintln!("       synapse version                  the compiler's version and the Unicode version it pins");
    exit(2)
}

fn main() {
    // The environment's switches are `SYNAPSE_*` (owner, 2026-10-09: "`a`");
    // an old `CCA_*` one is refused, naming the new.
    for (k, _) in std::env::vars_os() {
        if let Some(rest) = k.to_str().and_then(|k| k.strip_prefix("CCA_")) {
            let new = match rest {
                "STD_STUBS" => "SYNAPSE_CORD_STUBS".to_string(),
                "PRECOG_STATS" => "SYNAPSE_CORTEX_STATS".to_string(),
                other => format!("SYNAPSE_{}", other),
            };
            eprintln!("error: `CCA_{}` is an old name: write `{}`", rest, new);
            exit(2);
        }
    }
    // Checking, lowering and proving walk an expression by recursion, and
    // a generated line nesting 5,000 brackets overflowed the main thread's
    // 8 MB (check claude, seed 6241). The work runs on a thread with room
    // for that; only the pages it touches are ever used.
    let worker = std::thread::Builder::new().stack_size(1 << 30).spawn(real_main);
    match worker.map(|h| h.join()) {
        Ok(Ok(())) => {}
        _ => exit(101),
    }
}

fn real_main() {
    // As the system hands them over: a program's own words (after `--`)
    // may be any bytes, and `std::env::args` stops the compiler on one
    // that is not UTF-8. CCA's own words are read as text.
    let raw: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    let args: Vec<String> = raw.iter().map(|a| a.to_string_lossy().into_owned()).collect();
    let Some(cmd) = args.first() else { usage() };
    match cmd.as_str() {
        // "One Unicode version per build, recorded in the lock file beside the
        // compiler version" (reproducible-builds.md, item 7; owner,
        // 2026-09-24): what a lock file will carry, in one place.
        "version" => {
            let (a, b, c) = unicode_segmentation::UNICODE_VERSION;
            println!("synapse {}", env!("CARGO_PKG_VERSION"));
            println!("unicode {}.{}.{}", a, b, c);
        }
        // A collection's templates (owner, 2026-10-13): their members are
        // written into the file, under each template; a build only reads
        // them.
        "generate" => {
            if args.len() < 2 {
                usage();
            }
            let mut failed = false;
            for f in &args[1..] {
                if let Err(d) = generate_in(Path::new(f)) {
                    failed = true;
                    eprint!("{}", d);
                }
            }
            if failed {
                exit(1);
            }
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
        // Precog is Cortex (owner, 2026-10-09: "yea, go"), and the old name
        // is refused like any old spelling ("`b`").
        "precog" => {
            eprintln!("error: `precog` is an old spelling: write `synapse cortex`");
            exit(1);
        }
        "cortex" => {
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
            let mut target: Option<String> = None;
            let mut assert_proofs = false;
            let mut final_build = false;
            let mut emit: Option<String> = None;
            let mut i = 1;
            while i < args.len() {
                if args[i] == "--emit-llvm" && i + 1 < args.len() {
                    emit = Some(args[i + 1].clone());
                    i += 2;
                } else if args[i] == "--assert-proofs" {
                    assert_proofs = true;
                    i += 1;
                } else if args[i] == "--final" {
                    // `synthia final` (core-features.md): no run-time checks.
                    final_build = true;
                    i += 1;
                } else if args[i] == "-o" && i + 1 < args.len() {
                    out = Some(args[i + 1].clone());
                    i += 2;
                } else if args[i] == "--target" && i + 1 < args.len() {
                    target = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    files.push(args[i].clone());
                    i += 1;
                }
            }
            let mut failed = false;
            for f in &files {
                let dest = out.clone().unwrap_or_else(|| stem_of(Path::new(f)));
                if let Err(d) = build_opts(Path::new(f), Path::new(&dest), assert_proofs, final_build, false, false, target.as_deref(), emit.as_deref().map(Path::new)) {
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
            let words: Vec<std::ffi::OsString> = raw.get(split + 1..).map(|w| w.to_vec()).unwrap_or_default();
            let (rest, target) = take_target(&args[1..split]);
            for a in &rest {
                match a.as_str() {
                    "--assert-proofs" => assert_proofs = true,
                    "--final" => final_build = true,
                    f if file.is_none() && !f.starts_with('-') => file = Some(f.to_string()),
                    other => {
                        eprintln!("error: `synapse run` takes one program, `--target <name>`, and `--final` or `--assert-proofs`; `{}` is not one of them — the program's own words go after `--` (`--input` is not decided yet)", other);
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
            if let Err(d) = build_one(path, &exe, assert_proofs, final_build, true, target.as_deref()) {
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
        "prepare" => prepare(&args[1..]),
        "test" => {
            let mut file = None;
            let mut assert_proofs = false;
            let (rest, target) = take_target(&args[1..]);
            for a in &rest {
                match a.as_str() {
                    "--assert-proofs" => assert_proofs = true,
                    f if file.is_none() && !f.starts_with('-') => file = Some(f.to_string()),
                    other => {
                        eprintln!("error: `synapse test` takes one program or Neuron, `--target <name>` and `--assert-proofs`; `{}` is not one of them", other);
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
            let names = match build_opts(path, &exe, assert_proofs, false, true, true, target.as_deref(), None) {
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
            // "where: FILE, line N, column C", told as "line N of FILE"
            // (owner, 2026-10-08: every place names its file).
            let place = stderr.lines().find_map(|l| l.strip_prefix("where: ")).and_then(|w| {
                let (file, rest) = w.split_once(", line ")?;
                let line = rest.split(',').next()?;
                let file = std::path::Path::new(file).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| file.to_string());
                Some(format!("line {} of {}", line, file))
            });
            let what = if code == 104 { format!("proof failed: {}", what) } else { what.to_string() };
            Some(match place {
                Some(p) => format!("{}: {}", p, what),
                None => what,
            })
        }
        Some(code) => Some(format!("it ended with exit code {}", code)),
        None => Some(format!("it was stopped by signal {}", status.signal().unwrap_or(0))),
    }
}

/// `cca prepare` (owner, 2026-09-29, 2026-09-30): the Limb in the current
/// folder, or in a folder given first, written with a `/`; names are Limbs' `name`s, its own or its dependencies',
/// separated by commas (`geometry, money` or `geometry,money`), run in the
/// order given; a name two Limbs share is refused with their prefixes, and
/// `--prefix geo2, geo3` after the names picks by prefix; `--all` runs every Limb's `build.nrv`,
/// dependencies first. Each is built as a program of its own and run in
/// its Limb's folder; the first that fails stops the rest, and its exit is
/// cca's.
fn prepare(rest: &[String]) {
    // A word with a `/` is a folder, a bare word a Limb's name (owner,
    // 2026-09-30: "`/`"): `cca prepare ../app geometry`; `.` and `..` are
    // folders too, since no Limb is named that (owner: "yea"). The folder
    // comes first, as a program does in `cca run` (owner: "yea").
    let (dir, rest) = match rest.first() {
        Some(f) if is_folder(f) => (f.clone(), &rest[1..]),
        _ => (".".to_string(), rest),
    };
    if rest.is_empty() {
        // Refused, a folder alone too (owner, 2026-09-30: "`a`").
        eprintln!("error: `synapse prepare` needs to be told which Neurons to prepare: name them, `synapse prepare geometry, money`, or give `--all` for every one");
        exit(2);
    }
    let pick = if rest.len() == 1 && rest[0] == "--all" {
        brainstem::load::Pick::All
    } else {
        // Names, then `--prefix` and prefixes, each a comma list (owner,
        // 2026-09-30: "yea", "yea"): `cca prepare money --prefix geo2, geo3`.
        let at = rest.iter().position(|a| a == "--prefix");
        let (names, prefixes) = match at {
            Some(i) => (&rest[..i], Some(&rest[i + 1..])),
            None => (rest, None),
        };
        let mut wanted = Vec::new();
        for n in words_list(names, "name", "synapse prepare geometry, money") {
            wanted.push(brainstem::load::Wanted::Name(n));
        }
        if let Some(ps) = prefixes {
            if ps.is_empty() {
                eprintln!("error: `--prefix` is followed by prefixes, separated by commas: `synapse prepare --prefix geo2, geo3`");
                exit(2);
            }
            for p in words_list(ps, "prefix", "synapse prepare --prefix geo2, geo3") {
                wanted.push(brainstem::load::Wanted::Prefix(p));
            }
        }
        brainstem::load::Pick::Listed(wanted)
    };
    let mut sources = SourceMap::default();
    let plan = match brainstem::load::prepare_plan(&mut sources, Path::new(&dir), &pick) {
        Ok(p) => p,
        Err(d) => {
            eprint!("{}", d.render(&sources));
            exit(1);
        }
    };
    if plan.is_empty() {
        println!("no Neuron here has a build.nrv");
        return;
    }
    let dir = std::env::temp_dir().join(format!("cca-prepare-{}", std::process::id()));
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("error: cannot make {}: {}", dir.display(), e);
        exit(1);
    }
    for (k, (name, folder)) in plan.iter().enumerate() {
        println!("prepare: {}", name);
        let exe = dir.join(format!("build-{}", k));
        if let Err(d) = build_one(&folder.join("build.nrv"), &exe, false, false, true, None) {
            let _ = std::fs::remove_dir_all(&dir);
            eprint!("{}", d);
            exit(1);
        }
        let status = std::process::Command::new(&exe).current_dir(folder).status();
        match status {
            Ok(s) if s.success() => {}
            Ok(s) => {
                use std::os::unix::process::ExitStatusExt;
                let code = s.code().unwrap_or_else(|| 128 + s.signal().unwrap_or(0));
                let _ = std::fs::remove_dir_all(&dir);
                eprintln!("error: `{}`'s build.nrv exited with {}, and `synapse prepare` stops there", name, code);
                exit(code);
            }
            Err(e) => {
                let _ = std::fs::remove_dir_all(&dir);
                eprintln!("error: cannot run {}: {}", exe.display(), e);
                exit(1);
            }
        }
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// `--target <name>` taken out of a command's words: the target to build,
/// one the Limb lists (owner, 2026-09-30).
fn take_target(args: &[String]) -> (Vec<String>, Option<String>) {
    let mut rest = Vec::new();
    let mut target = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--target" {
            match args.get(i + 1) {
                Some(t) if target.is_none() => target = Some(t.clone()),
                Some(_) => {
                    eprintln!("error: one `--target` at a time: Synapse builds one target per build");
                    exit(2);
                }
                None => {
                    eprintln!("error: `--target` is followed by the target's name: `--target aarch64-macos`");
                    exit(2);
                }
            }
            i += 2;
        } else {
            rest.push(args[i].clone());
            i += 1;
        }
    }
    (rest, target)
}

/// A folder for `cca prepare`: a word with a `/`, or `.` or `..`.
fn is_folder(w: &str) -> bool {
    w.contains('/') || w == "." || w == ".."
}

/// A comma-separated list of names or prefixes for `cca prepare`, with or
/// without a space after each comma (owner, 2026-09-29).
fn words_list(words: &[String], what: &str, example: &str) -> Vec<String> {
    if let Some(a) = words.iter().find(|a| *a == "--prefix") {
        eprintln!("error: one `{}`, with its prefixes separated by commas: `synapse prepare --prefix geo2, geo3`", a);
        exit(2);
    }
    if let Some(a) = words.iter().find(|a| a.starts_with('-')) {
        eprintln!("error: `synapse prepare` takes names separated by commas, then `--prefix` and prefixes, or `--all` alone; `{}` is none of them", a);
        exit(2);
    }
    if let Some(a) = words.iter().find(|a| is_folder(a)) {
        eprintln!("error: `{}` is a folder, and the folder comes first: `synapse prepare {} …`", a, a);
        exit(2);
    }
    let mut v = Vec::new();
    if words.is_empty() {
        return v;
    }
    for part in words.join(" ").split(',') {
        let n = part.trim();
        if n.is_empty() {
            eprintln!("error: a {} is missing between two commas, or after the last: `{}`", what, example);
            exit(2);
        }
        if n.contains(char::is_whitespace) {
            let plural = if what == "prefix" { "prefixes" } else { "names" };
            eprintln!("error: {} are separated by commas: `{}`", plural, n.split_whitespace().collect::<Vec<_>>().join(", "));
            exit(2);
        }
        v.push(n.to_string());
    }
    v
}

/// `quiet`: what CCA says along the way (Precog's notes, the tally) goes to
/// stderr, for `cca run`, whose stdout is the program's.
/// A program's name from its path: the file's stem, or the folder's name
/// for a Neuron written as `.` or `..` (cord claude: `synapse build .`
/// panicked).
fn stem_of(path: &Path) -> String {
    let full = path.file_stem().map(|_| path.to_path_buf()).or_else(|| std::fs::canonicalize(path).ok());
    full.as_deref().and_then(|p| p.file_stem()).map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "program".into())
}

fn build_one(path: &Path, out: &Path, assert_proofs: bool, final_build: bool, quiet: bool, target: Option<&str>) -> Result<(), String> {
    build_opts(path, out, assert_proofs, final_build, quiet, false, target, None).map(|_| ())
}

/// `tests`: the test binary (`cca test`), and the names of its tests in
/// the order its first argument numbers them.
/// `target`: the one named with `--target`, or the Limb's default.
/// `emit`: a folder for the LLVM IR as CCA hands it over and as LLVM's
/// passes leave it (`--emit-llvm`, for translation validation).
#[allow(clippy::too_many_arguments)]
fn build_opts(path: &Path, out: &Path, assert_proofs: bool, final_build: bool, quiet: bool, tests: bool, target: Option<&str>, emit: Option<&Path>) -> Result<Vec<String>, String> {
    let say = |s: String| {
        if quiet {
            eprint!("{}", s);
        } else {
            print!("{}", s);
        }
    };
    let mut sources = SourceMap::default();
    let items = brainstem::load::load_for(path, &mut sources, brainstem::load::Purpose::Compile(target)).map_err(|d| d.render(&sources))?;
    let mut checker = brainstem::check::Checker::new(items);
    checker.std_stubs = std::env::var("SYNAPSE_CORD_STUBS").is_ok();
    let program = checker.check_program().map_err(|d| d.render(&sources))?;
    let mut mir = brainstem::mir::build::build(program);
    brainstem::mir::verify::program(&mir).map_err(|p| format!("error: the flow graph is not well formed — a bug in Synapse:\n  {}\n", p.join("\n  ")))?;
    let owned = brainstem::mir::ownership::check(&mut mir, &sources);
    if !owned.problems.is_empty() {
        return Err(owned.problems.join(""));
    }
    let analysis = brainstem::cortex::analyse_opts(&mut mir, !assert_proofs);
    for n in brainstem::cortex::notes(&analysis, &mir, &sources) {
        say(format!("\n{}", n));
    }
    let refusals = brainstem::cortex::refusals_opts(&analysis, &mir, &sources, final_build);
    if !refusals.is_empty() {
        return Err(refusals.join(""));
    }
    // Parallel `for` (`parallelism.md`): a `for para` whose iterations
    // conflict is an error; the rest is decided as the loops are written.
    let plan = brainstem::par::plan(&mir);
    if std::env::var("SYNAPSE_PAR").is_ok() {
        for ((fi, pre), pl) in &plan.loops {
            let f = &mir.functions[*fi];
            let sp = f.span(pl.fl.span);
            let at = format!("line {}", sources.get(sp.file).line_col(sp.start).0);
            eprintln!("par: {} (in {}, entered from b{}): {} block(s), {} private, {} fold(s), work {} per trip", at, f.name, pre, pl.blocks.len(), pl.private.len(), pl.folds.len(), pl.work);
        }
        for (fi, span, why) in &plan.declined {
            let f = &mir.functions[*fi];
            let sp = f.span(*span);
            eprintln!("par: line {} (in {}) stays in order: {}", sources.get(sp.file).line_col(sp.start).0, f.name, why);
        }
    }
    if !plan.errors.is_empty() {
        let mut s = String::new();
        for (fi, span, why) in &plan.errors {
            let d = brainstem::diag::Diagnostic::error(format!("this `for para` cannot run in slices: {}", why)).at(mir.functions[*fi].span(*span));
            s.push_str(&d.render(&sources));
        }
        return Err(s);
    }
    let ctx = brainstem::codegen::inkwell::context::Context::create();
    let name = stem_of(path);
    let test_names: Vec<String> = mir.functions.iter().filter(|f| f.is_test).map(|f| f.name.clone()).collect();
    let lowered = brainstem::codegen::lower_for(&ctx, &mir, &sources, &name, assert_proofs, tests).map_err(|problems| {
        let mut s = String::new();
        for p in problems {
            let mut d = brainstem::diag::Diagnostic::error(format!("not yet: {}", p.what));
            if let Some(sp) = p.span {
                d = d.at(sp);
            }
            d = d.why("Synapse can check, prove and report this program; what it cannot do yet is write the machine code for that part");
            s.push_str(&d.render(&sources));
        }
        s
    })?;
    if std::env::var("SYNAPSE_IR").is_ok() {
        println!("{}", lowered.module.print_to_string().to_string());
    }
    let obj = out.with_extension("o");
    brainstem::codegen::write_object(&lowered.module, &obj, emit).map_err(|e| format!("error: {}\n", e))?;
    brainstem::codegen::link(&obj, out).map_err(|e| format!("error: {}\n", e))?;
    // A final build's debug info goes beside it (owner, 2026-10-14); a
    // working build keeps it inside.
    let debug = if final_build { Some(brainstem::codegen::split_debug(out).map_err(|e| format!("error: {}\n", e))?) } else { None };
    // Every build ends with its size, broken down (owner, 2026-10-08).
    let size = brainstem::codegen::size::report(&mir, &obj, out);
    let _ = std::fs::remove_file(&obj);
    // `nocheck` sites carry no check: counted apart, not as "checked".
    let proven = analysis.sites.iter().filter(|r| matches!(r.verdict, brainstem::mir::Verdict::Proven { .. })).count();
    let trusted = analysis.sites.iter().filter(|r| matches!(r.verdict, brainstem::mir::Verdict::Trusted)).count();
    let checked = analysis.sites.len() - proven - trusted;
    if trusted > 0 {
        say(format!("{} sites: {} proven, {} checked at run time, {} !check\n", analysis.sites.len(), proven, checked, trusted));
    } else {
        say(format!("{} sites: {} proven, {} checked at run time\n", analysis.sites.len(), proven, checked));
    }
    say(size.unwrap_or_else(|e| format!("size: not measured: {}\n", e)));
    if !quiet {
        println!("wrote {}", out.display());
        if let Some(d) = &debug {
            println!("wrote {}, its debug info", d.display());
        }
    }
    Ok(test_names)
}

fn precog_one(path: &Path, json: bool) -> Result<(), String> {
    let mut sources = SourceMap::default();
    let items = brainstem::load::load(path, &mut sources).map_err(|d| d.render(&sources))?;
    let mut checker = brainstem::check::Checker::new(items);
    checker.std_stubs = std::env::var("SYNAPSE_CORD_STUBS").is_ok();
    let program = checker.check_program().map_err(|d| d.render(&sources))?;
    let mut mir = brainstem::mir::build::build(program);
    brainstem::mir::verify::program(&mir).map_err(|p| format!("error: the flow graph is not well formed — a bug in Synapse:\n  {}\n", p.join("\n  ")))?;
    let owned = brainstem::mir::ownership::check(&mut mir, &sources);
    if !owned.problems.is_empty() {
        return Err(owned.problems.join(""));
    }
    if std::env::var("SYNAPSE_DROPS").is_ok() {
        eprintln!("drops: {} removed as already given away, {} left on some paths only", owned.drops_removed, owned.conditional_drops);
    }
    let analysis = brainstem::cortex::analyse(&mut mir);
    if json {
        print!("{}", brainstem::cortex::report::render_json(&analysis, &mir, &sources));
        return Ok(());
    }
    print!("{}", brainstem::cortex::report::render(&analysis, &mir, &sources));
    for n in brainstem::cortex::notes(&analysis, &mir, &sources) {
        print!("\n{}", n);
    }
    let refusals = brainstem::cortex::refusals(&analysis, &mir, &sources);
    if !refusals.is_empty() {
        return Err(refusals.join(""));
    }
    Ok(())
}

fn mir_one(path: &Path) -> Result<(), String> {
    let mut sources = SourceMap::default();
    let items = brainstem::load::load(path, &mut sources).map_err(|d| d.render(&sources))?;
    let mut checker = brainstem::check::Checker::new(items);
    checker.std_stubs = std::env::var("SYNAPSE_CORD_STUBS").is_ok();
    let program = checker.check_program().map_err(|d| d.render(&sources))?;
    let mir = brainstem::mir::build::build(program);
    let problems = brainstem::mir::verify::program(&mir);
    for (i, f) in mir.functions.iter().enumerate() {
        println!("# f{}{}", i, if Some(brainstem::mir::FnIdx(i as u32)) == mir.main { "  (MAIN)" } else { "" });
        print!("{}", brainstem::mir::print::function(f, &mir.items));
        println!();
    }
    if let Err(problems) = problems {
        return Err(format!("error: the flow graph is not well formed — this is a bug in Synapse, not in the program:\n  {}\n", problems.join("\n  ")));
    }
    Ok(())
}

/// `synapse generate` on a file, or on every `.nrv` file of a Neuron.
fn generate_in(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        brainstem::parser::set_neuron_root(path);
        let mut files = Vec::new();
        let mut dirs = vec![path.to_path_buf()];
        while let Some(d) = dirs.pop() {
            let entries = std::fs::read_dir(&d).map_err(|e| format!("error: cannot read {}: {}\n", d.display(), e))?;
            for e in entries.flatten() {
                let p = e.path();
                let hidden = p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with('.') || n == "target");
                if p.is_dir() && !hidden {
                    dirs.push(p);
                } else if p.extension().is_some_and(|x| x == "nrv") {
                    files.push(p);
                }
            }
        }
        files.sort();
        for f in files {
            generate_one(&f)?;
        }
        return Ok(());
    }
    brainstem::parser::set_neuron_root(path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new(".")));
    generate_one(path)
}

fn generate_one(path: &Path) -> Result<(), String> {
    let mut sources = SourceMap::default();
    let text = std::fs::read_to_string(path).map_err(|e| format!("error: cannot read {}: {}\n", path.display(), e))?;
    let id = sources.add(path, text.clone());
    let render = |d: Box<Diagnostic>| d.render(&sources);
    let toks = brainstem::lexer::lex(id, &sources.get(id).text).map_err(render)?;
    let file = brainstem::parser::parse_file(&toks).map_err(render)?;
    if file.templates.is_empty() {
        return Ok(());
    }
    let new = brainstem::template::generate(&file, &text).map_err(render)?;
    if new != text {
        std::fs::write(path, &new).map_err(|e| format!("error: cannot write {}: {}\n", path.display(), e))?;
        println!("{}: {} template(s) written", path.display(), file.templates.len());
    } else {
        println!("{}: {} template(s), already as they make", path.display(), file.templates.len());
    }
    Ok(())
}

fn parse_one(path: &Path) -> Result<(), String> {
    let mut sources = SourceMap::default();
    let text = std::fs::read_to_string(path).map_err(|e| format!("error: cannot read {}: {}\n", path.display(), e))?;
    let id = sources.add(path, text);
    let render = |d: Box<Diagnostic>| d.render(&sources);
    let toks = brainstem::lexer::lex(id, &sources.get(id).text).map_err(render)?;
    brainstem::parser::parse_file(&toks).map_err(render)?;
    Ok(())
}

fn check_one(path: &Path) -> Result<(), String> {
    let mut sources = SourceMap::default();
    let items = brainstem::load::load(path, &mut sources).map_err(|d| d.render(&sources))?;
    let mut checker = brainstem::check::Checker::new(items);
    checker.std_stubs = std::env::var("SYNAPSE_CORD_STUBS").is_ok();
    checker.check_program().map_err(|d| d.render(&sources))?;
    Ok(())
}
