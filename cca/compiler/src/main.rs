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
    exit(2)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(cmd) = args.first() else { usage() };
    match cmd.as_str() {
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
            for f in &args[1..] {
                if let Err(d) = precog_one(Path::new(f)) {
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
                if let Err(d) = build_one(Path::new(f), Path::new(&dest), assert_proofs, final_build) {
                    failed = true;
                    eprint!("{}", d);
                }
            }
            if failed {
                exit(1);
            }
        }
        _ => usage(),
    }
}

fn build_one(path: &Path, out: &Path, assert_proofs: bool, final_build: bool) -> Result<(), String> {
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
        print!("\n{}", n);
    }
    let refusals = cca::precog::refusals_opts(&analysis, &mir, &sources, final_build);
    if !refusals.is_empty() {
        return Err(refusals.join(""));
    }
    let ctx = cca::codegen::inkwell::context::Context::create();
    let name = path.file_stem().unwrap().to_string_lossy().to_string();
    let lowered = cca::codegen::lower_opts(&ctx, &mir, &sources, &name, assert_proofs).map_err(|problems| {
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
        println!("{} sites: {} proven, {} checked at run time, {} nocheck", analysis.sites.len(), proven, checked, trusted);
    } else {
        println!("{} sites: {} proven, {} checked at run time", analysis.sites.len(), proven, checked);
    }
    println!("wrote {}", out.display());
    Ok(())
}

fn precog_one(path: &Path) -> Result<(), String> {
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
