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

thread_local! {
    /// The first pass over a program's files reads only their `precedence` blocks (owner, 2026-10-03:
    /// a Limb's levels apply wherever its operators are used)
    static LEVELS_ONLY: std::cell::Cell<bool> = std::cell::Cell::new(false);
}

/// Parse one file of the Limb rooted at `limb_root` (`[]` for the Limb being built).
fn parse_file_in(path: &Path, module: Vec<String>, limb_root: Vec<String>) -> Result<ast::Program, String> {
    let src = std::fs::read_to_string(path).map_err(|e| format!("{}: cannot read: {e}", path.display()))?;
    let toks = lexer::lex(&src).map_err(|e| format!("{}: line {}:{}: {}", path.display(), e.line, e.col, e.msg))?;
    if LEVELS_ONLY.with(|f| f.get()) && path.file_name().map_or(true, |n| n != "CCA.cplr") {
        parser::Parser::new_in_limb(toks, module, limb_root).scan_levels()
            .map_err(|e| format!("{}: line {}: {}", path.display(), e.line, e.msg))?;
        return Ok(ast::Program { funcs: Vec::new(), structs: Vec::new(), enums: Vec::new(), interfaces: Vec::new(), claims: Vec::new(), consts: Vec::new(), externs: Vec::new(), modules: Vec::new(), main: None, main_args: None, strict_modules: Vec::new(), bare_marks: Vec::new() });
    }
    parser::Parser::new_in_limb(toks, module, limb_root).program()
        .map_err(|e| if e.col == 0 { format!("{}: line {}: {}", path.display(), e.line, e.msg) } else { format!("{}: line {}:{}: {}", path.display(), e.line, e.col, e.msg) })
}

/// A Limb is a directory: `limb.cpls` is the root; every other `.cpls` file is
/// a module named after it, directories nesting (decided 2026-09-19). A lone
/// `.cpls` file is a Limb of one root file.
fn load(path: &Path) -> Result<ast::Program, String> {
    // first every file's `precedence` blocks, then the program with all of them known
    parser::SHARED_LEVELS.with(|l| l.borrow_mut().clear());
    parser::LACKS.with(|n| n.borrow_mut().clear());
    LEVELS_ONLY.with(|f| f.set(true));
    let first = load_files(path);
    LEVELS_ONLY.with(|f| f.set(false));
    first?;
    // then the program once to learn which of other Limbs' structs and enums may be written bare
    // (owner, 2026-10-06), and again with that known
    parser::BARE_TYPES.with(|t| t.borrow_mut().clear());
    let once = load_files(path)?;
    let marks = check_bare(&once).unwrap_or_default();
    let mut bare_types = Vec::new();
    for (key, mode) in &marks {
        let Some((kind, path)) = key.split_once(':') else { continue };
        if !["struct", "enum", "interface"].contains(&kind) || *mode == ast::Bare::Never { continue; }
        let Some(rest) = path.strip_prefix("::") else { continue };
        let (root, rel) = rest.split_once('.').unwrap_or((rest, ""));
        bare_types.push((rel.split('.').map(str::to_string).collect::<Vec<_>>(), path.to_string(), format!("::{root}"), *mode == ast::Bare::Only));
    }
    parser::BARE_TYPES.with(|t| *t.borrow_mut() = bare_types);
    parser::BARE_TYPE_USES.with(|u| u.borrow_mut().clear());
    parser::BARE_CALLS.with(|b| b.borrow_mut().clear());
    LIMB_POWERS.with(|l| l.borrow_mut().clear());
    parser::POWER_USES.with(|u| u.borrow_mut().clear());
    parser::CALL_SITES.with(|u| u.borrow_mut().clear());
    parser::VAR_SITES.with(|u| u.borrow_mut().clear());
    let prog = load_files(path)?;
    check_powers(&prog).map_err(|m| format!("{}: {m}", path.display()))?;
    // a bare call with more than one candidate is refused where it is written, run or not (owner,
    // 2026-10-06: "`a`, and no"); the interpreter makes the same check when a call runs
    for (from, written, std, line) in parser::BARE_CALLS.with(|b| b.borrow().clone()) {
        let shown = written.join(".");
        let own_module: Vec<String> = from.iter().cloned().chain(written[..written.len() - 1].iter().cloned()).collect();
        let own = prog.funcs.iter().any(|f| f.module == own_module && Some(&f.name) == written.last() && !f.is_op);
        let mine = from.first().filter(|r| r.starts_with("::")).cloned();
        let mut who: Vec<String> = Vec::new();
        if own { who.push(format!("`my.{}` (this Limb's)", own_module.iter().filter(|m| !m.starts_with("::")).chain(written.last()).cloned().collect::<Vec<_>>().join("."))); }
        if std { who.push(format!("`std::{shown}`")); }
        for f in &prog.funcs {
            let Some(root) = f.module.first().filter(|r| r.starts_with("::")) else { continue };
            if Some(root) == mine.as_ref() || f.vis != ast::Vis::Pub || f.is_op { continue; }
            let rel: Vec<&str> = f.module[1..].iter().map(String::as_str).chain(std::iter::once(f.name.as_str())).collect();
            if rel != written.iter().map(String::as_str).collect::<Vec<_>>() { continue; }
            if matches!(marks.get(&interp::bare_key("fn", &f.module, &f.name)), Some(ast::Bare::Optional | ast::Bare::Only)) { who.push(format!("`{}::{shown}`", root.trim_start_matches("::"))); }
        }
        if who.len() > 1 { return Err(format!("{}: line {line}: `{shown}` could be any of {}: write the path", path.display(), who.join(", "))); }
    }
    // this Limb's own struct or enum does not win over another Limb's bare one (owner, 2026-10-06: "no")
    for (own, theirs, written, root) in parser::BARE_TYPE_USES.with(|u| u.borrow().clone()) {
        if prog.structs.iter().any(|d| d.name == own) || prog.enums.iter().any(|d| d.name == own) || prog.interfaces.iter().any(|d| d.name == own) {
            let _ = theirs;
            return Err(format!("{}: `{written}` could be any of `my.{written}` (this Limb's), `{}::{written}`: write the path", path.display(), root.trim_start_matches("::")));
        }
    }
    Ok(prog)
}

