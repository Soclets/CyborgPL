//! `oracle file.cpls` — run a CyborgPL program the slow, obvious way.
//!
//! This is the reference interpreter: the executable specification. It knows
//! exactly what `design/` has decided and refuses everything else with a
//! message that says so. Where the design leaves a question open and the
//! interpreter has to pick, the pick is marked "provisional" in the source
//! and listed in oracle/README.md.

#![allow(dead_code)]

mod ast;
mod interp;
mod lexer;
mod parser;
mod value;

use std::path::{Path, PathBuf};
use std::process::exit;

/// Parse one file as the module `module` (empty for the root).
fn parse_file(path: &Path, module: Vec<String>) -> Result<ast::Program, String> {
    let src = std::fs::read_to_string(path).map_err(|e| format!("{}: cannot read: {e}", path.display()))?;
    let toks = lexer::lex(&src).map_err(|e| format!("{}: line {}:{}: {}", path.display(), e.line, e.col, e.msg))?;
    parser::Parser::new_in(toks, module).program()
        .map_err(|e| if e.col == 0 { format!("{}: line {}: {}", path.display(), e.line, e.msg) } else { format!("{}: line {}:{}: {}", path.display(), e.line, e.col, e.msg) })
}

/// A Limb is a directory: `limb.cpls` is the root; every other `.cpls` file is
/// a module named after it, directories nesting (decided 2026-09-19). A lone
/// `.cpls` file is a Limb of one root file.
fn load(path: &Path) -> Result<ast::Program, String> {
    if path.is_file() {
        if path.extension().map(|x| x != "cpls").unwrap_or(true) { return Err("oracle: a CyborgPL source file ends in .cpls".into()); }
        return parse_file(path, Vec::new());
    }
    if !path.is_dir() { return Err(format!("oracle: {} is neither a .cpls file nor a Limb directory", path.display())); }
    let root = path.join("limb.cpls");
    if !root.is_file() { return Err(format!("{}: a Limb directory needs its root file, limb.cpls", path.display())); }
    let mut prog = parse_file(&root, Vec::new())?;
    let mut files = Vec::new();
    collect(path, &mut Vec::new(), &mut files)?;
    files.sort();
    for (module, file) in files {
        // every directory on the way is a module too (implicit, public — provisional)
        for i in 1..module.len() {
            if !prog.modules.iter().any(|m| m.path == module[..i]) {
                prog.modules.push(ast::ModuleDef { path: module[..i].to_vec(), vis: ast::Vis::Pub, line: 0 });
            }
        }
        if prog.modules.iter().any(|m| m.path == module) {
            return Err(format!("{}: module `{}` is also declared as a block elsewhere; a module is a file or a block, not both", file.display(), module.join(".")));
        }
        prog.modules.push(ast::ModuleDef { path: module.clone(), vis: ast::Vis::Pub, line: 0 });
        let part = parse_file(&file, module)?;
        if part.main.is_some() { return Err(format!("{}: MAIN lives in limb.cpls, the root of the Limb", file.display())); }
        prog.funcs.extend(part.funcs);
        prog.structs.extend(part.structs);
        prog.consts.extend(part.consts);
        prog.modules.extend(part.modules);
    }
    Ok(prog)
}

fn collect(dir: &Path, prefix: &mut Vec<String>, out: &mut Vec<(Vec<String>, PathBuf)>) -> Result<(), String> {
    let mut entries: Vec<_> = std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let p = e.path();
        let name = e.file_name().to_string_lossy().to_string();
        if p.is_dir() {
            if name.starts_with('.') || name == "target" { continue; }
            parser::module_name_ok(&name).map_err(|m| format!("{}: `{name}` cannot name a module: {m}", p.display()))?;
            prefix.push(name);
            collect(&p, prefix, out)?;
            prefix.pop();
        } else if let Some(stem) = name.strip_suffix(".cpls") {
            if prefix.is_empty() && stem == "limb" { continue; }
            if stem == "limb" { return Err(format!("{}: `limb` names the root; a module cannot be called that", p.display())); }
            parser::module_name_ok(stem).map_err(|m| format!("{}: `{stem}` cannot name a module: {m}", p.display()))?;
            let mut module = prefix.clone();
            module.push(stem.to_string());
            out.push((module, p));
        }
    }
    Ok(())
}

pub fn run_path(path: &Path, out: Box<dyn std::io::Write>) -> Result<u8, String> {
    let prog = load(path)?;
    let mut interp = interp::Interp::new(&prog, out);
    interp.run().map_err(|e| if e.line == 0 { format!("{}: {}", path.display(), e.msg) } else { format!("{}: line {}: {}", path.display(), e.line, e.msg) })
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: oracle <file.cpls | limb-directory>");
        exit(2);
    }
    match run_path(Path::new(&args[1]), Box::new(std::io::stdout())) {
        Ok(code) => exit(code as i32),
        Err(msg) => { eprintln!("{msg}"); exit(1); }
    }
}
