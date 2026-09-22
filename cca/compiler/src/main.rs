use std::path::Path;
use std::process::exit;

use cca::diag::Diagnostic;
use cca::source::SourceMap;

fn usage() -> ! {
    eprintln!("usage: cca parse <file.cpls>…      parse only, report the first refusal");
    eprintln!("       cca check <file.cpls | limb-dir>…   parse and check");
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
        _ => usage(),
    }
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