/// A Limb whose `CCA.cplr` does not list std or a core reaches none of it (owner, 2026-10-05/06). One with no
/// `CCA.cplr` gets what the machine's package installed, and the oracle stands in for the full package.
/// A power as a manifest names it (owner, 2026-10-07): its path, the things named in `[ ]`, and for a
/// `deny` its `.except[ … ]` and `.for[ … ]`.
#[derive(Clone, Debug, Default)]
struct PowerRef { path: Vec<String>, things: Vec<String>, except: Vec<String>, for_: Vec<String> }

/// Each Limb's manifest, as the power check needs it: its root, prefix, cores, listed powers and denies.
struct LimbPowers { root: Vec<String>, prefix: String, cores: Vec<String>, powers: Vec<PowerRef>, deny: Vec<PowerRef> }

thread_local! {
    /// the `powers` and `deny` of the manifest just read
    static MANIFEST_POWERS: std::cell::RefCell<Option<(Vec<PowerRef>, Vec<PowerRef>)>> = std::cell::RefCell::new(None);
    static LIMB_POWERS: std::cell::RefCell<Vec<LimbPowers>> = std::cell::RefCell::new(Vec::new());
}

/// `c_calls.functions["write"].except["x"]`, as the parser turned it into text
fn parse_power(text: &str) -> Result<PowerRef, String> {
    let mut p = PowerRef::default();
    let mut rest = text;
    let mut last: Option<String> = None;
    while !rest.is_empty() {
        if let Some(r) = rest.strip_prefix('[') {
            let end = r.find(']').ok_or("an unclosed `[`")?;
            let things: Vec<String> = r[..end].split(',').filter(|t| !t.is_empty()).map(|t| t.trim_matches('"').to_string()).collect();
            match last.as_deref() { Some("except") => p.except = things, Some("for") => p.for_ = things, _ => p.things = things }
            rest = &r[end + 1..];
            continue;
        }
        let r = rest.strip_prefix('.').unwrap_or(rest);
        let end = r.find(['.', '[']).unwrap_or(r.len());
        let word = &r[..end];
        if word == "except" || word == "for" { last = Some(word.to_string()); } else { p.path.push(word.to_string()); last = None; }
        rest = &r[end..];
    }
    if p.path.is_empty() { return Err("a power names its path: `cca::Power.c_calls`".into()); }
    Ok(p)
}

fn mark_lacks(dir: &Path, root: &[String]) -> Result<(), String> {
    if !dir.join("CCA.cplr").is_file() { return Ok(()); }
    let (prefix, deps, cores) = read_manifest(dir)?;
    let (powers, deny) = MANIFEST_POWERS.with(|m| m.borrow_mut().take()).unwrap_or_default();
    LIMB_POWERS.with(|l| l.borrow_mut().push(LimbPowers { root: root.to_vec(), prefix: prefix.unwrap_or_default(), cores: cores.clone(), powers, deny }));
    let mut lacks: Vec<String> = OFFICIAL_CORES.iter().filter(|c| !cores.iter().any(|x| x == *c)).map(|c| c.to_string()).collect();
    if !deps.iter().any(Dep::is_std) { lacks.push("std".into()); }
    parser::LACKS.with(|n| n.borrow_mut().push((root.to_vec(), lacks)));
    Ok(())
}

