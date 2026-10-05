//! `oracle file.cpls` — run a CyborgPL program the slow, obvious way.
//!
//! This is the reference interpreter: the executable specification. It knows
//! exactly what `design/` has decided and refuses everything else with a
//! message that says so. Where the design leaves a question open and the
//! interpreter has to pick, the pick is marked "provisional" in the source
//! and listed in oracle/README.md.

#![allow(dead_code)]

mod ast;
mod bigint;
mod deci;
mod interp;
mod lexer;
mod parser;
mod value;

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

/// A dependency from `CCA.cplr`: `cca::Dep[path = "../geometry"]` (or an `id` and `version`, from
/// the registry, which the oracle cannot fetch). It names no word: code writes the Limb's own 'prefix'.
struct Dep { what: String, path: Option<String>, version: Option<String>, id: Option<String> }

/// Read a Limb's `CCA.cplr` (owner, 2026-09-29): one plain `const` per setting, in CyborgPL syntax,
/// the shapes from the built-in `cca` Limb. Returns its 'prefix' (none when there is no `CCA.cplr`) and its dependencies.
fn read_manifest(dir: &Path) -> Result<(Option<String>, Vec<Dep>), String> {
    let file = dir.join("CCA.cplr");
    if !file.is_file() { return Ok((None, Vec::new())); }
    let prog = parse_file(&file, Vec::new())?;
    let shown = file.display();
    if prog.main.is_some() || !prog.funcs.is_empty() || !prog.structs.is_empty() || !prog.enums.is_empty() {
        return Err(format!("{shown}: CCA.cplr holds settings only: one `const` per setting"));
    }
    let text = |b: &ast::Bracket| -> Option<String> { match b.values.as_slice() { [ast::Value::Expr(ast::Expr::Text(t, _))] => Some(t.clone()), _ => None } };
    let (mut name, mut prefix, mut deps) = (false, None, Vec::new());
    let (mut has_version, mut targets) = (false, 0);
    for c in &prog.consts {
        if c.vis != ast::Vis::Private { return Err(format!("{shown}: line {}: `pub` means nothing in CCA.cplr, which no Limb reads: `const '{}' := …;`", c.line, c.name)); }
        match c.name.as_str() {
            // a label for people; code writes the 'prefix' (owner, 2026-09-29: "Infront and name are different things")
            // strict: the module-name rule (owner, 2026-09-30: "strict"), though code writes the prefix
            "name" => {
                let n = text(&c.value).ok_or_else(|| format!("{shown}: line {}: 'name' is a text: `const 'name' := \"shop\";`", c.line))?;
                parser::module_name_ok(&n).map_err(|m| format!("{shown}: line {}: `{n}` cannot name a Limb: {m}", c.line))?;
                name = true;
            }
            // the word code writes in front of `::` to reach this Limb, its maker's pick (owner, 2026-09-29: b, "required")
            "prefix" => {
                let p = text(&c.value).ok_or_else(|| format!("{shown}: line {}: 'prefix' is a text: `const 'prefix' := \"shop\";`", c.line))?;
                parser::module_name_ok(&p).map_err(|m| format!("{shown}: line {}: `{p}` cannot be a prefix, written in code as `{p}::`: {m}", c.line))?;
                prefix = Some(p);
            }
            "version" => {
                let v = text(&c.value).ok_or_else(|| format!("{shown}: line {}: 'version' is a text", c.line))?;
                // any text, at most 256 bytes, as in Synthia.ttsr (owner, 2026-09-29: "256 bytes it is"; for CCA.cplr: "same")
                if v.len() > 256 { return Err(format!("{shown}: line {}: 'version' is at most 256 bytes, and this one is {}", c.line, v.len())); }
                has_version = true;
            }
            "target" => return Err(format!("{shown}: line {}: a Limb builds for several targets, so the setting is 'targets': `const 'targets' := [cca::Target[name = \"aarch64-macos\"],];`", c.line)),
            // several targets, each with its settings (owner, 2026-09-29): CCA's package manager can do
            // whatever Synthia can, in its plainer way
            "targets" => {
                fn each(e: &ast::Expr, out: &mut Vec<ast::Expr>) {
                    match e { ast::Expr::Nested(b) => for v in &b.values { let ast::Value::Expr(x) = v; each(x, out) }, other => out.push(other.clone()) }
                }
                let mut items = Vec::new();
                for v in &c.value.values { let ast::Value::Expr(x) = v; each(x, &mut items); }
                let count = items.len();
                targets += count;
                let mut defaults = 0;
                let mut names = Vec::new();
                for it in items {
                    let ast::Expr::Construct(shape, fields, line) = it else { return Err(format!("{shown}: line {}: a target is `cca::Target[name = …]`, with a `runner` if it has one", c.line)) };
                    if shape != "::cca.Target" { return Err(format!("{shown}: line {line}: a target is a `cca::Target`")); }
                    if !fields.iter().any(|(f, _)| f == "name") { return Err(format!("{shown}: line {line}: a target needs its `name`")); }
                    for (f, b) in &fields {
                        // the one built when none is named (owner, 2026-09-29: b)
                        if f == "default" {
                            match b.values.as_slice() { [ast::Value::Expr(ast::Expr::Bool(true, _))] => defaults += 1, [ast::Value::Expr(ast::Expr::Bool(false, _))] => {},
                                _ => return Err(format!("{shown}: line {line}: `default` is `true` or `false`")) }
                            continue;
                        }
                        if f != "name" && f != "runner" { return Err(format!("{shown}: line {line}: a `cca::Target` has no `{f}`; it has `name`, `runner` and `default`")); }
                        let t = text(b).ok_or_else(|| format!("{shown}: line {line}: `{f}` is a text"))?;
                        if f == "name" { names.push((t, line)); }
                    }
                }
                if count > 1 && defaults != 1 { return Err(format!("{shown}: line {}: with several targets, exactly one says `default = true` — the one built when none is named ({defaults} do)", c.line)); }
                // only a target CCA can build (owner, 2026-09-30: "refuse", as Synthia.ttsr's "Only what's
                // actually doable"); the one list is CCA's
                let doable: Vec<&str> = include_str!("../../cca/targets.txt").lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).collect();
                for (t, line) in names {
                    if !doable.contains(&t.as_str()) { return Err(format!("{shown}: line {line}: `{t}` is not a target CCA can build; it builds for {}", doable.iter().map(|d| format!("`{d}`")).collect::<Vec<_>>().join(", "))); }
                }
            }
            "dependencies" => {
                fn each(e: &ast::Expr, out: &mut Vec<ast::Expr>) {
                    match e { ast::Expr::Nested(b) => for v in &b.values { let ast::Value::Expr(x) = v; each(x, out) }, other => out.push(other.clone()) }
                }
                let mut items = Vec::new();
                for v in &c.value.values { let ast::Value::Expr(x) = v; each(x, &mut items); }
                for it in items {
                    let ast::Expr::Construct(shape, fields, line) = it else { return Err(format!("{shown}: line {}: a dependency is `cca::Dep[path = …]` or `cca::Dep[id = …, version = …]`", c.line)) };
                    if shape != "::cca.Dep" { return Err(format!("{shown}: line {line}: a dependency is a `cca::Dep`, not `{}`", shape.trim_start_matches("::").replacen('.', "::", 1))); }
                    let mut d = Dep { what: String::new(), path: None, version: None, id: None };
                    for (f, b) in &fields {
                        // two Limbs with one prefix are an error, fixed by editing a Limb's source (owner, 2026-09-29:
                        // "Just edit the source code"), so a dependency names no word
                        if f == "name" || f == "prefix" { return Err(format!("{shown}: line {line}: a dependency names no word: code writes the Limb's own 'prefix', from its `CCA.cplr`")); }
                        let t = text(b).ok_or_else(|| format!("{shown}: line {line}: `{f}` is a text"))?;
                        match f.as_str() { "path" => d.path = Some(t), "version" => d.version = Some(t), "id" => d.id = Some(t),
                            other => return Err(format!("{shown}: line {line}: a `cca::Dep` has no `{other}`; it has `path`, or `id` and `version`")) }
                    }
                    d.what = match (&d.path, &d.id) { (Some(p), _) => format!("the Limb at `{p}`"), (None, Some(i)) => format!("the Limb `{}…`", &i[..i.len().min(8)]), _ => "the dependency".into() };
                    if d.path.is_some() == d.version.is_some() { return Err(format!("{shown}: line {line}: a dependency says where it comes from once: a `path` on this machine, or an `id` and `version` from the registry")); }
                    // the registry knows a Limb by its ID, never its name (owner, 2026-09-29): names may repeat
                    // an ID is 64 hex digits wherever it is written (the owner's definition; compiler claude, 2026-09-29)
                    if let Some(i) = &d.id {
                        if i.len() != 64 || !i.bytes().all(|b| b.is_ascii_hexdigit()) { return Err(format!("{shown}: line {line}: an id is 64 hex digits, 256 random bits")); }
                    }
                    if d.version.is_some() && d.id.is_none() { return Err(format!("{shown}: line {line}: a `version` comes from the registry, which knows a Limb by its ID: `id = \"…\"`, 64 hex digits")); }
                    deps.push(d);
                }
            }
            other => return Err(format!("{shown}: line {}: CCA.cplr has no setting '{other}'; its settings are 'name', 'prefix', 'version', 'dependencies' and 'targets'", c.line)),
        }
    }
    // a build needs the Limb's name, version and a target; only publishing needs more (owner, 2026-09-29:
    // "yea", matching Synthia.ttsr)
    let why = "a build needs the Limb's 'name', 'prefix', 'version' and at least one target";
    if !name { return Err(format!("{shown}: CCA.cplr has no 'name': {why}: `const 'name' := \"shop\";`")); }
    if prefix.is_none() { return Err(format!("{shown}: CCA.cplr has no 'prefix', the word code writes in front of `::` to reach this Limb: {why}: `const 'prefix' := \"shop\";`")); }
    if !has_version { return Err(format!("{shown}: CCA.cplr has no 'version': {why}: `const 'version' := \"1.0.0\";`")); }
    if targets == 0 { return Err(format!("{shown}: CCA.cplr has no target: {why}: `const 'targets' := [cca::Target[name = \"aarch64-macos\"],];`")); }
    Ok((prefix, deps))
}

