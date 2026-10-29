use std::path::Path;
use std::process::exit;

use kayo::diag::Diagnostic;
use kayo::source::SourceMap;

fn usage() -> ! {
    eprintln!("usage: kayo parse <file.cy>…      parse only, report the first refusal");
    eprintln!("       kayo check <file.cy | kit-dir>…   parse and check");
    eprintln!("       kayo mir <file.cy | kit-dir>       check, then print the flow graph");
    eprintln!("       kayo prove <file.cy | kit-dir>   check, lower, prove, and report");
    eprintln!("       kayo build <file.cy | kit-dir> [-o out] [--target t] [--final] [--emit-llvm dir] [--emit-llvm-tv dir]   …then generate, assemble and link; --final refuses what is not proven and writes the debug info beside the program (out.dSYM, out.debug); dir/before.ll and dir/after.ll are the LLVM IR before and after LLVM's passes, and --emit-llvm-tv writes dir/tv-before.ll and dir/tv-after.ll, the same with inlining off, for translation validation");
    eprintln!("       kayo run <file.cy | kit-dir> [-- words…]   build, then run it here with its words; its exit is kayo's");
    eprintln!("       kayo test <file.cy | kit-dir>      run every `test fn`, each on its own, and report");
    eprintln!("       kayo prepare [folder/] [<name>, …] [--prefix <prefix>, …] | --all   run those kits' build.cy, in their own folders");
    eprintln!("       kayo generate <file.cy | kit-dir>…   write each collection template's members under it");
    eprintln!("       kayo version                  the tools this kayo belongs to and the Unicode version it pins");
    exit(2)
}

/// The CyDT release this KAYO\n is, when it is one (owner, 2026-10-16: a
/// release names itself `CyDT 1 KAYO\n`); none between releases, where it
/// names the commit it was built from instead.
const RELEASE: Option<u32> = None;