fn load_files(path: &Path) -> Result<ast::Program, String> {
    if path.is_file() {
        if path.extension().map(|x| x != "cpls").unwrap_or(true) { return Err("oracle: a CyborgPL source file ends in .cpls".into()); }
        return parse_file(path, Vec::new());
    }
    if !path.is_dir() { return Err(format!("oracle: {} is neither a .cpls file nor a Limb directory", path.display())); }
    mark_lacks(path, &[])?;
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
        prog.bare_marks.extend(part.bare_marks);
        prog.enums.extend(part.enums);
        prog.interfaces.extend(part.interfaces);
        prog.claims.extend(part.claims);
    }
    Ok(prog)
}

/// A dependency from `CCA.cplr`: `cca::Dep[path = "../geometry"]` (or an `id` and `version`, from
/// the registry, which the oracle cannot fetch). It names no word: code writes the Limb's own 'prefix'.
struct Dep { what: String, path: Option<String>, version: Option<String>, id: Option<String> }

impl Dep { fn is_std(&self) -> bool { self.id.as_deref() == Some(parser::STD_ID) } }

/// The official cores (design/cores.md, 2026-10-06). Another core is a Rust plugin, which the oracle
/// cannot load.
const OFFICIAL_CORES: [&str; 10] = ["IEG", "ASP", "Genny", "MCPS", "AutoGPU", "Metal", "Nvidia", "Vulkan", "WSTN", "MTM"];

use parser::CORE_IDS;

