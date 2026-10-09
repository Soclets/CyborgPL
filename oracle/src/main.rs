//! `oracle file.nrv` — run a Nerve program the slow, obvious way.
//!
//! This is the reference interpreter: the executable specification. It knows
//! exactly what `design/` has decided and refuses everything else with a
//! message that says so. Where the design leaves a question open and the
//! interpreter has to pick, the pick is marked "provisional" in the source
//! and listed in oracle/README.md.

#![allow(dead_code)]

mod ast;
mod bigint;
mod case;
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
    /// a Neuron's levels apply wherever its operators are used)
    static LEVELS_ONLY: std::cell::Cell<bool> = std::cell::Cell::new(false);
}

/// Parse one file of the Neuron rooted at `limb_root` (`[]` for the Neuron being built).
fn parse_file_in(path: &Path, module: Vec<String>, limb_root: Vec<String>) -> Result<ast::Program, String> {
    let src = std::fs::read_to_string(path).map_err(|e| format!("{}: cannot read: {e}", path.display()))?;
    let mut toks = lexer::lex(&src).map_err(|e| format!("{}: line {}:{}: {}", path.display(), e.line, e.col, e.msg))?;
    case::rewrite(&mut toks, &limb_root, &module);
    if LEVELS_ONLY.with(|f| f.get()) && path.file_name().map_or(true, |n| n != "Kinesin.nrp") {
        parser::Parser::new_in_limb(toks, module, limb_root).scan_levels()
            .map_err(|e| format!("{}: line {}: {}", path.display(), e.line, e.msg))?;
        return Ok(ast::Program { funcs: Vec::new(), structs: Vec::new(), enums: Vec::new(), interfaces: Vec::new(), claims: Vec::new(), consts: Vec::new(), externs: Vec::new(), modules: Vec::new(), main: None, main_args: None, strict_modules: Vec::new(), bare_marks: Vec::new(), case_marks: Vec::new() });
    }
    parser::Parser::new_in_limb(toks, module, limb_root).program()
        .map_err(|e| if e.col == 0 { format!("{}: line {}: {}", path.display(), e.line, e.msg) } else { format!("{}: line {}:{}: {}", path.display(), e.line, e.col, e.msg) })
}

/// A Neuron is a directory: `neuron.nrv` is the root; every other `.nrv` file is
/// a module named after it, directories nesting (decided 2026-09-19). A lone
/// `.nrv` file is a Neuron of one root file.
fn load(path: &Path) -> Result<ast::Program, String> {
    // first every file's `precedence` blocks, then the program with all of them known
    parser::SHARED_LEVELS.with(|l| l.borrow_mut().clear());
    parser::LACKS.with(|n| n.borrow_mut().clear());
    parser::TYPE_PATHS.with(|t| t.borrow_mut().clear());
    case::PLAN.with(|p| p.borrow_mut().clear());
    LEVELS_ONLY.with(|f| f.set(true));
    let first = load_files(path);
    LEVELS_ONLY.with(|f| f.set(false));
    first?;
    // then the program once to learn which of other Neurons' structs and enums may be written bare
    // (owner, 2026-10-06), and again with that known
    parser::BARE_TYPES.with(|t| t.borrow_mut().clear());
    let once = load_files(path)?;
    let marks = check_bare(&once).unwrap_or_default();
    // which names are case-insensitive (owner, 2026-10-06), so the last reading spells each as declared
    let plan = case::plan(&once, &marks).map_err(|m| format!("{}: {m}", path.display()))?;
    case::PLAN.with(|p| *p.borrow_mut() = plan);
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
    let mut prog = load_files(path)?;
    // the base's `optional` (owner, 2026-10-08: "an iterator's next returns an Option-like value", named
    // `optional` with `value` and `none`): every program has it, as it has `list`
    let prelude = lexer::lex("enum optional <T: any> { value: <T>  none }").map_err(|e| format!("the base's optional: {}", e.msg))?;
    let base = parser::Parser::new(prelude).program().map_err(|e| format!("the base's optional: {}", e.msg))?;
    if prog.enums.iter().any(|e| e.name == "optional") {
        return Err(format!("{}: `optional` is the base's (`optional.value[…]` / `optional.none`); an enum cannot take its name", path.display()));
    }
    prog.enums.extend(base.enums);
    check_powers(&prog).map_err(|m| format!("{}: {m}", path.display()))?;
    check_type_names(&prog).map_err(|m| format!("{}: {m}", path.display()))?;
    check_redeclared(&prog).map_err(|m| format!("{}: {m}", path.display()))?;
    check_return_literals(&prog).map_err(|m| format!("{}: {m}", path.display()))?;
    // a bare call with more than one candidate is refused where it is written, run or not (owner,
    // 2026-10-06: "`a`, and no"); the interpreter makes the same check when a call runs
    for (from, written, std, line) in parser::BARE_CALLS.with(|b| b.borrow().clone()) {
        let shown = written.join(".");
        let own_module: Vec<String> = from.iter().cloned().chain(written[..written.len() - 1].iter().cloned()).collect();
        let own = prog.funcs.iter().any(|f| f.module == own_module && Some(&f.name) == written.last() && !f.is_op);
        let mine = from.first().filter(|r| r.starts_with("::")).cloned();
        let mut who: Vec<String> = Vec::new();
        if own { who.push(format!("`my.{}` (this Neuron's)", own_module.iter().filter(|m| !m.starts_with("::")).chain(written.last()).cloned().collect::<Vec<_>>().join("."))); }
        if std { who.push(format!("`cord::{shown}`")); }
        for f in &prog.funcs {
            let Some(root) = f.module.first().filter(|r| r.starts_with("::")) else { continue };
            if Some(root) == mine.as_ref() || f.vis != ast::Vis::Pub || f.is_op { continue; }
            let rel: Vec<&str> = f.module[1..].iter().map(String::as_str).chain(std::iter::once(f.name.as_str())).collect();
            if rel != written.iter().map(String::as_str).collect::<Vec<_>>() { continue; }
            if matches!(marks.get(&interp::bare_key("fn", &f.module, &f.name)), Some(ast::Bare::Optional | ast::Bare::Only)) { who.push(format!("`{}::{shown}`", root.trim_start_matches("::"))); }
        }
        if who.len() > 1 { return Err(format!("{}: line {line}: `{shown}` could be any of {}: write the path", path.display(), who.join(", "))); }
    }
    // this Neuron's own struct or enum does not win over another Neuron's bare one (owner, 2026-10-06: "no")
    for (own, theirs, written, root) in parser::BARE_TYPE_USES.with(|u| u.borrow().clone()) {
        if prog.structs.iter().any(|d| d.name == own) || prog.enums.iter().any(|d| d.name == own) || prog.interfaces.iter().any(|d| d.name == own) {
            let _ = theirs;
            return Err(format!("{}: `{written}` could be any of `my.{written}` (this Neuron's), `{}::{written}`: write the path", path.display(), match root.trim_start_matches("::") { "std" => "cord", r => r }));
        }
    }
    Ok(prog)
}

