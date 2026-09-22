use std::path::Path;
use std::process::exit;

use cca::diag::Diagnostic;
use cca::source::SourceMap;

fn usage() -> ! {
    eprintln!("usage: cca parse <file.cpls>…      parse only, report the first refusal");
    eprintln!("       cca check <file.cpls | limb-dir>…   parse and check");
    eprintln!("       cca mir <file.cpls | limb-dir>       check, then print the flow graph");
    eprintln!("       cca precog <file.cpls | limb-dir>    check, lower, prove, and report");
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
        _ => usage(),
    }
}

fn precog_one(path: &Path) -> Result<(), String> {
    let mut sources = SourceMap::default();
    let items = cca::load::load(path, &mut sources).map_err(|d| d.render(&sources))?;
    let mut checker = cca::check::Checker::new(items);
    checker.std_stubs = std::env::var("CCA_STD_STUBS").is_ok();
    let program = checker.check_program().map_err(|d| d.render(&sources))?;
    let mut mir = cca::mir::build::build(program);
    cca::mir::verify::program(&mir).map_err(|p| format!("error: the flow graph is not well formed — a bug in CCA:\n  {}\n", p.join("\n  ")))?;
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