/// Read a Limb's `CCA.cplr`: one CyborgPL value, `cca::Manifest[ … ]` (owner, 2026-10-06: "`c`. It may
/// not look as complex, but at least it's not much of a roleplay"), replacing 2026-09-29's one `const`
/// per setting. Returns its 'prefix' (none when there is no `CCA.cplr`), its dependencies and its cores.
fn read_manifest(dir: &Path) -> Result<(Option<String>, Vec<Dep>, Vec<String>), String> {
    let file = dir.join("CCA.cplr");
    if !file.is_file() { return Ok((None, Vec::new(), Vec::new())); }
    let shown = file.display();
    let src = std::fs::read_to_string(&file).map_err(|e| format!("{shown}: cannot read: {e}"))?;
    let toks = lexer::lex(&src).map_err(|e| format!("{shown}: line {}:{}: {}", e.line, e.col, e.msg))?;
    let one = "CCA.cplr is one value: `cca::Manifest[name = \"shop\", prefix = \"shop\", version = \"1.0.0\", targets = [cca::Target[name = \"aarch64-macos\"],]];`";
    if matches!(toks.first().map(|t| &t.tok), Some(lexer::Tok::Word(w)) if w == "const" || w == "pub") {
        return Err(format!("{shown}: line {}: settings are no longer `const`s (2026-10-06): {one}", toks[0].line));
    }
    let (fields, mline) = parser::Parser::new(toks).manifest()
        .map_err(|e| format!("{shown}: line {}: {}", e.line, e.msg))?
        .ok_or_else(|| format!("{shown}: {one}"))?;
    let text = |b: &ast::Bracket| -> Option<String> { match b.values.as_slice() { [ast::Value::Expr(ast::Expr::Text(t, _))] => Some(t.clone()), _ => None } };
    fn each(e: &ast::Expr, out: &mut Vec<ast::Expr>) {
        match e { ast::Expr::Nested(b) => for v in &b.values { let ast::Value::Expr(x) = v; each(x, out) }, other => out.push(other.clone()) }
    }
    let items = |b: &ast::Bracket| { let mut v = Vec::new(); for x in &b.values { let ast::Value::Expr(x) = x; each(x, &mut v); } v };
    let (mut name, mut prefix, mut deps, mut cores) = (false, None, Vec::new(), Vec::new());
    let (mut powers_seen, mut deny_seen): (Vec<PowerRef>, Vec<PowerRef>) = (Vec::new(), Vec::new());
    let (mut has_version, mut targets) = (false, 0);
    let mut seen: Vec<&str> = Vec::new();
    for (f, value) in &fields {
        let line = value.line;
        if seen.contains(&f.as_str()) { return Err(format!("{shown}: line {line}: `{f}` is given twice")); }
        seen.push(f);
        match f.as_str() {
            // a label for people; code writes the 'prefix' (owner, 2026-09-29: "Infront and name are different things")
            // strict: the module-name rule (owner, 2026-09-30: "strict"), though code writes the prefix
            "name" => {
                let n = text(value).ok_or_else(|| format!("{shown}: line {line}: `name` is a text: `name = \"shop\"`"))?;
                parser::module_name_ok(&n).map_err(|m| format!("{shown}: line {line}: `{n}` cannot name a Limb: {m}"))?;
                name = true;
            }
            // the word code writes in front of `::` to reach this Limb, its maker's pick (owner, 2026-09-29: b, "required")
            "prefix" => {
                let p = text(value).ok_or_else(|| format!("{shown}: line {line}: `prefix` is a text: `prefix = \"shop\"`"))?;
                parser::module_name_ok(&p).map_err(|m| format!("{shown}: line {line}: `{p}` cannot be a prefix, written in code as `{p}::`: {m}"))?;
                prefix = Some(p);
            }
            "version" => {
                let v = text(value).ok_or_else(|| format!("{shown}: line {line}: `version` is a text"))?;
                // any text, at most 256 bytes, as in Synthia.ttsr (owner, 2026-09-29: "256 bytes it is"; for CCA.cplr: "same")
                if v.len() > 256 { return Err(format!("{shown}: line {line}: `version` is at most 256 bytes, and this one is {}", v.len())); }
                has_version = true;
            }
            "target" => return Err(format!("{shown}: line {line}: a Limb builds for several targets, so the field is `targets`: `targets = [cca::Target[name = \"aarch64-macos\"],]`")),
            // several targets, each with its settings (owner, 2026-09-29): CCA's package manager can do
            // whatever Synthia can, in its plainer way
            "targets" => {
                let items = items(value);
                let count = items.len();
                targets += count;
                let mut defaults = 0;
                let mut names = Vec::new();
                for it in items {
                    let ast::Expr::Construct(shape, fields, line) = it else { return Err(format!("{shown}: line {line}: a target is `cca::Target[name = …]`, with a `runner` if it has one")) };
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
                if count > 1 && defaults != 1 { return Err(format!("{shown}: line {line}: with several targets, exactly one says `default = true` — the one built when none is named ({defaults} do)")); }
                // only a target CCA can build (owner, 2026-09-30: "refuse", as Synthia.ttsr's "Only what's
                // actually doable"); the one list is CCA's
                let doable: Vec<&str> = include_str!("../../cca/targets.txt").lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).collect();
                for (t, line) in names {
                    if !doable.contains(&t.as_str()) { return Err(format!("{shown}: line {line}: `{t}` is not a target CCA can build; it builds for {}", doable.iter().map(|d| format!("`{d}`")).collect::<Vec<_>>().join(", "))); }
                }
            }
            "dependencies" => {
                for it in items(value) {
                    let ast::Expr::Construct(shape, fields, line) = it else { return Err(format!("{shown}: line {line}: a dependency is `cca::Dep[path = …]` or `cca::Dep[id = …, version = …]`")) };
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
                    // std is listed like any limb, by its ID and version; CCA uses the installed copy when they
                    // match (owner, 2026-10-06), and the oracle stands in for that copy
                    if d.is_std() {
                        if deps.iter().any(|x: &Dep| x.is_std()) { return Err(format!("{shown}: line {line}: std is listed twice")); }
                        if d.version.as_deref() != Some(parser::STD_VERSION) { return Err(format!("{shown}: line {line}: the oracle stands in for std `{}` only, the installed one; this asks for `{}`", parser::STD_VERSION, d.version.as_deref().unwrap_or(""))); }
                        d.what = "std".into();
                    }
                    deps.push(d);
                }
            }
            // the cores a Limb is built for (owner, 2026-10-05: "Limbs should say what cores they are built
            // for"), each by its registry ID and version, or a `path` for one never registered (2026-10-06)
            "cores" => {
                for it in items(value) {
                    let ast::Expr::Construct(shape, fields, line) = it else { return Err(format!("{shown}: line {line}: a core is `cca::Core[id = \"…\", version = \"…\"]`")) };
                    if shape != "::cca.Core" { return Err(format!("{shown}: line {line}: a core is a `cca::Core`, not `{}`", shape.trim_start_matches("::").replacen('.', "::", 1))); }
                    let (mut id, mut version, mut cpath) = (None, None, None);
                    for (f, b) in &fields {
                        let t = text(b).ok_or_else(|| format!("{shown}: line {line}: a core's `{f}` is a text"))?;
                        match f.as_str() {
                            "id" => id = Some(t), "version" => version = Some(t), "path" => cpath = Some(t),
                            "name" => return Err(format!("{shown}: line {line}: a core is listed by its registry ID since 2026-10-06 (names repeat, IDs do not): `cca::Core[id = \"…\", version = \"…\"]`{}", CORE_IDS.iter().find(|(n, _)| *n == t).map(|(n, i)| format!("; {n}'s is `{i}`")).unwrap_or_default())),
                            other => return Err(format!("{shown}: line {line}: a `cca::Core` has no `{other}`; it has `id` and `version`, or a `path`")),
                        }
                    }
                    if cpath.is_some() {
                        if id.is_some() || version.is_some() { return Err(format!("{shown}: line {line}: a core says where it comes from once: an `id` and `version` from the registry, or a `path` on this machine")); }
                        return Err(format!("{shown}: line {line}: the core at `{}` is not registered, so not verified; loading another core is not built yet (a Rust plugin, which the oracle cannot load)", cpath.unwrap()));
                    }
                    let id = id.ok_or_else(|| format!("{shown}: line {line}: a core needs its `id` (64 hex digits) and `version`"))?;
                    if id.len() != 64 || !id.bytes().all(|b| b.is_ascii_hexdigit()) { return Err(format!("{shown}: line {line}: an id is 64 hex digits, 256 random bits")); }
                    let version = version.ok_or_else(|| format!("{shown}: line {line}: a core needs its `version`, as a registry limb does"))?;
                    let Some((core, _)) = CORE_IDS.iter().find(|(_, i)| *i == id) else {
                        return Err(format!("{shown}: line {line}: the core `{}…` is not an official one; loading another core is not built yet (a Rust plugin, which the oracle cannot load)", &id[..8]));
                    };
                    let want = parser::core_version(core);
                    if version != want { return Err(format!("{shown}: line {line}: the oracle stands in for {core} `{want}` only, the installed one; this asks for `{version}`")); }
                    let core = core.to_string();
                    if cores.contains(&core) { return Err(format!("{shown}: line {line}: the core {core} is listed twice")); }
                    cores.push(core);
                }
            }
            // the powers a Limb uses, and what the program takes away (owner, 2026-10-07)
            "powers" | "deny" => {
                let mut list = Vec::new();
                for it in items(value) {
                    let ast::Expr::Text(t, _) = &it else { return Err(format!("{shown}: line {line}: a power is `cca::Power.c_calls`, narrowed with `.` and naming things in `[ ]`")) };
                    let Some(text) = t.strip_prefix("power:") else { return Err(format!("{shown}: line {line}: a power is `cca::Power.c_calls`, not a text")) };
                    let pw = parse_power(text).map_err(|m| format!("{shown}: line {line}: {m}"))?;
                    if f == "powers" && (!pw.except.is_empty() || !pw.for_.is_empty()) { return Err(format!("{shown}: line {line}: `.except` and `.for` belong to `deny`; `powers` lists what this Limb uses")); }
                    list.push(pw);
                }
                if f == "powers" { powers_seen = list; } else { deny_seen = list; }
            }
            other => return Err(format!("{shown}: line {line}: a `cca::Manifest` has no field `{other}`; its fields are `name`, `prefix`, `version`, `targets`, `dependencies`, `cores`, `powers` and `deny`")),
        }
    }
    // a build needs the Limb's name, version and a target; only publishing needs more (owner, 2026-09-29:
    // "yea", matching Synthia.ttsr)
    let why = "a build needs the Limb's `name`, `prefix`, `version` and at least one target";
    if !name { return Err(format!("{shown}: line {mline}: the manifest has no `name`: {why}: `name = \"shop\"`")); }
    if prefix.is_none() { return Err(format!("{shown}: line {mline}: the manifest has no `prefix`, the word code writes in front of `::` to reach this Limb: {why}: `prefix = \"shop\"`")); }
    if !has_version { return Err(format!("{shown}: line {mline}: the manifest has no `version`: {why}: `version = \"1.0.0\"`")); }
    if targets == 0 { return Err(format!("{shown}: line {mline}: the manifest has no target: {why}: `targets = [cca::Target[name = \"aarch64-macos\"],]`")); }
    // std needs ASP (owner, 2026-10-06: "You genuinely have to be shitting on my ass, if you installed
    // std without fucking ASP")
    if deps.iter().any(Dep::is_std) && !cores.iter().any(|c| c == "ASP") {
        return Err(format!("{shown}: line {mline}: std needs the core ASP, so its maths has true values and checks: add `{}` to `cores`", parser::core_line("ASP")));
    }
    // std needs IEG too: its operators on built-in types are a power IEG grants (owner, 2026-10-06: "`a`")
    if deps.iter().any(Dep::is_std) && !cores.iter().any(|c| c == "IEG") {
        return Err(format!("{shown}: line {mline}: std needs the core IEG, which grants it the operators on built-in types: add `{}` to `cores`", parser::core_line("IEG")));
    }
    // cores depend on cores (owner, 2026-10-05); MCPS "Needs IEG, not std" (owner, 2026-10-06)
    if cores.iter().any(|c| c == "MCPS") && !cores.iter().any(|c| c == "IEG") {
        return Err(format!("{shown}: line {mline}: the core MCPS needs IEG: add `{}` to `cores`", parser::core_line("IEG")));
    }
    MANIFEST_POWERS.with(|m| *m.borrow_mut() = Some((powers_seen, deny_seen)));
    Ok((prefix, deps, cores))
}