/// A Neuron whose `Kinesin.nrp` does not list std or a core reaches none of it (owner, 2026-10-05/06). One with no
/// `Kinesin.nrp` gets what the machine's package installed, and the oracle stands in for the full package.
/// A power as a manifest names it (owner, 2026-10-07): its path, the things named in `[ ]`, and for a
/// `deny` its `.except[ … ]` and `.for[ … ]`.
#[derive(Clone, Debug, Default)]
struct PowerRef { path: Vec<String>, things: Vec<String>, except: Vec<String>, for_: Vec<String> }

/// Each Neuron's manifest, as the power check needs it: its root, prefix, cores, listed powers and denies.
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
    if p.path.is_empty() { return Err("a power names its path: `kin::Power.c_calls`".into()); }
    Ok(p)
}

fn mark_lacks(dir: &Path, root: &[String]) -> Result<(), String> {
    if !dir.join("Kinesin.nrp").is_file() { return Ok(()); }
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
        if path.extension().map(|x| x == "cpls").unwrap_or(false) { return Err(format!("{}: `.cpls` is the old ending: rename the file to `.nrv` (owner, 2026-10-09)", path.display())); }
        if path.extension().map(|x| x != "nrv").unwrap_or(true) { return Err("oracle: a Nerve source file ends in .nrv".into()); }
        return parse_file(path, Vec::new());
    }
    if !path.is_dir() { return Err(format!("oracle: {} is neither a .nrv file nor a Neuron directory", path.display())); }
    mark_lacks(path, &[])?;
    let mut prog = load_limb(path, Vec::new())?;
    // the Neurons it depends on, from its `Kinesin.nrp` (owner, 2026-09-29), each under a root of its own
    let mut loaded: Vec<(String, PathBuf)> = Vec::new();
    load_deps(path, &mut prog, &mut loaded, &mut Vec::new())?;
    Ok(prog)
}

/// One Neuron directory, its modules under `root` (`[]` for the Neuron being built).
fn load_limb(path: &Path, root: Vec<String>) -> Result<ast::Program, String> {
    if !path.join("neuron.nrv").is_file() && path.join("limb.cpls").is_file() { return Err(format!("{}: `limb.cpls` is the old root file: rename it to `neuron.nrv` (owner, 2026-10-09)", path.display())); }
    let rootfile = path.join("neuron.nrv");
    if !rootfile.is_file() { return Err(format!("{}: a Neuron directory needs its root file, neuron.nrv", path.display())); }
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
        if part.main.is_some() { return Err(format!("{}: MAIN lives in neuron.nrv, the root of the Neuron", file.display())); }
        prog.funcs.extend(part.funcs);
        prog.structs.extend(part.structs);
        prog.consts.extend(part.consts);
        prog.externs.extend(part.externs);
        prog.modules.extend(part.modules);
        prog.strict_modules.extend(part.strict_modules);
        prog.bare_marks.extend(part.bare_marks);
        prog.case_marks.extend(part.case_marks);
        prog.enums.extend(part.enums);
        prog.interfaces.extend(part.interfaces);
        prog.claims.extend(part.claims);
    }
    Ok(prog)
}

/// A dependency from `Kinesin.nrp`: `kin::Dep[path = "../geometry"]` (or an `id` and `version`, from
/// the registry, which the oracle cannot fetch). It names no word: code writes the Neuron's own 'prefix'.
struct Dep { what: String, path: Option<String>, version: Option<String>, id: Option<String> }

impl Dep { fn is_std(&self) -> bool { self.id.as_deref() == Some(parser::STD_ID) } }

/// The official cores (design/cores.md, 2026-10-06). Another core is a Rust plugin, which the oracle
/// cannot load.
const OFFICIAL_CORES: [&str; 13] = ["IEG", "MCPS", "AutoGPU", "Metal", "Nvidia", "Vulkan", "WSTN", "TAS", "AI", "WSMW", "Money", "UniTect", "CAPS"];

use parser::CORE_IDS;

