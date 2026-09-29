//! `oracle file.cpls` — run a CyborgPL program the slow, obvious way.
//!
//! This is the reference interpreter: the executable specification. It knows
//! exactly what `design/` has decided and refuses everything else with a
//! message that says so. Where the design leaves a question open and the
//! interpreter has to pick, the pick is marked "provisional" in the source
//! and listed in oracle/README.md.

#![allow(dead_code)]

mod ast;
mod deci;
mod interp;
mod lexer;
mod parser;
mod value;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;

/// Parse one file as the module `module` (empty for the root).
fn parse_file(path: &Path, module: Vec<String>) -> Result<ast::Program, String> {
    parse_file_in(path, module, Vec::new())
}

/// Parse one file of the Limb rooted at `limb_root` (`[]` for the Limb being built).
fn parse_file_in(path: &Path, module: Vec<String>, limb_root: Vec<String>) -> Result<ast::Program, String> {
    let src = std::fs::read_to_string(path).map_err(|e| format!("{}: cannot read: {e}", path.display()))?;
    let toks = lexer::lex(&src).map_err(|e| format!("{}: line {}:{}: {}", path.display(), e.line, e.col, e.msg))?;
    parser::Parser::new_in_limb(toks, module, limb_root).program()
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
    let mut prog = load_limb(path, Vec::new())?;
    // the Limbs it depends on, from its `CCA.cplr` (owner, 2026-09-29), each under a root of its own
    let mut loaded: Vec<(String, PathBuf)> = Vec::new();
    load_deps(path, &mut prog, &mut loaded, &mut Vec::new())?;
    Ok(prog)
}

/// One Limb directory, its modules under `root` (`[]` for the Limb being built).
fn load_limb(path: &Path, root: Vec<String>) -> Result<ast::Program, String> {
    let rootfile = path.join("limb.cpls");
    if !rootfile.is_file() { return Err(format!("{}: a Limb directory needs its root file, limb.cpls", path.display())); }
    let mut prog = parse_file_in(&rootfile, root.clone(), root.clone())?;
    let mut files = Vec::new();
    collect(path, &mut Vec::new(), &mut files)?;
    files.sort();
    for (module, file) in files {
        // every directory on the way is a module too (implicit, public — provisional)
        for i in 1..module.len() {
            let dirpath: Vec<String> = root.iter().cloned().chain(module[..i].iter().cloned()).collect();
            if !prog.modules.iter().any(|m| m.path == dirpath) {
                prog.modules.push(ast::ModuleDef { path: dirpath, vis: ast::Vis::Pub, line: 0 });
            }
        }
        let module: Vec<String> = root.iter().cloned().chain(module).collect();
        if prog.modules.iter().any(|m| m.path == module) {
            return Err(format!("{}: module `{}` is also declared as a block elsewhere; a module is a file or a block, not both", file.display(), module.join(".")));
        }
        prog.modules.push(ast::ModuleDef { path: module.clone(), vis: ast::Vis::Pub, line: 0 });
        let part = parse_file_in(&file, module, root.clone())?;
        if part.main.is_some() { return Err(format!("{}: MAIN lives in limb.cpls, the root of the Limb", file.display())); }
        prog.funcs.extend(part.funcs);
        prog.structs.extend(part.structs);
        prog.consts.extend(part.consts);
        prog.externs.extend(part.externs);
        prog.modules.extend(part.modules);
        prog.strict_modules.extend(part.strict_modules);
        prog.enums.extend(part.enums);
        prog.interfaces.extend(part.interfaces);
        prog.claims.extend(part.claims);
    }
    Ok(prog)
}

/// A dependency from `CCA.cplr`: `cca::Dep[name = "geometry", path = "../geometry"]` (or a
/// `version`, from the registry, which the oracle cannot fetch).
struct Dep { name: String, path: Option<String>, version: Option<String> }