/// Load every Limb `dir` depends on, and theirs, each once, under `::prefix`.
fn load_deps(dir: &Path, prog: &mut ast::Program, loaded: &mut Vec<(String, PathBuf)>, chain: &mut Vec<PathBuf>) -> Result<(), String> {
    let (_, deps, _) = read_manifest(dir)?;
    let here = dir.canonicalize().map_err(|e| format!("{}: {e}", dir.display()))?;
    chain.push(here);
    for d in deps {
        let here_file = dir.join("CCA.cplr");
        if d.is_std() { continue; }
        let Some(rel) = &d.path else {
            return Err(format!("{}: {} comes from the registry; the oracle takes `path` dependencies only", here_file.display(), d.what));
        };
        let depdir = dir.join(rel);
        let canon = depdir.canonicalize().map_err(|_| format!("{}: a dependency is at `{rel}`, and there is no directory there", here_file.display()))?;
        if chain.contains(&canon) { return Err(format!("{}: {} depends, through its dependencies, on itself", here_file.display(), d.what)); }
        if !depdir.join("CCA.cplr").is_file() { return Err(format!("{}: {} has no `CCA.cplr`, so no 'prefix' for code to reach it by", here_file.display(), d.what)); }
        let (Some(prefix), _, _) = read_manifest(&depdir)? else { unreachable!() };
        // a Limb may pick `std` or `cca`; it just clashes with the built-in one (owner, 2026-09-29)
        if prefix == "std" || prefix == "cca" { return Err(format!("{}: {} has the prefix `{prefix}`, which is the built-in `{prefix}::`; edit its 'prefix'", here_file.display(), d.what)); }
        if let Some((_, p)) = loaded.iter().find(|(n, _)| *n == prefix) {
            if *p != canon { return Err(format!("{}: two Limbs have the prefix `{prefix}` ({} and {}); edit one Limb's 'prefix'", here_file.display(), p.display(), canon.display())); }
            continue;
        }
        loaded.push((prefix.clone(), canon.clone()));
        let root = vec![format!("::{prefix}")];
        mark_lacks(&depdir, &root)?;
        let mut dep = load_limb(&depdir, root.clone())?;
        // a dependency's MAIN never runs: only the Limb being built has one (the oracle's reading)
        dep.main = None;
        dep.main_args = None;
        prog.modules.push(ast::ModuleDef { path: root, vis: ast::Vis::Pub, line: 0 });
        prog.funcs.extend(dep.funcs); prog.structs.extend(dep.structs); prog.enums.extend(dep.enums);
        prog.interfaces.extend(dep.interfaces); prog.claims.extend(dep.claims); prog.consts.extend(dep.consts);
        prog.externs.extend(dep.externs); prog.modules.extend(dep.modules); prog.strict_modules.extend(dep.strict_modules); prog.bare_marks.extend(dep.bare_marks);
        load_deps(&depdir, prog, loaded, chain)?;
    }
    chain.pop();
    Ok(())
}