/// Read a Neuron's `Kinesin.nrp`: one Nerve value, `kin::Manifest[ … ]` (owner, 2026-10-06: "`c`. It may
/// not look as complex, but at least it's not much of a roleplay"), replacing 2026-09-29's one `const`
/// per setting. Returns its 'prefix' (none when there is no `Kinesin.nrp`), its dependencies and its cores.
fn read_manifest(dir: &Path) -> Result<(Option<String>, Vec<Dep>, Vec<String>), String> {
    let file = dir.join("Kinesin.nrp");
    if !file.is_file() { return Ok((None, Vec::new(), Vec::new())); }
    let shown = file.display();
    let src = std::fs::read_to_string(&file).map_err(|e| format!("{shown}: cannot read: {e}"))?;
    let toks = lexer::lex(&src).map_err(|e| format!("{shown}: line {}:{}: {}", e.line, e.col, e.msg))?;
    let one = "Kinesin.nrp is one value: `kin::Manifest[name = \"shop\", prefix = \"shop\", version = \"1.0.0\", targets = [kin::Target[name = \"aarch64-macos\"],]];`";
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
                parser::module_name_ok(&n).map_err(|m| format!("{shown}: line {line}: `{n}` cannot name a Neuron: {m}"))?;
                name = true;
            }
            // the word code writes in front of `::` to reach this Neuron, its maker's pick (owner, 2026-09-29: b, "required")
            "prefix" => {
                let p = text(value).ok_or_else(|| format!("{shown}: line {line}: `prefix` is a text: `prefix = \"shop\"`"))?;
                parser::module_name_ok(&p).map_err(|m| format!("{shown}: line {line}: `{p}` cannot be a prefix, written in code as `{p}::`: {m}"))?;
                prefix = Some(p);
            }
            "version" => {
                let v = text(value).ok_or_else(|| format!("{shown}: line {line}: `version` is a text"))?;
                // any text, at most 256 bytes, as in Synthia.ttsr (owner, 2026-09-29: "256 bytes it is"; for Kinesin.nrp: "same")
                if v.len() > 256 { return Err(format!("{shown}: line {line}: `version` is at most 256 bytes, and this one is {}", v.len())); }
                has_version = true;
            }
            "target" => return Err(format!("{shown}: line {line}: a Neuron builds for several targets, so the field is `targets`: `targets = [kin::Target[name = \"aarch64-macos\"],]`")),
            // several targets, each with its settings (owner, 2026-09-29): Synapse's package manager can do
            // whatever Synthia can, in its plainer way
            "targets" => {
                let items = items(value);
                let count = items.len();
                targets += count;
                let mut defaults = 0;
                let mut names = Vec::new();
                for it in items {
                    let ast::Expr::Construct(shape, fields, line) = it else { return Err(format!("{shown}: line {line}: a target is `kin::Target[name = …]`, with a `runner` if it has one")) };
                    if shape != "::kin.Target" { return Err(format!("{shown}: line {line}: a target is a `kin::Target`")); }
                    if !fields.iter().any(|(f, _)| f == "name") { return Err(format!("{shown}: line {line}: a target needs its `name`")); }
                    for (f, b) in &fields {
                        // the one built when none is named (owner, 2026-09-29: b)
                        if f == "default" {
                            match b.values.as_slice() { [ast::Value::Expr(ast::Expr::Bool(true, _))] => defaults += 1, [ast::Value::Expr(ast::Expr::Bool(false, _))] => {},
                                _ => return Err(format!("{shown}: line {line}: `default` is `true` or `false`")) }
                            continue;
                        }
                        if f != "name" && f != "runner" { return Err(format!("{shown}: line {line}: a `kin::Target` has no `{f}`; it has `name`, `runner` and `default`")); }
                        let t = text(b).ok_or_else(|| format!("{shown}: line {line}: `{f}` is a text"))?;
                        if f == "name" { names.push((t, line)); }
                    }
                }
                if count > 1 && defaults != 1 { return Err(format!("{shown}: line {line}: with several targets, exactly one says `default = true` — the one built when none is named ({defaults} do)")); }
                // only a target Synapse can build (owner, 2026-09-30: "refuse", as Synthia.ttsr's "Only what's
                // actually doable"); the one list is Synapse's
                let doable: Vec<&str> = include_str!("../../synapse/targets.txt").lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).collect();
                for (t, line) in names {
                    if !doable.contains(&t.as_str()) { return Err(format!("{shown}: line {line}: `{t}` is not a target Synapse can build; it builds for {}", doable.iter().map(|d| format!("`{d}`")).collect::<Vec<_>>().join(", "))); }
                }
            }
            "dependencies" => {
                for it in items(value) {
                    let ast::Expr::Construct(shape, fields, line) = it else { return Err(format!("{shown}: line {line}: a dependency is `kin::Dep[path = …]` or `kin::Dep[id = …, version = …]`")) };
                    if shape != "::kin.Dep" { return Err(format!("{shown}: line {line}: a dependency is a `kin::Dep`, not `{}`", shape.trim_start_matches("::").replacen('.', "::", 1))); }
                    let mut d = Dep { what: String::new(), path: None, version: None, id: None };
                    for (f, b) in &fields {
                        // two Neurons with one prefix are an error, fixed by editing a Neuron's source (owner, 2026-09-29:
                        // "Just edit the source code"), so a dependency names no word
                        if f == "name" || f == "prefix" { return Err(format!("{shown}: line {line}: a dependency names no word: code writes the Neuron's own 'prefix', from its `Kinesin.nrp`")); }
                        let t = text(b).ok_or_else(|| format!("{shown}: line {line}: `{f}` is a text"))?;
                        match f.as_str() { "path" => d.path = Some(t), "version" => d.version = Some(t), "id" => d.id = Some(t),
                            other => return Err(format!("{shown}: line {line}: a `kin::Dep` has no `{other}`; it has `path`, or `id` and `version`")) }
                    }
                    d.what = match (&d.path, &d.id) { (Some(p), _) => format!("the Neuron at `{p}`"), (None, Some(i)) => format!("the Neuron `{}…`", &i[..i.len().min(8)]), _ => "the dependency".into() };
                    if d.path.is_some() == d.version.is_some() { return Err(format!("{shown}: line {line}: a dependency says where it comes from once: a `path` on this machine, or an `id` and `version` from the registry")); }
                    // the registry knows a Neuron by its ID, never its name (owner, 2026-09-29): names may repeat
                    // an ID is 64 hex digits wherever it is written (the owner's definition; compiler claude, 2026-09-29)
                    if let Some(i) = &d.id {
                        if i.len() != 64 || !i.bytes().all(|b| b.is_ascii_hexdigit()) { return Err(format!("{shown}: line {line}: an id is 64 hex digits, 256 random bits")); }
                    }
                    if d.version.is_some() && d.id.is_none() { return Err(format!("{shown}: line {line}: a `version` comes from the registry, which knows a Neuron by its ID: `id = \"…\"`, 64 hex digits")); }
                    // std is listed like any limb, by its ID and version; Synapse uses the installed copy when they
                    // match (owner, 2026-10-06), and the oracle stands in for that copy
                    if d.is_std() {
                        if deps.iter().any(|x: &Dep| x.is_std()) { return Err(format!("{shown}: line {line}: cord is listed twice")); }
                        if d.version.as_deref() != Some(parser::STD_VERSION) { return Err(format!("{shown}: line {line}: the oracle stands in for cord `{}` only, the installed one; this asks for `{}`", parser::STD_VERSION, d.version.as_deref().unwrap_or(""))); }
                        d.what = "std".into();
                    }
                    deps.push(d);
                }
            }
            // the cores a Neuron is built for (owner, 2026-10-05: "Limbs should say what cores they are built
            // for"), each by its registry ID and version, or a `path` for one never registered (2026-10-06)
            // a core is a lobe (owner, 2026-10-09); `cores` and `kin::Core` are the old spellings
            "cores" => return Err(format!("{shown}: line {line}: `cores` is an old spelling: write `lobes` (owner, 2026-10-09)")),
            "lobes" => {
                for it in items(value) {
                    let ast::Expr::Construct(shape, fields, line) = it else { return Err(format!("{shown}: line {line}: a lobe is `kin::Lobe[id = \"…\", version = \"…\"]`")) };
                    if shape == "::kin.Core" { return Err(format!("{shown}: line {line}: `kin::Core` is an old spelling: write `kin::Lobe` (owner, 2026-10-09)")); }
                    if shape != "::kin.Lobe" { return Err(format!("{shown}: line {line}: a lobe is a `kin::Lobe`, not `{}`", shape.trim_start_matches("::").replacen('.', "::", 1))); }
                    let (mut id, mut version, mut cpath) = (None, None, None);
                    for (f, b) in &fields {
                        let t = text(b).ok_or_else(|| format!("{shown}: line {line}: a lobe's `{f}` is a text"))?;
                        match f.as_str() {
                            "id" => id = Some(t), "version" => version = Some(t), "path" => cpath = Some(t),
                            "name" => return Err(format!("{shown}: line {line}: a lobe is listed by its registry ID since 2026-10-06 (names repeat, IDs do not): `kin::Lobe[id = \"…\", version = \"…\"]`{}", CORE_IDS.iter().find(|(n, _)| *n == t).map(|(n, i)| format!("; {n}'s is `{i}`")).unwrap_or_default())),
                            other => return Err(format!("{shown}: line {line}: a `kin::Lobe` has no `{other}`; it has `id` and `version`, or a `path`")),
                        }
                    }
                    if cpath.is_some() {
                        if id.is_some() || version.is_some() { return Err(format!("{shown}: line {line}: a lobe says where it comes from once: an `id` and `version` from the registry, or a `path` on this machine")); }
                        return Err(format!("{shown}: line {line}: the lobe at `{}` is not registered, so not verified; loading another lobe is not built yet (a Rust plugin, which the oracle cannot load)", cpath.unwrap()));
                    }
                    let id = id.ok_or_else(|| format!("{shown}: line {line}: a lobe needs its `id` (64 hex digits) and `version`"))?;
                    if id.len() != 64 || !id.bytes().all(|b| b.is_ascii_hexdigit()) { return Err(format!("{shown}: line {line}: an id is 64 hex digits, 256 random bits")); }
                    let version = version.ok_or_else(|| format!("{shown}: line {line}: a lobe needs its `version`, as a registry limb does"))?;
                    if id == parser::MTM_ID { return Err(format!("{shown}: line {line}: MTM is gone (owner, 2026-10-07): every other spelling is refused, naming the one to write; take this lobe out")); }
                    let Some((core, _)) = CORE_IDS.iter().find(|(_, i)| *i == id) else {
                        return Err(format!("{shown}: line {line}: the lobe `{}…` is not an official one; loading another lobe is not built yet (a Rust plugin, which the oracle cannot load)", &id[..8]));
                    };
                    // ASP and Genny folded into the base (owner, 2026-10-08: "`a`" to both); their IDs no longer name a lobe
                    if *core == "ASP" || *core == "Genny" { return Err(format!("{shown}: line {line}: {core} is retired: since 2026-10-08 it is part of the Brainstem, Nerve's base; take it out of `lobes`")); }
                    let want = parser::core_version(core);
                    if version != want { return Err(format!("{shown}: line {line}: the oracle stands in for {core} `{want}` only, the installed one; this asks for `{version}`")); }
                    let core = core.to_string();
                    if cores.contains(&core) { return Err(format!("{shown}: line {line}: the lobe {core} is listed twice")); }
                    cores.push(core);
                }
            }
            // the powers a Neuron uses, and what the program takes away (owner, 2026-10-07)
            "powers" | "deny" => {
                let mut list = Vec::new();
                for it in items(value) {
                    let ast::Expr::Text(t, _) = &it else { return Err(format!("{shown}: line {line}: a power is `kin::Power.c_calls`, narrowed with `.` and naming things in `[ ]`")) };
                    let Some(text) = t.strip_prefix("power:") else { return Err(format!("{shown}: line {line}: a power is `kin::Power.c_calls`, not a text")) };
                    let pw = parse_power(text).map_err(|m| format!("{shown}: line {line}: {m}"))?;
                    if f == "powers" && (!pw.except.is_empty() || !pw.for_.is_empty()) { return Err(format!("{shown}: line {line}: `.except` and `.for` belong to `deny`; `powers` lists what this Neuron uses")); }
                    list.push(pw);
                }
                if f == "powers" { powers_seen = list; } else { deny_seen = list; }
            }
            other => return Err(format!("{shown}: line {line}: a `kin::Manifest` has no field `{other}`; its fields are `name`, `prefix`, `version`, `targets`, `dependencies`, `lobes`, `powers` and `deny`")),
        }
    }
    // a build needs the Neuron's name, version and a target; only publishing needs more (owner, 2026-09-29:
    // "yea", matching Synthia.ttsr)
    let why = "a build needs the Neuron's `name`, `prefix`, `version` and at least one target";
    if !name { return Err(format!("{shown}: line {mline}: the manifest has no `name`: {why}: `name = \"shop\"`")); }
    if prefix.is_none() { return Err(format!("{shown}: line {mline}: the manifest has no `prefix`, the word code writes in front of `::` to reach this Neuron: {why}: `prefix = \"shop\"`")); }
    if !has_version { return Err(format!("{shown}: line {mline}: the manifest has no `version`: {why}: `version = \"1.0.0\"`")); }
    if targets == 0 { return Err(format!("{shown}: line {mline}: the manifest has no target: {why}: `targets = [kin::Target[name = \"aarch64-macos\"],]`")); }
    // cord needed ASP and IEG until 2026-10-08, when the operators and ASP's number rules moved into
    // the base (owner: "`a`" to both); it needs no lobe now (design claude's reading)
    // cores depend on cores (owner, 2026-10-05); MCPS "Needs IEG, not std" (owner, 2026-10-06)
    if cores.iter().any(|c| c == "MCPS") && !cores.iter().any(|c| c == "IEG") {
        return Err(format!("{shown}: line {mline}: the lobe MCPS needs IEG: add `{}` to `lobes`", parser::core_line("IEG")));
    }
    MANIFEST_POWERS.with(|m| *m.borrow_mut() = Some((powers_seen, deny_seen)));
    Ok((prefix, deps, cores))
}