/// Load every Limb `dir` depends on, and theirs, each once, under `::prefix`.
fn load_deps(dir: &Path, prog: &mut ast::Program, loaded: &mut Vec<(String, PathBuf)>, chain: &mut Vec<PathBuf>) -> Result<(), String> {
    let (_, deps) = read_manifest(dir)?;
    let here = dir.canonicalize().map_err(|e| format!("{}: {e}", dir.display()))?;
    chain.push(here);
    for d in deps {
        let here_file = dir.join("CCA.cplr");
        let Some(rel) = &d.path else {
            return Err(format!("{}: {} comes from the registry; the oracle takes `path` dependencies only", here_file.display(), d.what));
        };
        let depdir = dir.join(rel);
        let canon = depdir.canonicalize().map_err(|_| format!("{}: a dependency is at `{rel}`, and there is no directory there", here_file.display()))?;
        if chain.contains(&canon) { return Err(format!("{}: {} depends, through its dependencies, on itself", here_file.display(), d.what)); }
        if !depdir.join("CCA.cplr").is_file() { return Err(format!("{}: {} has no `CCA.cplr`, so no 'prefix' for code to reach it by", here_file.display(), d.what)); }
        let (Some(prefix), _) = read_manifest(&depdir)? else { unreachable!() };
        // a Limb may pick `std` or `cca`; it just clashes with the built-in one (owner, 2026-09-29)
        if prefix == "std" || prefix == "cca" { return Err(format!("{}: {} has the prefix `{prefix}`, which is the built-in `{prefix}::`; edit its 'prefix'", here_file.display(), d.what)); }
        if let Some((_, p)) = loaded.iter().find(|(n, _)| *n == prefix) {
            if *p != canon { return Err(format!("{}: two Limbs have the prefix `{prefix}` ({} and {}); edit one Limb's 'prefix'", here_file.display(), p.display(), canon.display())); }
            continue;
        }
        loaded.push((prefix.clone(), canon.clone()));
        let root = vec![format!("::{prefix}")];
        let mut dep = load_limb(&depdir, root.clone())?;
        // a dependency's MAIN never runs: only the Limb being built has one (the oracle's reading)
        dep.main = None;
        dep.main_args = None;
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
    run_path_as(path, out, false)
}

/// `tests`: run every `test fn` instead of MAIN (`cca test`, owner 2026-09-29).
pub fn run_path_as(path: &Path, out: Box<dyn std::io::Write>, tests: bool) -> Result<u8, String> {
    run_with_args(path, out, tests, Vec::new())
}

/// `args`: the words after `--`, which MAIN reads as C's argc/argv (owner, 2026-09-29).
pub fn run_with_args(path: &Path, out: Box<dyn std::io::Write>, tests: bool, args: Vec<String>) -> Result<u8, String> {
    let prog = load(path)?;
    let mut interp = interp::Interp::new(&prog, out);
    let mut argv = vec![path.display().to_string()];
    argv.extend(args);
    interp.set_args(argv);
    let r = if tests { interp.run_tests() } else { interp.run() };
    r.map_err(|e| { let msg = op_words(&e.msg); if e.line == 0 { format!("{}: {}", path.display(), msg) } else { format!("{}: line {}: {}", path.display(), e.line, msg) } })
}

/// The interpreter names operators by the symbols it works with inside (`+`, `x~`); a message
/// names them by their words (owner, 2026-10-03: the operators are words; check claude's note).
fn op_words(msg: &str) -> String {
    let mut m = msg.to_string();
    for (sym, word) in [("`xx~`", "`pow.wrap`"), ("`x~`", "`times.wrap`"), ("`+~`", "`plus.wrap`"), ("`-~`", "`minus.wrap`"), ("`xx`", "`pow`"), ("`x`", "`times`"), ("`+`", "`plus`"), ("`-`", "`minus`")] {
        m = m.replace(sym, word);
    }
    m
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // `oracle [--test] <program> [-- words for the program's argv]`
    let (head, rest) = match args.iter().position(|a| a == "--") { Some(i) => (&args[..i], args[i + 1..].to_vec()), None => (&args[..], Vec::new()) };
    let (tests, target) = match head {
        [_, t] => (false, t),
        [_, flag, t] if flag == "--test" => (true, t),
        _ => { eprintln!("usage: oracle [--test] <file.cpls | limb-directory> [-- program arguments]"); exit(2); }
    };
    match run_with_args(Path::new(target), Box::new(std::io::stdout()), tests, rest) {
        Ok(code) => exit(code as i32),
        Err(msg) => { eprintln!("{msg}"); exit(1); }
    };
}