/// The power check (owner, 2026-10-07): each use is granted (Signatar grants the dangerous powers to
/// every limb, IEG grants vouching and the operator powers to the limbs that list it), listed in the
/// using Limb's `powers`, and not taken away by the program's `deny`. A Limb with no `CCA.cplr` has
/// every power, and nothing is checked.
fn check_powers(prog: &ast::Program) -> Result<(), String> {
    use ast::Ty;
    let mut uses: Vec<(Vec<String>, String, Option<String>, usize)> = parser::POWER_USES.with(|u| u.borrow().clone());
    // C calls: a call whose path is an extern function
    for (root, path, line) in parser::CALL_SITES.with(|c| c.borrow().clone()) {
        let (module, name) = path.split_at(path.len().max(1) - 1);
        if prog.funcs.iter().any(|f| f.is_extern && f.module == module && Some(&f.name) == name.first()) {
            uses.push((root, "c_calls.functions".into(), name.first().cloned(), line));
        }
    }
    // C variables: a quoted name that is an extern variable of the same Limb
    for (root, name, line) in parser::VAR_SITES.with(|c| c.borrow().clone()) {
        let in_root = |m: &[String]| m.first().filter(|r| r.starts_with("::")).cloned().into_iter().collect::<Vec<_>>() == root;
        if prog.externs.iter().any(|e| e.name == name && in_root(&e.module)) { uses.push((root, "c_calls.variables".into(), Some(name), line)); }
    }
    // an operator whose every operand is a built-in type (owner, 2026-10-07: "`a`")
    for f in prog.funcs.iter().filter(|f| f.is_op && !f.params.is_empty()) {
        let builtin = |t: &Option<Ty>| matches!(t, Some(Ty::Int(_) | Ty::Uint(_) | Ty::Bin(_) | Ty::Brain | Ty::Deci(_) | Ty::Bool | Ty::Str(_) | Ty::Char(_)));
        if f.params.iter().all(|p| builtin(&p.ty)) {
            let root: Vec<String> = f.module.first().filter(|r| r.starts_with("::")).cloned().into_iter().collect();
            let what = if f.params.len() == 1 { "prefix_operators" } else { "operators_on_builtin_types" };
            uses.push((root, what.into(), Some(f.name.rsplit('.').next().unwrap_or(&f.name).to_string()), f.line));
        }
    }
    LIMB_POWERS.with(|lp| {
        let lp = lp.borrow();
        let program = lp.iter().find(|l| l.root.is_empty());
        for (root, path, thing, line) in uses {
            let Some(limb) = lp.iter().find(|l| l.root == root) else { continue };
            let segs: Vec<&str> = path.split('.').collect();
            let who = if root.is_empty() { "this Limb".to_string() } else { format!("`{}`", limb.prefix) };
            let what = match (segs[0], segs.get(1).copied(), &thing) {
                ("inline_assembly", _, _) => "runs assembly (`inline.Assembly`)".to_string(),
                ("c_calls", Some("functions"), Some(t)) => format!("calls C (`{t}`)"),
                ("c_calls", Some("variables"), Some(t)) => format!("uses C's variable '{t}'"),
                ("raw_memory", Some(n), Some(t)) => format!("{} raw memory (`{t}`)", match n { "read" => "reads", "write" => "writes", _ => "casts" }),
                ("vouch", Some("c"), _) => "vouches with `NO_FUCKING_CHECKS`".to_string(),
                ("vouch", _, _) => "vouches with `!check`".to_string(),
                (_, _, Some(t)) => format!("defines the operator `{t}` on built-in types"),
                _ => format!("uses `{path}`"),
            };
            let at = if root.is_empty() { format!("line {line}") } else { format!("line {line} of `{}`", limb.prefix) };
            // 1. granted: Signatar's to every limb; vouching and the operator powers by IEG
            let by_ieg = matches!(segs[0], "vouch" | "operators_on_builtin_types" | "prefix_operators");
            if by_ieg && !limb.cores.iter().any(|c| c == "IEG") {
                return Err(format!("{at}: {who} {what}, a power no core it lists grants: IEG grants `{}` (add `{}` to its `cores`)", segs[0], parser::core_line("IEG")));
            }
            // 2. listed by the Limb
            let covers = |p: &PowerRef| p.path.len() <= segs.len() && p.path.iter().zip(&segs).all(|(a, b)| a == b) && (p.things.is_empty() || thing.as_ref().map_or(false, |t| p.things.contains(t)));
            if !limb.powers.iter().any(covers) {
                return Err(format!("{at}: {who} {what}, and its CCA.cplr does not list `{}`: add `cca::Power.{}` to its `powers`", segs[0], segs[0]));
            }
            // 3. not denied by the program
            if let Some(prog_limb) = program {
                for d in &prog_limb.deny {
                    let path_ok = d.path.len() <= segs.len() && d.path.iter().zip(&segs).all(|(a, b)| a == b);
                    let things_ok = d.things.is_empty() || thing.as_ref().map_or(false, |t| d.things.contains(t));
                    let for_ok = d.for_.is_empty() || d.for_.contains(&limb.prefix);
                    let except_hit = thing.as_ref().map_or(false, |t| d.except.contains(t));
                    if path_ok && things_ok && for_ok && !except_hit {
                        return Err(format!("{at}: {who} {what}, which this program denies (`deny`: `cca::Power.{}`)", d.path.join(".")));
                    }
                }
            }
        }
        Ok(())
    })
}