/// Load every Neuron `dir` depends on, and theirs, each once, under `::prefix`.
fn load_deps(dir: &Path, prog: &mut ast::Program, loaded: &mut Vec<(String, PathBuf)>, chain: &mut Vec<PathBuf>) -> Result<(), String> {
    let (_, deps, _) = read_manifest(dir)?;
    let here = dir.canonicalize().map_err(|e| format!("{}: {e}", dir.display()))?;
    chain.push(here);
    for d in deps {
        let here_file = dir.join("Kinesin.nrp");
        if d.is_std() { continue; }
        let Some(rel) = &d.path else {
            return Err(format!("{}: {} comes from the registry; the oracle takes `path` dependencies only", here_file.display(), d.what));
        };
        let depdir = dir.join(rel);
        let canon = depdir.canonicalize().map_err(|_| format!("{}: a dependency is at `{rel}`, and there is no directory there", here_file.display()))?;
        if chain.contains(&canon) { return Err(format!("{}: {} depends, through its dependencies, on itself", here_file.display(), d.what)); }
        if !depdir.join("Kinesin.nrp").is_file() { return Err(format!("{}: {} has no `Kinesin.nrp`, so no 'prefix' for code to reach it by", here_file.display(), d.what)); }
        let (Some(prefix), _, _) = read_manifest(&depdir)? else { unreachable!() };
        // a Neuron may pick `std` or `cca`; it just clashes with the built-in one (owner, 2026-09-29)
        // `cord` is the built-in's name since 2026-10-09; `cord` (2026-10-07) and `std` are its old ones, refused in code
        if prefix == "std" || prefix == "stdkit" || prefix == "cord" || prefix == "cca" || prefix == "kin" { return Err(format!("{}: {} has the prefix `{prefix}`, which names the built-in `{}::`; edit its 'prefix'", here_file.display(), d.what, if prefix == "cca" { "cca" } else { "cord" })); }
        if let Some((_, p)) = loaded.iter().find(|(n, _)| *n == prefix) {
            if *p != canon { return Err(format!("{}: two Neurons have the prefix `{prefix}` ({} and {}); edit one Neuron's 'prefix'", here_file.display(), p.display(), canon.display())); }
            continue;
        }
        loaded.push((prefix.clone(), canon.clone()));
        let root = vec![format!("::{prefix}")];
        mark_lacks(&depdir, &root)?;
        let mut dep = load_limb(&depdir, root.clone())?;
        // a dependency's MAIN never runs: only the Neuron being built has one (the oracle's reading)
        dep.main = None;
        dep.main_args = None;
        prog.modules.push(ast::ModuleDef { path: root, vis: ast::Vis::Pub, line: 0 });
        prog.funcs.extend(dep.funcs); prog.structs.extend(dep.structs); prog.enums.extend(dep.enums);
        prog.interfaces.extend(dep.interfaces); prog.claims.extend(dep.claims); prog.consts.extend(dep.consts);
        prog.externs.extend(dep.externs); prog.modules.extend(dep.modules); prog.strict_modules.extend(dep.strict_modules); prog.bare_marks.extend(dep.bare_marks); prog.case_marks.extend(dep.case_marks);
        load_deps(&depdir, prog, loaded, chain)?;
    }
    chain.pop();
    Ok(())
}