/// Read a Limb's `CCA.cplr` (owner, 2026-09-29): one plain `const` per setting, in CyborgPL syntax,
/// the shapes from the built-in `cca` Limb. Returns its `name`, if set, and its dependencies.
fn read_manifest(dir: &Path) -> Result<(Option<String>, Vec<Dep>), String> {
    let file = dir.join("CCA.cplr");
    if !file.is_file() { return Ok((None, Vec::new())); }
    let prog = parse_file(&file, Vec::new())?;
    let shown = file.display();
    if prog.main.is_some() || !prog.funcs.is_empty() || !prog.structs.is_empty() || !prog.enums.is_empty() {
        return Err(format!("{shown}: CCA.cplr holds settings only: one `const` per setting"));
    }
    let text = |b: &ast::Bracket| -> Option<String> { match b.values.as_slice() { [ast::Value::Expr(ast::Expr::Text(t, _))] => Some(t.clone()), _ => None } };
    let (mut name, mut deps) = (None, Vec::new());
    for c in &prog.consts {
        if c.vis != ast::Vis::Private { return Err(format!("{shown}: line {}: `pub` means nothing in CCA.cplr, which no Limb reads: `const '{}' := …;`", c.line, c.name)); }
        match c.name.as_str() {
            "name" => name = Some(text(&c.value).ok_or_else(|| format!("{shown}: line {}: 'name' is a text: `const 'name' := \"shop\";`", c.line))?),
            "version" | "target" => { text(&c.value).ok_or_else(|| format!("{shown}: line {}: '{}' is a text", c.line, c.name))?; }
            "dependencies" => {
                fn each(e: &ast::Expr, out: &mut Vec<ast::Expr>) {
                    match e { ast::Expr::Nested(b) => for v in &b.values { let ast::Value::Expr(x) = v; each(x, out) }, other => out.push(other.clone()) }
                }
                let mut items = Vec::new();
                for v in &c.value.values { let ast::Value::Expr(x) = v; each(x, &mut items); }
                for it in items {
                    let ast::Expr::Construct(shape, fields, line) = it else { return Err(format!("{shown}: line {}: a dependency is `cca::Dep[name = …, path = …]` or `cca::Dep[name = …, version = …]`", c.line)) };
                    if shape != "::cca.Dep" { return Err(format!("{shown}: line {line}: a dependency is a `cca::Dep`, not `{}`", shape.trim_start_matches("::").replacen('.', "::", 1))); }
                    let mut d = Dep { name: String::new(), path: None, version: None };
                    for (f, b) in &fields {
                        let t = text(b).ok_or_else(|| format!("{shown}: line {line}: `{f}` is a text"))?;
                        match f.as_str() { "name" => d.name = t, "path" => d.path = Some(t), "version" => d.version = Some(t),
                            other => return Err(format!("{shown}: line {line}: a `cca::Dep` has no `{other}`; it has `name`, and `path` or `version`")) }
                    }
                    if d.name.is_empty() { return Err(format!("{shown}: line {line}: a dependency needs its `name`")); }
                    if d.path.is_some() == d.version.is_some() { return Err(format!("{shown}: line {line}: `{}` says where it comes from once: a `path` on this machine, or a `version` from the registry", d.name)); }
                    deps.push(d);
                }
            }
            other => return Err(format!("{shown}: line {}: CCA.cplr has no setting '{other}'; its settings are 'name', 'version', 'dependencies' and 'target'", c.line)),
        }
    }
    Ok((name, deps))
}

/// Load every Limb `dir` depends on, and theirs, each once, under `::name`.
fn load_deps(dir: &Path, prog: &mut ast::Program, loaded: &mut Vec<(String, PathBuf)>, chain: &mut Vec<PathBuf>) -> Result<(), String> {
    let (_, deps) = read_manifest(dir)?;
    let here = dir.canonicalize().map_err(|e| format!("{}: {e}", dir.display()))?;
    chain.push(here);
    for d in deps {
        let Some(rel) = &d.path else {
            return Err(format!("{}: `{}` comes from the registry (`version`); the oracle takes `path` dependencies only", dir.join("CCA.cplr").display(), d.name));
        };
        let depdir = dir.join(rel);
        let canon = depdir.canonicalize().map_err(|_| format!("{}: `{}` is at `{rel}`, and there is no directory there", dir.join("CCA.cplr").display(), d.name))?;
        if chain.contains(&canon) { return Err(format!("{}: `{}` depends, through its dependencies, on itself", dir.join("CCA.cplr").display(), d.name)); }
        if let Some((_, p)) = loaded.iter().find(|(n, _)| *n == d.name) {
            if *p != canon { return Err(format!("{}: two different Limbs are called `{}` ({} and {})", dir.join("CCA.cplr").display(), d.name, p.display(), canon.display())); }
            continue;
        }
        let (own, _) = read_manifest(&depdir)?;
        if let Some(own) = own { if own != d.name { return Err(format!("{}: the Limb at `{rel}` calls itself `{own}`, not `{}`", dir.join("CCA.cplr").display(), d.name)); } }
        loaded.push((d.name.clone(), canon.clone()));
        let root = vec![format!("::{}", d.name)];
        let mut dep = load_limb(&depdir, root.clone())?;
        // a dependency's MAIN never runs: only the Limb being built has one (the oracle's reading)
        dep.main = None;
        prog.modules.push(ast::ModuleDef { path: root, vis: ast::Vis::Pub, line: 0 });
        prog.funcs.extend(dep.funcs); prog.structs.extend(dep.structs); prog.enums.extend(dep.enums);
        prog.interfaces.extend(dep.interfaces); prog.claims.extend(dep.claims); prog.consts.extend(dep.consts);
        prog.externs.extend(dep.externs); prog.modules.extend(dep.modules); prog.strict_modules.extend(dep.strict_modules);
        load_deps(&depdir, prog, loaded, chain)?;
    }
    chain.pop();
    Ok(())
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
    let mut out = out;
    for _ in 0..parser::AMPERSANDS.swap(0, std::sync::atomic::Ordering::Relaxed) {
        // The build's remark on the `&` spelling: once per `&`, on stdout, before the program runs (owner, 2026-09-21).
        writeln!(out, "What are you? Rust programmer or Claude?").map_err(|e| e.to_string())?;
    }
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
    };
}