/// Every `pub` thing says how other Limbs write its path, on itself, its module or its kind, and the
/// marks that cover it agree (owner, 2026-10-06: "I don't think there should be a default"; "`c`").
fn check_bare(prog: &ast::Program) -> Result<std::collections::HashMap<String, ast::Bare>, String> {
    use ast::{Bare, BareTarget, Vis};
    let block = |m: &[String]| prog.modules.iter().any(|d| d.path == m && d.line > 0);
    let word = |m: Bare| match m { Bare::Never => "bare.never", Bare::Optional => "bare.optional", Bare::Only => "bare.only" };
    let mut marks = std::collections::HashMap::new();
    let mut items: Vec<(&str, &str, Vec<String>, usize)> = Vec::new();
    for f in &prog.funcs {
        if f.vis != Vis::Pub || f.is_test { continue; }
        let last = f.name.rsplit('.').next().unwrap_or("");
        // a glyph operator has nowhere to write a path (2026-10-02)
        if f.is_op && !last.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') { continue; }
        items.push((if f.is_op { "op" } else { "fn" }, &f.name, f.module.clone(), f.line));
    }
    let module_of = |name: &str| { let mut v: Vec<String> = name.split('.').map(str::to_string).collect(); v.pop(); v };
    for d in &prog.structs { if d.vis == Vis::Pub { items.push(("struct", &d.name, module_of(&d.name), d.line)); } }
    for d in &prog.enums { if d.vis == Vis::Pub { items.push(("enum", &d.name, d.module.clone(), d.line)); } }
    // a const takes no mark: `'pi'` alone is a variable, so its path is always written (owner, 2026-10-06: "`b`")
    for d in &prog.interfaces { if d.vis == Vis::Pub { items.push(("interface", &d.name, d.module.clone(), d.line)); } }
    for (kind, name, module, line) in items {
        let root: Vec<String> = match module.first() { Some(r) if r.starts_with("::") => vec![r.clone()], _ => Vec::new() };
        // `shapes.area` in this Limb, `areas::shapes.area` in a dependency
        let last = name.rsplit('.').next().unwrap_or(name);
        let mut segs: Vec<String> = module.iter().map(|m| m.trim_start_matches("::").to_string()).collect();
        segs.push(last.to_string());
        let shown = if root.is_empty() { segs.join(".") } else { format!("{}::{}", segs[0], segs[1..].join(".")) };
        let covering: Vec<&ast::BareMark> = prog.bare_marks.iter().filter(|m| match &m.target {
            BareTarget::Item(k, n) => k == kind && *n == module.iter().map(String::as_str).chain(std::iter::once(last)).collect::<Vec<_>>().join("."),
            BareTarget::Block(p) => module.starts_with(p),
            BareTarget::File(p) => module.starts_with(p) && (p.len()..module.len()).all(|i| block(&module[..=i])),
            BareTarget::Kind(k) => k == kind && m.limb_root == root,
        }).collect();
        let Some(first) = covering.first() else {
            return Err(format!("line {line}: `{kind} {shown}` is `pub`, and nothing says how other Limbs write its path: mark it (`pub bare.never {kind} …`), its module, or its kind in `limb.cpls` (`bare.never [{kind}];`) as `bare.never` (path required), `bare.optional` or `bare.only` (path forbidden); there is no default"));
        };
        if let Some(other) = covering.iter().find(|m| m.mode != first.mode) {
            return Err(format!("line {line}: `{kind} {shown}` is marked both `{}` (line {}) and `{}` (line {}): marks that disagree are an error", word(first.mode), first.line, word(other.mode), other.line));
        }
        marks.insert(interp::bare_key(kind, &module, last), first.mode);
    }
    // two Limbs forbidding the path to one name clash as soon as both are depended on (owner,
    // 2026-10-06: "`a`"), fixed as a prefix clash is, by editing one Limb's source
    let mut only: Vec<(String, String)> = Vec::new();
    for (key, mode) in &marks {
        if *mode != Bare::Only { continue; }
        let Some((kind, path)) = key.split_once(':') else { continue };
        let Some(rest) = path.strip_prefix("::") else { continue };
        let (root, rel) = rest.split_once('.').unwrap_or((rest, ""));
        let rel = format!("{kind} {rel}");
        if let Some((other, _)) = only.iter().find(|(r, x)| *x == rel && r != root) {
            let mut two = [other.clone(), root.to_string()]; two.sort();
            return Err(format!("`{}` and `{}` both forbid the path to `{rel}` (`bare.only`), so it cannot tell them apart: edit one Limb's source, as for a prefix clash", two[0], two[1]));
        }
        only.push((root.to_string(), rel));
    }
    Ok(marks)
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
            if stem == "my" { return Err(format!("{}: `my` is the root's path (`my.helper[]`); a module cannot be called that", p.display())); }
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
    let marks = check_bare(&prog).map_err(|m| format!("{}: {m}", path.display()))?;
    let mut interp = interp::Interp::new(&prog, out);
    interp.set_bare(marks);
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
    for (sym, word) in [("`xx.ieee`", "`pow.ieee`"), ("`+.ieee`", "`plus.ieee`"), ("`-.ieee`", "`minus.ieee`"), ("`x.ieee`", "`times.ieee`"), ("`xx~`", "`pow.wrap`"), ("`x~`", "`times.wrap`"), ("`+~`", "`plus.wrap`"), ("`-~`", "`minus.wrap`"), ("`xx`", "`pow`"), ("`x`", "`times`"), ("`+`", "`plus`"), ("`-`", "`minus`")] {
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