/// The power check (owner, 2026-10-07): each use is granted (Brainstem grants the dangerous powers to
/// every limb, IEG grants vouching and the operator powers to the limbs that list it), listed in the
/// using Neuron's `powers`, and not taken away by the program's `deny`. A Neuron with no `Kinesin.nrp` has
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
    // C variables: a quoted name that is an extern variable of the same Neuron
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
            let who = if root.is_empty() { "this Neuron".to_string() } else { format!("`{}`", limb.prefix) };
            let what = match (segs[0], segs.get(1).copied(), &thing) {
                ("inline_assembly", _, _) => "runs assembly (`inline.ASM`)".to_string(),
                ("c_calls", Some("functions"), Some(t)) => format!("calls C (`{t}`)"),
                ("c_calls", Some("variables"), Some(t)) => format!("uses C's variable '{t}'"),
                ("raw_memory", Some(n), Some(t)) => format!("{} raw memory (`{t}`)", match n { "read" => "reads", "write" => "writes", _ => "casts" }),
                ("vouch", Some("c"), _) => "vouches with `NO_FUCKING_CHECKS`".to_string(),
                ("vouch", _, _) => "vouches with `no_check`".to_string(),
                (_, _, Some(t)) => format!("defines the operator `{t}` on built-in types"),
                _ => format!("uses `{path}`"),
            };
            let at = if root.is_empty() { format!("line {line}") } else { format!("line {line} of `{}`", limb.prefix) };
            // 1. granted: Brainstem's to every limb; vouching and the operator powers by IEG
            let by_ieg = matches!(segs[0], "vouch" | "operators_on_builtin_types" | "prefix_operators");
            if by_ieg && !limb.cores.iter().any(|c| c == "IEG") {
                return Err(format!("{at}: {who} {what}, a power no lobe it lists grants: IEG grants `{}` (add `{}` to its `lobes`)", segs[0], parser::core_line("IEG")));
            }
            // 2. listed by the Neuron
            let covers = |p: &PowerRef| p.path.len() <= segs.len() && p.path.iter().zip(&segs).all(|(a, b)| a == b) && (p.things.is_empty() || thing.as_ref().map_or(false, |t| p.things.contains(t)));
            if !limb.powers.iter().any(covers) {
                return Err(format!("{at}: {who} {what}, and its Kinesin.nrp does not list `{}`: add `kin::Power.{}` to its `powers`", segs[0], segs[0]));
            }
            // 3. not denied by the program
            if let Some(prog_limb) = program {
                for d in &prog_limb.deny {
                    let path_ok = d.path.len() <= segs.len() && d.path.iter().zip(&segs).all(|(a, b)| a == b);
                    let things_ok = d.things.is_empty() || thing.as_ref().map_or(false, |t| d.things.contains(t));
                    let for_ok = d.for_.is_empty() || d.for_.contains(&limb.prefix);
                    let except_hit = thing.as_ref().map_or(false, |t| d.except.contains(t));
                    if path_ok && things_ok && for_ok && !except_hit {
                        return Err(format!("{at}: {who} {what}, which this program denies (`deny`: `kin::Power.{}`)", d.path.join(".")));
                    }
                }
            }
        }
        Ok(())
    })
}