fn main() {
    // The environment's switches take the command's name, `KAYO_*`
    // (owner, 2026-10-09: "`a`"); an old `CCA_*`, `SYNAPSE_*` or `ZOMP_*`
    // one is refused, naming the new.
    for (k, _) in std::env::vars_os() {
        let Some(k) = k.to_str() else { continue };
        let Some((old, rest)) = ["CCA_", "SYNAPSE_", "ZOMP_"].iter().find_map(|o| k.strip_prefix(o).map(|r| (*o, r))) else { continue };
        let new = match rest {
            "STD_STUBS" | "CORD_STUBS" => "KAYO_STD_STUBS".to_string(),
            "PRECOG_STATS" | "CORTEX_STATS" => "KAYO_PRECOG_STATS".to_string(),
            other => format!("KAYO_{}", other),
        };
        eprintln!("error: `{}{}` is an old name: write `{}`", old, rest, new);
        exit(2);
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
    let mut raw: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    // `--message-format json` (design/diagnostics.md rule 3): each error
    // and check one JSON line, its fixes as replacements an editor applies.
    let end = raw.iter().position(|a| a == "--").unwrap_or(raw.len());
    if let Some(i) = raw[..end].iter().position(|a| a == "--message-format") {
        match raw.get(i + 1).and_then(|f| f.to_str()) {
            Some("json") => kayo::diag::json_messages(),
            Some("text") => {}
            _ => {
                eprintln!("error: `--message-format` is followed by `json` or `text`");
                exit(2);
            }
        }
        raw.drain(i..(i + 2).min(raw.len()));
    }
    let args: Vec<String> = raw.iter().map(|a| a.to_string_lossy().into_owned()).collect();
    let Some(cmd) = args.first() else { usage() };
    match cmd.as_str() {
        // "One Unicode version per build, recorded in the lock file beside the
        // compiler version" (reproducible-builds.md, item 7; owner,
        // 2026-09-24): what a lock file will carry, in one place.
        "version" => {
            let (a, b, c) = unicode_segmentation::UNICODE_VERSION;
            // KAYO\n is named by its tools: a release by its number, `CyDT 1
            // KAYO\n`, whole numbers and no dots, and a build between
            // releases by its commit, `CyDT 5eac982a KAYO\n` (owner,
            // 2026-10-16, distribution.md).
            match RELEASE {
                Some(n) => println!("CyDT {} KAYO\\n", n),
                None => println!("CyDT {} KAYO\\n", env!("KAYO_COMMIT")),
            }
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
        // The command is `prove` (owner, 2026-10-14; `precog` was refused
        // from 2026-10-09, "`b`"), though the prover is Precog again
        // (2026-10-16).
        "precog" => {
            eprintln!("error: `precog` is an old spelling: write `kayo prove`");
            exit(1);
        }
        // Cortex was the prover's name from 2026-10-09 to 2026-10-14.
        "cortex" => {
            eprintln!("error: `kayo cortex` is an old name: write `kayo prove`");
            exit(1);
        }
        "prove" => {
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
                } else if args[i] == "--emit-llvm-tv" && i + 1 < args.len() {
                    // For translation validation (owner, 2026-10-15: "(a)").
                    kayo::codegen::set_tv_dir(Some(std::path::PathBuf::from(&args[i + 1])));
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
            // Nothing named is nothing built: say how, not a silent 0.
            if files.is_empty() {
                usage();
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
                        eprintln!("error: `kayo run` takes one program, `--target <name>`, and `--final` or `--assert-proofs`; `{}` is not one of them — the program's own words go after `--` (`--input` is not decided yet)", other);
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
                        eprintln!("error: `kayo test` takes one program or kit, `--target <name>` and `--assert-proofs`; `{}` is not one of them", other);
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
/// proof, when proofs are asserted) says `check[name] at file:line:col` /
/// `proof failed: …` with `where: file, line N, column M`.
fn test_verdict(status: &std::process::ExitStatus, stderr: &str) -> Option<String> {
    use std::os::unix::process::ExitStatusExt;
    match status.code() {
        Some(0) => None,
        Some(1) => Some(stderr.lines().find(|l| l.starts_with("line ")).unwrap_or("a `must` did not hold").to_string()),
        Some(code @ (101 | 104)) => {
            // A check says `check[name] at FILE:LINE:COL`, its sentence on
            // `about:` until its fix is written, its name after that.
            let first = stderr.lines().find(|l| l.starts_with("check[")).unwrap_or("");
            let name = first.strip_prefix("check[").and_then(|r| r.split_once(']')).map(|(n, _)| n.to_string());
            let what = stderr.lines().find_map(|l| l.strip_prefix("about: ").or_else(|| l.strip_prefix("proof failed: "))).map(str::to_string).or(name).unwrap_or_else(|| "a check fired".into());
            // Told as "line N of FILE" (owner, 2026-10-08: every place names
            // its file).
            let place = first.split_once("] at ").and_then(|(_, at)| {
                let mut parts = at.rsplitn(3, ':');
                let _col = parts.next()?;
                let line = parts.next()?;
                let file = parts.next()?;
                let file = std::path::Path::new(file).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| file.to_string());
                Some(format!("line {} of {}", line, file))
            }).or_else(|| stderr.lines().find_map(|l| l.strip_prefix("where: ")).and_then(|w| {
                let (file, rest) = w.split_once(", line ")?;
                let line = rest.split(',').next()?;
                let file = std::path::Path::new(file).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| file.to_string());
                Some(format!("line {} of {}", line, file))
            }));
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

/// `cca prepare` (owner, 2026-09-29, 2026-09-30): the Kit in the current
/// folder, or in a folder given first, written with a `/`; names are Kits' `name`s, its own or its dependencies',
/// separated by commas (`geometry, money` or `geometry,money`), run in the
/// order given; a name two Kits share is refused with their prefixes, and
/// `--prefix geo2, geo3` after the names picks by prefix; `--all` runs every Kit's `build.nrv`,
/// dependencies first. Each is built as a program of its own and run in
/// its Kit's folder; the first that fails stops the rest, and its exit is
/// cca's.
fn prepare(rest: &[String]) {
    // A word with a `/` is a folder, a bare word a Kit's name (owner,
    // 2026-09-30: "`/`"): `cca prepare ../app geometry`; `.` and `..` are
    // folders too, since no Kit is named that (owner: "yea"). The folder
    // comes first, as a program does in `cca run` (owner: "yea").
    let (dir, rest) = match rest.first() {
        Some(f) if is_folder(f) => (f.clone(), &rest[1..]),
        _ => (".".to_string(), rest),
    };
    if rest.is_empty() {
        // Refused, a folder alone too (owner, 2026-09-30: "`a`").
        eprintln!("error: `kayo prepare` needs to be told which kits to prepare: name them, `kayo prepare geometry, money`, or give `--all` for every one");
        exit(2);
    }
    let pick = if rest.len() == 1 && rest[0] == "--all" {
        kayo::load::Pick::All
    } else {
        // Names, then `--prefix` and prefixes, each a comma list (owner,
        // 2026-09-30: "yea", "yea"): `cca prepare money --prefix geo2, geo3`.
        let at = rest.iter().position(|a| a == "--prefix");
        let (names, prefixes) = match at {
            Some(i) => (&rest[..i], Some(&rest[i + 1..])),
            None => (rest, None),
        };
        let mut wanted = Vec::new();
        for n in words_list(names, "name", "kayo prepare geometry, money") {
            wanted.push(kayo::load::Wanted::Name(n));
        }
        if let Some(ps) = prefixes {
            if ps.is_empty() {
                eprintln!("error: `--prefix` is followed by prefixes, separated by commas: `kayo prepare --prefix geo2, geo3`");
                exit(2);
            }
            for p in words_list(ps, "prefix", "kayo prepare --prefix geo2, geo3") {
                wanted.push(kayo::load::Wanted::Prefix(p));
            }
        }
        kayo::load::Pick::Listed(wanted)
    };
    let mut sources = SourceMap::default();
    let plan = match kayo::load::prepare_plan(&mut sources, Path::new(&dir), &pick) {
        Ok(p) => p,
        Err(d) => {
            eprint!("{}", d.render(&sources));
            exit(1);
        }
    };
    if plan.is_empty() {
        println!("no kit here has a build.cy");
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
        if let Err(d) = build_one(&folder.join("build.cy"), &exe, false, false, true, None) {
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
                eprintln!("error: `{}`'s build.cy exited with {}, and `kayo prepare` stops there", name, code);
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
/// one the Kit lists (owner, 2026-09-30).
fn take_target(args: &[String]) -> (Vec<String>, Option<String>) {
    let mut rest = Vec::new();
    let mut target = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--target" {
            match args.get(i + 1) {
                Some(t) if target.is_none() => target = Some(t.clone()),
                Some(_) => {
                    eprintln!("error: one `--target` at a time: kayo builds one target per build");
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
        eprintln!("error: one `{}`, with its prefixes separated by commas: `kayo prepare --prefix geo2, geo3`", a);
        exit(2);
    }
    if let Some(a) = words.iter().find(|a| a.starts_with('-')) {
        eprintln!("error: `kayo prepare` takes names separated by commas, then `--prefix` and prefixes, or `--all` alone; `{}` is none of them", a);
        exit(2);
    }
    if let Some(a) = words.iter().find(|a| is_folder(a)) {
        eprintln!("error: `{}` is a folder, and the folder comes first: `kayo prepare {} …`", a, a);
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

/// Each compile phase and how long it took, for the build report.
struct Laps {
    lap: std::time::Instant,
    phases: Vec<(&'static str, std::time::Duration)>,
}

impl Laps {
    fn mark(&mut self, name: &'static str) {
        self.phases.push((name, self.lap.elapsed()));
        self.lap = std::time::Instant::now();
    }
}

fn build_one(path: &Path, out: &Path, assert_proofs: bool, final_build: bool, quiet: bool, target: Option<&str>) -> Result<(), String> {
    build_opts(path, out, assert_proofs, final_build, quiet, false, target, None).map(|_| ())
}

/// `tests`: the test binary (`cca test`), and the names of its tests in
/// the order its first argument numbers them.
/// `target`: the one named with `--target`, or the Kit's default.
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
    // Every phase timed, for the build report (owner, 2026-10-19).
    let started_at = std::time::SystemTime::now();
    let started = std::time::Instant::now();
    let mut laps = Laps { lap: std::time::Instant::now(), phases: Vec::new() };
    let mut sources = SourceMap::default();
    let items = kayo::load::load_for(path, &mut sources, kayo::load::Purpose::Compile(target)).map_err(|d| d.render(&sources))?;
    laps.mark("Reading the files and kits");
    let mut checker = kayo::check::Checker::new(items);
    checker.std_stubs = std::env::var("KAYO_STD_STUBS").is_ok();
    let program = checker.check_program().map_err(|d| d.render(&sources))?;
    let twins: Vec<String> = kayo::check::case_twins(&program).into_iter().map(|(at, msg)| kayo::diag::Diagnostic::note(msg).at(at).render(&sources)).collect();
    laps.mark("Checking");
    let mut mir = kayo::mir::build::build(program);
    kayo::mir::verify::program(&mir).map_err(|p| format!("error: the flow graph is not well formed — a bug in kayo:\n  {}\n", p.join("\n  ")))?;
    laps.mark("Lowering to the flow graph");
    let owned = kayo::mir::ownership::check(&mut mir, &sources);
    if !owned.problems.is_empty() {
        return Err(owned.problems.join(""));
    }
    laps.mark("Ownership and borrows");
    let analysis = kayo::precog::analyse_opts(&mut mir, !assert_proofs);
    // Every note goes in the build report only; there are no warnings
    // (owner, 2026-10-22; diagnostics.md rule 4). Each `use` has one.
    let mut notes = kayo::precog::notes(&analysis, &mir, &sources);
    for (at, name, path) in kayo::uses::seen() {
        notes.push(kayo::diag::Diagnostic::note(format!("`{}` stands for `{}` in this file (its `use`)", name, path)).at(at).render(&sources));
    }
    notes.extend(kayo::precog::paged::notes(&mir, &sources));
    notes.extend(twins);
    let refusals = kayo::precog::refusals_opts(&analysis, &mir, &sources, final_build);
    if !refusals.is_empty() {
        return Err(refusals.join(""));
    }
    laps.mark("Precog");
    // Parallel `for` (`parallelism.md`): a `for para` whose iterations
    // conflict is an error; the rest is decided as the loops are written.
    let plan = kayo::par::plan(&mir);
    if std::env::var("KAYO_PAR").is_ok() {
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
            let d = kayo::diag::Diagnostic::error(format!("this `for %para` cannot run in slices: {}", why)).named("mcps::para_cannot_split").at(mir.functions[*fi].span(*span)).fix_edit(mir.functions[*fi].span(*span), kayo::diag::Edit::Swap(vec!["para ".into()]), "");
            s.push_str(&d.render(&sources));
        }
        return Err(s);
    }
    laps.mark("Planning parallel loops");
    let ctx = kayo::codegen::inkwell::context::Context::create();
    let name = stem_of(path);
    let test_names: Vec<String> = mir.functions.iter().filter(|f| f.is_test).map(|f| f.name.clone()).collect();
    let lowered = kayo::codegen::lower_for(&ctx, &mir, &sources, &name, assert_proofs, tests).map_err(|problems| {
        let mut s = String::new();
        for p in problems {
            let mut d = kayo::diag::Diagnostic::error(format!("not yet: {}", p.what)).named("not_built_yet");
            if let Some(sp) = p.span {
                d = d.at(sp);
            }
            d = d.why("kayo can check, prove and report this program; what it cannot do yet is write the machine code for that part");
            s.push_str(&d.render(&sources));
        }
        s
    })?;
    if std::env::var("KAYO_IR").is_ok() {
        println!("{}", lowered.module.print_to_string().to_string());
    }
    laps.mark("Generating LLVM IR");
    let obj = out.with_extension("o");
    let object = kayo::codegen::write_object(&lowered.module, &obj, emit).map_err(|e| format!("error: {}\n", e))?;
    laps.lap = std::time::Instant::now();
    laps.phases.push(("LLVM's passes", object.optimize_time));
    laps.phases.push(("Writing the object", object.emit_time));
    kayo::codegen::link(&obj, out).map_err(|e| format!("error: {}\n", e))?;
    laps.mark("Linking");
    // A final build's debug info goes beside it (owner, 2026-10-14); a
    // working build keeps it inside.
    let debug = if final_build { Some(kayo::codegen::split_debug(out).map_err(|e| format!("error: {}\n", e))?) } else { None };
    if debug.is_some() {
        laps.mark("Writing the debug info apart");
    }
    // Every build measures its size, broken down (owner, 2026-10-08).
    let sizes = kayo::codegen::size::sizes(&mir, &obj, out);
    let _ = std::fs::remove_file(&obj);
    laps.mark("Measuring sizes");
    // Everything else goes in the report beside the program (owner,
    // 2026-10-19); the terminal keeps errors, warnings and one line.
    let c_lines = kayo::cheader::read_lines();
    let facts = kayo::report::Facts {
        source: path,
        program: out,
        final_build,
        assert_proofs,
        started: started_at,
        total: started.elapsed(),
        phases: &laps.phases,
        mir: &mir,
        sources: &sources,
        analysis: &analysis,
        plan: &plan,
        sizes: sizes.as_ref().map_err(|e| e.as_str()),
        structs: &lowered.structs,
        consts: &lowered.consts,
        object: &object,
        notes: &notes,
        debug: debug.as_deref(),
        c_lines: &c_lines,
        tests,
    };
    let report = kayo::report::path_for(out);
    if let Err(e) = std::fs::write(&report, kayo::report::render(&facts)) {
        eprintln!("error: the build report could not be written to {}: {}", report.display(), e);
    }
    say(kayo::report::summary_line(&facts, &report));
    Ok(test_names)
}

fn precog_one(path: &Path, json: bool) -> Result<(), String> {
    let mut sources = SourceMap::default();
    let items = kayo::load::load(path, &mut sources).map_err(|d| d.render(&sources))?;
    let mut checker = kayo::check::Checker::new(items);
    checker.std_stubs = std::env::var("KAYO_STD_STUBS").is_ok();
    let program = checker.check_program().map_err(|d| d.render(&sources))?;
    let mut mir = kayo::mir::build::build(program);
    kayo::mir::verify::program(&mir).map_err(|p| format!("error: the flow graph is not well formed — a bug in kayo:\n  {}\n", p.join("\n  ")))?;
    let owned = kayo::mir::ownership::check(&mut mir, &sources);
    if !owned.problems.is_empty() {
        return Err(owned.problems.join(""));
    }
    if std::env::var("KAYO_DROPS").is_ok() {
        eprintln!("drops: {} removed as already given away, {} left on some paths only", owned.drops_removed, owned.conditional_drops);
    }
    let analysis = kayo::precog::analyse(&mut mir);
    if json {
        print!("{}", kayo::precog::report::render_json(&analysis, &mir, &sources));
        return Ok(());
    }
    print!("{}", kayo::precog::report::render(&analysis, &mir, &sources));
    for n in kayo::precog::notes(&analysis, &mir, &sources) {
        print!("\n{}", n);
    }
    let refusals = kayo::precog::refusals(&analysis, &mir, &sources);
    if !refusals.is_empty() {
        return Err(refusals.join(""));
    }
    Ok(())
}

fn mir_one(path: &Path) -> Result<(), String> {
    let mut sources = SourceMap::default();
    let items = kayo::load::load(path, &mut sources).map_err(|d| d.render(&sources))?;
    let mut checker = kayo::check::Checker::new(items);
    checker.std_stubs = std::env::var("KAYO_STD_STUBS").is_ok();
    let program = checker.check_program().map_err(|d| d.render(&sources))?;
    let mir = kayo::mir::build::build(program);
    let problems = kayo::mir::verify::program(&mir);
    for (i, f) in mir.functions.iter().enumerate() {
        println!("# f{}{}", i, if Some(kayo::mir::FnIdx(i as u32)) == mir.main { "  (MAIN)" } else { "" });
        print!("{}", kayo::mir::print::function(f, &mir.items));
        println!();
    }
    if let Err(problems) = problems {
        return Err(format!("error: the flow graph is not well formed — this is a bug in kayo, not in the program:\n  {}\n", problems.join("\n  ")));
    }
    Ok(())
}

/// `synapse generate` on a file, or on every `.nrv` file of a Neuron.
fn generate_in(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        kayo::parser::set_neuron_root(path);
        let mut files = Vec::new();
        let mut dirs = vec![path.to_path_buf()];
        while let Some(d) = dirs.pop() {
            let entries = std::fs::read_dir(&d).map_err(|e| format!("error: cannot read {}: {}\n", d.display(), e))?;
            for e in entries.flatten() {
                let p = e.path();
                let hidden = p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with('.') || n == "target");
                if p.is_dir() && !hidden {
                    dirs.push(p);
                } else if p.extension().is_some_and(|x| x == "cy") {
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
    kayo::parser::set_neuron_root(path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new(".")));
    generate_one(path)
}

fn generate_one(path: &Path) -> Result<(), String> {
    let mut sources = SourceMap::default();
    let text = std::fs::read_to_string(path).map_err(|e| format!("error: cannot read {}: {}\n", path.display(), e))?;
    let id = sources.add(path, text.clone());
    let render = |d: Box<Diagnostic>| d.render(&sources);
    let toks = sources.lex_file(id).map_err(render)?;
    let file = kayo::parser::parse_file(&toks).map_err(render)?;
    if file.templates.is_empty() {
        return Ok(());
    }
    let new = kayo::template::generate(&file, &text).map_err(render)?;
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
    let toks = sources.lex_file(id).map_err(render)?;
    kayo::parser::parse_file(&toks).map_err(render)?;
    Ok(())
}

fn check_one(path: &Path) -> Result<(), String> {
    let mut sources = SourceMap::default();
    let items = kayo::load::load(path, &mut sources).map_err(|d| d.render(&sources))?;
    let mut checker = kayo::check::Checker::new(items);
    checker.std_stubs = std::env::var("KAYO_STD_STUBS").is_ok();
    checker.check_program().map_err(|d| d.render(&sources))?;
    Ok(())
}