/// Every `pub` thing says how other Neurons write its path, on itself, its module or its kind, and the
/// marks that cover it agree (owner, 2026-10-06: "I don't think there should be a default"; "`c`").
/// Every type a signature, field, payload, const or C variable names is declared somewhere, even in
/// a function nothing calls (found 2026-10-07: `fn f ['v': nosuchtype] { }` ran).
fn check_type_names(prog: &ast::Program) -> Result<(), String> {
    use ast::Ty;
    let known: std::collections::HashSet<&str> = prog.structs.iter().map(|d| d.name.as_str())
        .chain(prog.enums.iter().map(|d| d.name.as_str()))
        .chain(prog.interfaces.iter().map(|d| d.name.as_str())).collect();
    fn walk<'a>(t: &'a Ty, out: &mut Vec<&'a str>) {
        match t {
            Ty::Struct(n) => out.push(n),
            Ty::Inst(n, args) => { out.push(n); for a in args { walk(a, out); } }
            Ty::List(e) | Ty::Array(e) | Ty::Ptr(e, _) | Ty::Channel(e) | Ty::Simd(e, _) | Ty::Ranged(e, ..) => walk(e, out),
            Ty::Map(k, v, _) => { walk(k, out); walk(v, out); }
            Ty::Fn(ps, r, _) => { for (_, p) in ps { walk(p, out); } if let Some(r) = r { walk(r, out); } }
            _ => {}
        }
    }
    let mut named: Vec<(&str, usize)> = Vec::new();
    let mut found = Vec::new();
    for f in &prog.funcs {
        for p in &f.params { if let Some(t) = &p.ty { found.clear(); walk(t, &mut found); for n in &found { named.push((n, f.line)); } } }
        if let Some(t) = &f.ret { found.clear(); walk(t, &mut found); for n in &found { named.push((n, f.line)); } }
    }
    for d in &prog.structs { for fld in &d.fields { found.clear(); walk(&fld.ty, &mut found); for n in &found { named.push((n, d.line)); } } }
    for d in &prog.enums { for c in &d.cases { if let Some(t) = &c.ty { found.clear(); walk(t, &mut found); for n in &found { named.push((n, d.line)); } } } }
    for c in &prog.consts { if let Some(t) = &c.ty { found.clear(); walk(t, &mut found); for n in &found { named.push((n, c.line)); } } }
    for e in &prog.externs { found.clear(); walk(&e.ty, &mut found); for n in &found { named.push((n, e.line)); } }
    for (n, line) in named {
        if !known.contains(n) {
            let shown = n.trim_start_matches("::");
            return Err(format!("line {line}: no type named `{shown}` is declared"));
        }
    }
    Ok(())
}

/// A plain `let` over a visible name is refused, and `shadow` needs one to shadow, in code that runs
/// or not (check claude, 2026-10-08: the oracle checked only where it ran; owner, 2026-09-14 and
/// 2026-10-08). Scopes as the interpreter keeps them: a function's parameters, then each block;
/// a closure starts afresh with its parameters and captures.
fn check_redeclared(prog: &ast::Program) -> Result<(), String> {
    use ast::{Stmt, Expr, Handler, AsmBind};
    type Scopes = Vec<Vec<(String, bool)>>;   // (name, sealed)
    // a const sits in the outermost scope, its name marked `const:`
    fn find<'a>(sc: &'a Scopes, n: &str) -> Option<&'a (String, bool)> { sc.iter().rev().flat_map(|s| s.iter()).find(|(m, _)| m == n || m.strip_prefix("const:") == Some(n)) }
    fn decl(sc: &mut Scopes, n: &str, shadow: bool, sealed: bool, line: usize) -> Result<(), String> {
        match (find(sc, n), shadow) {
            (Some(_), false) => return Err(format!("line {line}: '{n}' is already declared in an enclosing scope; to shadow it deliberately write `shadow '{n}' := …;` (owner, 2026-10-08)")),
            (None, true) => return Err(format!("line {line}: `shadow` declares over an existing '{n}', but none is visible here")),
            (Some((m, true)), true) if m.starts_with("const:") => return Err(format!("line {line}: '{n}' is a const and cannot be shadowed")),
            (Some((_, true)), true) => return Err(format!("line {line}: '{n}' cannot be shadowed: it is `no_shadow` (or a loop's own counter)")),
            _ => {}
        }
        sc.last_mut().unwrap().push((n.to_string(), sealed));
        Ok(())
    }
    fn block(sc: &mut Scopes, body: &[Stmt], extra: &[(String, bool)]) -> Result<(), String> {
        sc.push(extra.to_vec());
        for st in body { stmt(sc, st)?; }
        sc.pop();
        Ok(())
    }
    fn arms(sc: &mut Scopes, arms: &[ast::Arm]) -> Result<(), String> {
        for a in arms {
            if let Some(v) = &a.value { expr(sc, v)?; }
            block(sc, &a.body, &a.bind.iter().map(|b| (b.clone(), false)).collect::<Vec<_>>())?;
        }
        Ok(())
    }
    fn handler(sc: &mut Scopes, h: &Option<Handler>) -> Result<(), String> {
        match h { Some(Handler::Arms(a)) => arms(sc, a), Some(Handler::Leave(st)) => stmt(sc, st), _ => Ok(()) }
    }
    fn bracket(sc: &mut Scopes, b: &ast::Bracket) -> Result<(), String> {
        for ast::Value::Expr(e) in &b.values { expr(sc, e)?; }
        Ok(())
    }
    fn expr(sc: &mut Scopes, e: &Expr) -> Result<(), String> {
        match e {
            Expr::Call(c) => { bracket(sc, &c.args)?; handler(sc, &c.handler) }
            Expr::Match(v, a, _) => { bracket(sc, v)?; arms(sc, a) }
            Expr::Lambda(f, caps, _) => {
                let mut fresh: Scopes = vec![caps.iter().map(|(_, n)| (n.clone(), false)).chain(f.params.iter().map(|p| (p.name.clone(), false))).collect()];
                block(&mut fresh, &f.body, &[])
            }
            _ => Ok(()),
        }
    }
    fn stmt(sc: &mut Scopes, st: &Stmt) -> Result<(), String> {
        match st {
            Stmt::Decl { name, value, shadow, sealed, line, .. } => { bracket(sc, value)?; decl(sc, name, *shadow, *sealed, *line) }
            Stmt::If { arms: a, otherwise, .. } => { for (c, b) in a { bracket(sc, c)?; block(sc, b, &[])?; } if let Some(o) = otherwise { block(sc, o, &[])?; } Ok(()) }
            Stmt::Loop { body, .. } | Stmt::Block { body, .. } => block(sc, body, &[]),
            Stmt::While { cond, body, .. } => { bracket(sc, cond)?; block(sc, body, &[]) }
            Stmt::For { name, index, over, body, .. } => {
                bracket(sc, over)?;
                let mut names = vec![(name.clone(), true)];
                if let Some(i) = index { names.push((i.clone(), true)); }
                block(sc, body, &names)
            }
            Stmt::Range { name, bounds, body, .. } => { bracket(sc, bounds)?; block(sc, body, &[(name.clone(), true)]) }
            Stmt::Tier { body, .. } => stmt(sc, body),
            Stmt::Match { value, arms: a, .. } => { bracket(sc, value)?; arms(sc, a) }
            Stmt::Call { call, .. } => { bracket(sc, &call.args)?; handler(sc, &call.handler) }
            Stmt::CallValue { args, handler: h, .. } | Stmt::MapFailStmt { args, handler: h, .. } => { bracket(sc, args)?; handler(sc, h) }
            Stmt::Concur { body, handler: h, .. } => { block(sc, body, &[])?; handler(sc, h) }
            Stmt::Whichever { arms: a, handler: h, .. } => { for w in a { block(sc, &w.body, &[(w.name.clone(), false)])?; } handler(sc, h) }
            Stmt::Asm { binds, line, .. } => { for b in binds { if let AsmBind::Out(v, _) = b { if find(sc, v).is_none() { decl(sc, v, false, false, *line)?; } } } Ok(()) }
            _ => Ok(()),
        }
    }
    let consts = |module: &[String]| -> Vec<(String, bool)> { prog.consts.iter().filter(|c| c.module == module).map(|c| (format!("const:{}", c.name), true)).collect() };
    for f in &prog.funcs {
        let mut sc: Scopes = vec![consts(&f.module), f.params.iter().map(|p| (p.name.clone(), false)).collect()];
        block(&mut sc, &f.body, &[])?;
    }
    if let Some(m) = &prog.main {
        let mut sc: Scopes = vec![consts(&[]), prog.main_args.iter().flat_map(|(a, b)| [(a.clone(), false), (b.clone(), false)]).collect()];
        block(&mut sc, m, &[])?;
    }
    Ok(())
}

/// A function's `return` of a plain literal of the wrong kind is refused in code that never runs too
/// (check claude, seed 44056: the oracle checked a return only where it ran), as a `let` over a name is
/// (caba9b84). Only literals are judged here: a `bool` where a number is due, a number where a `bool` is,
/// a text where neither text nor anything else takes it; the interpreter checks the rest as it runs.
fn check_return_literals(prog: &ast::Program) -> Result<(), String> {
    use ast::{Expr, Stmt, Ty};
    fn kind_ok(e: &Expr, t: &Ty) -> Option<&'static str> {
        let number = matches!(t, Ty::Int(_) | Ty::Uint(_) | Ty::Bin(_) | Ty::Brain | Ty::Deci(_));
        let text = matches!(t, Ty::Str(_));
        match e {
            Expr::Bool(..) if number || text => Some("a `bool`"),
            Expr::Int(..) | Expr::Bin(..) | Expr::Exact(..) if matches!(t, Ty::Bool) || text => Some("a number"),
            Expr::Text(..) if number || matches!(t, Ty::Bool) => Some("a text"),
            _ => None,
        }
    }
    fn walk(body: &[Stmt], t: &Ty, f: &str) -> Result<(), String> {
        for st in body {
            match st {
                Stmt::Return { value: Some(b), line } => {
                    if let [ast::Value::Expr(e)] = b.values.as_slice() {
                        if let Some(what) = kind_ok(e, t) {
                            return Err(format!("line {line}: `{f}` returns {}, but this `return` gives {what}", value::ty_name(t)));
                        }
                    }
                }
                Stmt::If { arms, otherwise, .. } => { for (_, b) in arms { walk(b, t, f)?; } if let Some(o) = otherwise { walk(o, t, f)?; } }
                Stmt::Loop { body, .. } | Stmt::Block { body, .. } | Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::Range { body, .. } => walk(body, t, f)?,
                Stmt::Tier { body, .. } => walk(std::slice::from_ref(&**body), t, f)?,
                Stmt::Match { arms, .. } => { for a in arms { walk(&a.body, t, f)?; } }
                _ => {}
            }
        }
        Ok(())
    }
    for func in &prog.funcs {
        if let Some(t) = &func.ret { walk(&func.body, t, &func.name)?; }
    }
    Ok(())
}

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
        // `shapes.area` in this Neuron, `areas::shapes.area` in a dependency
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
            return Err(format!("line {line}: `{kind} {shown}` is `pub`, and nothing says how other Neurons write its path: mark it (`pub bare.never {kind} …`), its module, or its kind in `neuron.nrv` (`bare.never [{kind}];`) as `bare.never` (path required), `bare.optional` or `bare.only` (path forbidden); there is no default"));
        };
        if let Some(other) = covering.iter().find(|m| m.mode != first.mode) {
            return Err(format!("line {line}: `{kind} {shown}` is marked both `{}` ({}) and `{}` ({}): marks that disagree are an error", word(first.mode), crate::interp::line_of(first.line), word(other.mode), crate::interp::line_of(other.line)));
        }
        marks.insert(interp::bare_key(kind, &module, last), first.mode);
    }
    // two Neurons forbidding the path to one name clash as soon as both are depended on (owner,
    // 2026-10-06: "`a`"), fixed as a prefix clash is, by editing one Neuron's source
    let mut only: Vec<(String, String)> = Vec::new();
    for (key, mode) in &marks {
        if *mode != Bare::Only { continue; }
        let Some((kind, path)) = key.split_once(':') else { continue };
        let Some(rest) = path.strip_prefix("::") else { continue };
        let (root, rel) = rest.split_once('.').unwrap_or((rest, ""));
        let rel = format!("{kind} {rel}");
        if let Some((other, _)) = only.iter().find(|(r, x)| *x == rel && r != root) {
            let mut two = [other.clone(), root.to_string()]; two.sort();
            return Err(format!("`{}` and `{}` both forbid the path to `{rel}` (`bare.only`), so it cannot tell them apart: edit one Neuron's source, as for a prefix clash", two[0], two[1]));
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
        } else if name.ends_with(".cpls") || name == "CCA.cplr" || name == "build.cplr" {
            let new = match name.as_str() { "CCA.cplr" => "Kinesin.nrp".to_string(), "build.cplr" => "build.nrv".to_string(), _ => name.replace(".cpls", ".nrv") };
            return Err(format!("{}: `{name}` has the old name: rename it to `{new}` (owner, 2026-10-09)", p.display()));
        } else if let Some(stem) = name.strip_suffix(".nrv") {
            if prefix.is_empty() && stem == "neuron" { continue; }
            // a Neuron's `build.nrv` is its build program, run by `synapse prepare`, not a module (owner, 2026-10-09: "yea")
            if prefix.is_empty() && stem == "build" { continue; }
            if stem == "neuron" { return Err(format!("{}: `neuron` names the root; a module cannot be called that", p.display())); }
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

/// `tests`: run every `test fn` instead of MAIN (`synapse test`, owner 2026-09-29).
pub fn run_path_as(path: &Path, out: Box<dyn std::io::Write>, tests: bool) -> Result<u8, String> {
    run_with_args(path, out, tests, Vec::new())
}

/// `args`: the words after `--`, which MAIN reads as C's argc/argv (owner, 2026-09-29).
pub fn run_with_args(path: &Path, out: Box<dyn std::io::Write>, tests: bool, args: Vec<String>) -> Result<u8, String> {
    interp::set_shown_file(&if path.is_dir() { "neuron.nrv".to_string() } else { path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default() });
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
    // the comparisons are words too since 2026-10-07; the interpreter keeps their old symbols inside
    for (sym, word) in [("`!==`", "`not_equal`"), ("`</==`", "`at_most`"), ("`>/==`", "`at_least`"), ("`==`", "`equal_to`"), ("`xx.ieee`", "`pow.ieee`"), ("`+.ieee`", "`plus.ieee`"), ("`-.ieee`", "`minus.ieee`"), ("`x.ieee`", "`times.ieee`"), ("`xx~`", "`pow.wrap`"), ("`x~`", "`times.wrap`"), ("`+~`", "`plus.wrap`"), ("`-~`", "`minus.wrap`"), ("`xx`", "`pow`"), ("`x`", "`times`"), ("`+`", "`plus`"), ("`-`", "`minus`")] {
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
        _ => { eprintln!("usage: oracle [--test] <file.nrv | limb-directory> [-- program arguments]"); exit(2); }
    };
    // the parser and the interpreter recurse, so a call nested a few thousand deep in one expression
    // needs more than the main thread's 8 MB (check claude, seed 32299): 1 GB, only what is used is touched
    let target = target.clone();
    let run = std::thread::Builder::new().stack_size(1 << 30).spawn(move || run_with_args(Path::new(&target), Box::new(std::io::stdout()), tests, rest))
        .expect("the oracle's thread");
    match run.join() {
        Ok(Ok(code)) => exit(code as i32),
        Ok(Err(msg)) => { eprintln!("{msg}"); exit(1); }
        Err(_) => exit(101),
    };
}
