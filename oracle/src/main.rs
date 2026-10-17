//! `oracle file.cy` — run a CyborgPL program the slow, obvious way.
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
mod cheader;
mod generate;
mod deci;
mod interp;
mod lexer;
mod parser;
mod spellings;
mod value;

use std::path::{Path, PathBuf};
use std::process::exit;

/// Parse one file as the module `module` (empty for the root).
fn parse_file(path: &Path, module: Vec<String>) -> Result<ast::Program, String> {
    parse_file_in(path, module, Vec::new())
}

thread_local! {
    /// `oracle --generate <path>`: write the templates' generated parts instead of running
    static GENERATE: std::cell::Cell<bool> = std::cell::Cell::new(false);
    /// The first pass over a program's files reads only their `precedence` blocks (owner, 2026-10-03:
    /// a kit's levels apply wherever its operators are used)
    static LEVELS_ONLY: std::cell::Cell<bool> = std::cell::Cell::new(false);
}

/// Parse one file of the kit rooted at `kit_root` (`[]` for the kit being built).
fn parse_file_in(path: &Path, module: Vec<String>, kit_root: Vec<String>) -> Result<ast::Program, String> {
    let src = std::fs::read_to_string(path).map_err(|e| format!("{}: cannot read: {e}", path.display()))?;
    let mut toks = lexer::lex(&src).map_err(|e| format!("{}: line {}:{}: {}", path.display(), e.line, e.col, e.msg))?;
    case::rewrite(&mut toks, &kit_root, &module);
    // the kit's folder, where an `incl.…` path starts (owner, 2026-10-10): up from the file's own folder
    // once per module level below the root (`ui/header.cy` is `ui.header`, one folder down)
    let mut dir = path.parent().map(Path::to_path_buf).unwrap_or_default();
    for _ in 0..(module.len().saturating_sub(kit_root.len())).saturating_sub(1) { dir.pop(); }
    // a file named bare (`oracle t.cy`) has the empty path as its folder, which is "here" (compiler claude)
    if dir.as_os_str().is_empty() { dir = PathBuf::from("."); }
    parser::NEURON_DIR.with(|d| *d.borrow_mut() = dir);
    if LEVELS_ONLY.with(|f| f.get()) && path.file_name().map_or(true, |n| n != "CyJet.cyr") {
        parser::Parser::new_in_kit(toks, module, kit_root).scan_levels()
            .map_err(|e| format!("{}: line {}: {}", path.display(), e.line, e.msg))?;
        return Ok(ast::Program { funcs: Vec::new(), structs: Vec::new(), enums: Vec::new(), interfaces: Vec::new(), claims: Vec::new(), consts: Vec::new(), externs: Vec::new(), modules: Vec::new(), main: None, main_args: None, strict_modules: Vec::new(), bare_marks: Vec::new(), case_marks: Vec::new() });
    }
    parser::CURRENT_FILE.with(|f| *f.borrow_mut() = path.to_path_buf());
    let r = parser::Parser::new_in_kit(toks, module.clone(), kit_root.clone()).program()
        .map_err(|e| if e.col == 0 { format!("{}: line {}: {}", path.display(), e.line, e.msg) } else { format!("{}: line {}:{}: {}", path.display(), e.line, e.col, e.msg) })?;
    // a file with collection templates is kept for the check of its generated parts (owner, 2026-10-13)
    if parser::TEMPLATES.with(|t| t.borrow().iter().any(|(f, _, _)| f == path)) {
        generate::PENDING.with(|p| {
            let mut p = p.borrow_mut();
            p.retain(|x| x.path != path);
            p.push(generate::Pending { path: path.to_path_buf(), src: src.clone(), module, kit_root });
        });
    }
    Ok(r)
}

/// A kit is a directory: `kit.cy` is the root; every other `.cy` file is
/// a module named after it, directories nesting (decided 2026-09-19). A lone
/// `.cy` file is a kit of one root file.
fn load(path: &Path) -> Result<ast::Program, String> {
    // first every file's `precedence` blocks, then the program with all of them known
    parser::SHARED_LEVELS.with(|l| l.borrow_mut().clear());
    parser::LACKS.with(|n| n.borrow_mut().clear());
    parser::TYPE_PATHS.with(|t| t.borrow_mut().clear());
    parser::TEMPLATES.with(|t| t.borrow_mut().clear());
    generate::PENDING.with(|p| p.borrow_mut().clear());
    case::PLAN.with(|p| p.borrow_mut().clear());
    LEVELS_ONLY.with(|f| f.set(true));
    let first = load_files(path);
    LEVELS_ONLY.with(|f| f.set(false));
    first?;
    // then the program once to learn which of other kits' structs and enums may be written bare
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
    KIT_POWERS.with(|l| l.borrow_mut().clear());
    parser::POWER_USES.with(|u| u.borrow_mut().clear());
    parser::CALL_SITES.with(|u| u.borrow_mut().clear());
    parser::VAR_SITES.with(|u| u.borrow_mut().clear());
    let mut prog = load_files(path)?;
    // the base's `optional` (owner, 2026-10-08: "an iterator's next returns an Option-like value", named
    // `optional` with `value` and `none`): every program has it, as it has `list`
    let prelude = lexer::lex("enum optional <T: any> { value: <T>, none }").map_err(|e| format!("the base's optional: {}", e.msg))?;
    let base = parser::Parser::new(prelude).program().map_err(|e| format!("the base's optional: {}", e.msg))?;
    if prog.enums.iter().any(|e| e.name == "optional") {
        return Err(format!("{}: `optional` is the base's (`optional.value[…]` / `optional.none`); an enum cannot take its name", path.display()));
    }
    prog.enums.extend(base.enums);
    check_powers(&prog).map_err(|m| format!("{}: {m}", path.display()))?;
    check_type_names(&prog).map_err(|m| format!("{}: {m}", path.display()))?;
    let unkept = check_redeclared(&prog).map_err(|m| format!("{}: {m}", path.display()))?;
    check_widthless_shifts(&prog).map_err(|m| format!("{}: {m}", path.display()))?;
    check_return_literals(&prog).map_err(|m| format!("{}: {m}", path.display()))?;
    if let Some(m) = unkept { return Err(format!("{}: {m}", path.display())); }
    check_opaque_held(&prog).map_err(|m| format!("{}: {m}", path.display()))?;
    // a bare call with more than one candidate is refused where it is written, run or not (owner,
    // 2026-10-06: "`a`, and no"); the interpreter makes the same check when a call runs
    for (from, written, std, line) in parser::BARE_CALLS.with(|b| b.borrow().clone()) {
        let shown = written.join(".");
        let own_module: Vec<String> = from.iter().cloned().chain(written[..written.len() - 1].iter().cloned()).collect();
        let own = prog.funcs.iter().any(|f| f.module == own_module && Some(&f.name) == written.last() && !f.is_op);
        let mine = from.first().filter(|r| r.starts_with("::")).cloned();
        let mut who: Vec<String> = Vec::new();
        if own { who.push(format!("`my.{}` (this kit's)", own_module.iter().filter(|m| !m.starts_with("::")).chain(written.last()).cloned().collect::<Vec<_>>().join("."))); }
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
    // this kit's own struct or enum does not win over another kit's bare one (owner, 2026-10-06: "no")
    for (own, theirs, written, root) in parser::BARE_TYPE_USES.with(|u| u.borrow().clone()) {
        if prog.structs.iter().any(|d| d.name == own) || prog.enums.iter().any(|d| d.name == own) || prog.interfaces.iter().any(|d| d.name == own) {
            let _ = theirs;
            return Err(format!("{}: `{written}` could be any of `my.{written}` (this kit's), `{}::{written}`: write the path", path.display(), match root.trim_start_matches("::") { "std" => "std", r => r }));
        }
    }
    Ok(prog)
}

/// A kit whose `CyJet.cyr` does not list std or a core reaches none of it (owner, 2026-10-05/06). One with no
/// `CyJet.cyr` gets what the machine's package installed, and the oracle stands in for the full package.
/// A power as a manifest names it (owner, 2026-10-07): its path, the things named in `[ ]`, and for a
/// `deny` its `.except[ … ]` and `.for[ … ]`.
#[derive(Clone, Debug, Default)]
struct PowerRef { path: Vec<String>, things: Vec<String>, except: Vec<String>, for_: Vec<String> }

/// Each kit's manifest, as the power check needs it: its root, prefix, cores, listed powers and denies.
struct KitPowers { root: Vec<String>, prefix: String, cores: Vec<String>, powers: Vec<PowerRef>, deny: Vec<PowerRef> }

thread_local! {
    /// the `powers` and `deny` of the manifest just read
    static MANIFEST_POWERS: std::cell::RefCell<Option<(Vec<PowerRef>, Vec<PowerRef>)>> = std::cell::RefCell::new(None);
    static KIT_POWERS: std::cell::RefCell<Vec<KitPowers>> = std::cell::RefCell::new(Vec::new());
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
    if p.path.is_empty() { return Err("a power names its path: `cyjet::power.c_calls`".into()); }
    Ok(p)
}

/// A file under an old name, and the name it has now: CyborgPL's names (owner, 2026-10-16), Zag's
/// (2026-10-15) and Nerve's (2026-10-09) before them
fn renamed_file(name: &str) -> Option<String> {
    Some(match name {
        "strike.zg" | "neuron.nrv" | "limb.cpls" => "kit.cy".into(),
        "Ricochet.zv" | "Kinesin.nrp" | "CCA.cplr" => "CyJet.cyr".into(),
        "build.zg" | "build.nrv" | "build.cplr" => "build.cy".into(),
        "zag_version.zv" | "nerve_version.nrp" => "cy_ver.cyr".into(),
        _ => {
            for (old, new) in [(".zg", ".cy"), (".zv", ".cyr"), (".nrv", ".cy"), (".cpls", ".cy"), (".nrp", ".cyr"), (".cplr", ".cyr")] {
                if let Some(stem) = name.strip_suffix(old) { return Some(format!("{stem}{new}")); }
            }
            return None;
        }
    })
}

fn mark_lacks(dir: &Path, root: &[String]) -> Result<(), String> {
    if !dir.join("CyJet.cyr").is_file() { return Ok(()); }
    let (prefix, deps, cores) = read_manifest(dir)?;
    let (powers, deny) = MANIFEST_POWERS.with(|m| m.borrow_mut().take()).unwrap_or_default();
    KIT_POWERS.with(|l| l.borrow_mut().push(KitPowers { root: root.to_vec(), prefix: prefix.unwrap_or_default(), cores: cores.clone(), powers, deny }));
    let mut lacks: Vec<String> = OFFICIAL_CORES.iter().filter(|c| !cores.iter().any(|x| x == *c)).map(|c| c.to_string()).collect();
    if !deps.iter().any(Dep::is_std) { lacks.push("std".into()); }
    parser::LACKS.with(|n| n.borrow_mut().push((root.to_vec(), lacks)));
    Ok(())
}

fn load_files(path: &Path) -> Result<ast::Program, String> {
    if path.is_file() {
        if let Some(new) = path.file_name().and_then(|n| renamed_file(&n.to_string_lossy())) { return Err(format!("{}: the old ending: rename the file to `{new}`", path.display())); }
        if path.extension().map(|x| x != "cy").unwrap_or(true) { return Err("oracle: a CyborgPL source file ends in .cy".into()); }
        return parse_file(path, Vec::new());
    }
    if !path.is_dir() { return Err(format!("oracle: {} is neither a .cy file nor a kit directory", path.display())); }
    mark_lacks(path, &[])?;
    let mut prog = load_kit(path, Vec::new())?;
    // the kits it depends on, from its `CyJet.cyr` (owner, 2026-09-29), each under a root of its own
    let mut loaded: Vec<(String, PathBuf)> = Vec::new();
    load_deps(path, &mut prog, &mut loaded, &mut Vec::new())?;
    Ok(prog)
}

/// One kit directory, its modules under `root` (`[]` for the kit being built).
fn load_kit(path: &Path, root: Vec<String>) -> Result<ast::Program, String> {
    for old in ["strike.zg", "Ricochet.zv", "limb.cpls", "neuron.nrv", "Kinesin.nrp", "CCA.cplr"] {
        if path.join(old).is_file() { return Err(format!("{}: `{old}` has the old name: rename it to `{}`", path.display(), renamed_file(old).unwrap())); }
    }
    let rootfile = path.join("kit.cy");
    if !rootfile.is_file() { return Err(format!("{}: a kit directory needs its root file, kit.cy", path.display())); }
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
        if part.main.is_some() { return Err(format!("{}: MAIN lives in kit.cy, the root of the kit", file.display())); }
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

/// A dependency from `CyJet.cyr`: `cyjet::dep[path = "../geometry"]` (or an `id` and `version`, from
/// the registry, which the oracle cannot fetch). It names no word: code writes the kit's own 'prefix'.
struct Dep { what: String, path: Option<String>, version: Option<String>, id: Option<String> }

impl Dep { fn is_std(&self) -> bool { self.id.as_deref() == Some(parser::STD_ID) } }

/// The official cores (design/cores.md, 2026-10-06). Another core is a Rust limb, which the oracle
/// cannot load.
const OFFICIAL_CORES: [&str; 10] = ["MCPS", "AutoGPU", "Metal", "Nvidia", "Vulkan", "AI", "WSMW", "Money", "CNC", "unicode"];

use parser::CORE_IDS;

/// Read a kit's `CyJet.cyr`: one CyborgPL value, `cyjet::manifest[ … ]` (owner, 2026-10-06: "`c`. It may
/// not look as complex, but at least it's not much of a roleplay"), replacing 2026-09-29's one `const`
/// per setting. Returns its 'prefix' (none when there is no `CyJet.cyr`), its dependencies and its cores.
fn read_manifest(dir: &Path) -> Result<(Option<String>, Vec<Dep>, Vec<String>), String> {
    let file = dir.join("CyJet.cyr");
    if !file.is_file() { return Ok((None, Vec::new(), Vec::new())); }
    let shown = file.display();
    let src = std::fs::read_to_string(&file).map_err(|e| format!("{shown}: cannot read: {e}"))?;
    let toks = lexer::lex(&src).map_err(|e| format!("{shown}: line {}:{}: {}", e.line, e.col, e.msg))?;
    let one = "CyJet.cyr is one value: `cyjet::manifest[name = \"shop\", prefix = \"shop\", version = \"1.0.0\", targets = [cyjet::target[name = \"aarch64-macos\"],]];`";
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
                parser::module_name_ok(&n).map_err(|m| format!("{shown}: line {line}: `{n}` cannot name a kit: {m}"))?;
                name = true;
            }
            // the word code writes in front of `::` to reach this kit, its maker's pick (owner, 2026-09-29: b, "required")
            "prefix" => {
                let p = text(value).ok_or_else(|| format!("{shown}: line {line}: `prefix` is a text: `prefix = \"shop\"`"))?;
                parser::module_name_ok(&p).map_err(|m| format!("{shown}: line {line}: `{p}` cannot be a prefix, written in code as `{p}::`: {m}"))?;
                prefix = Some(p);
            }
            "version" => {
                let v = text(value).ok_or_else(|| format!("{shown}: line {line}: `version` is a text"))?;
                // any text, at most 256 bytes, as in Synthia.ttsr (owner, 2026-09-29: "256 bytes it is"; for CyJet.cyr: "same")
                if v.len() > 256 { return Err(format!("{shown}: line {line}: `version` is at most 256 bytes, and this one is {}", v.len())); }
                has_version = true;
            }
            "target" => return Err(format!("{shown}: line {line}: a kit builds for several targets, so the field is `targets`: `targets = [cyjet::target[name = \"aarch64-macos\"],]`")),
            // several targets, each with its settings (owner, 2026-09-29): kayo's package manager can do
            // whatever Synthia can, in its plainer way
            "targets" => {
                let items = items(value);
                let count = items.len();
                targets += count;
                let mut defaults = 0;
                let mut names = Vec::new();
                for it in items {
                    let ast::Expr::Construct(shape, fields, line) = it else { return Err(format!("{shown}: line {line}: a target is `cyjet::target[name = …]`, with a `runner` if it has one")) };
                    if shape != "::cyjet.target" { return Err(format!("{shown}: line {line}: a target is a `cyjet::target`")); }
                    if !fields.iter().any(|(f, _)| f == "name") { return Err(format!("{shown}: line {line}: a target needs its `name`")); }
                    for (f, b) in &fields {
                        // the one built when none is named (owner, 2026-09-29: b)
                        if f == "default" {
                            match b.values.as_slice() { [ast::Value::Expr(ast::Expr::Bool(true, _))] => defaults += 1, [ast::Value::Expr(ast::Expr::Bool(false, _))] => {},
                                _ => return Err(format!("{shown}: line {line}: `default` is `true` or `false`")) }
                            continue;
                        }
                        if f != "name" && f != "runner" { return Err(format!("{shown}: line {line}: a `cyjet::target` has no `{f}`; it has `name`, `runner` and `default`")); }
                        let t = text(b).ok_or_else(|| format!("{shown}: line {line}: `{f}` is a text"))?;
                        if f == "name" { names.push((t, line)); }
                    }
                }
                if count > 1 && defaults != 1 { return Err(format!("{shown}: line {line}: with several targets, exactly one says `default = true` — the one built when none is named ({defaults} do)")); }
                // only a target kayo can build (owner, 2026-09-30: "refuse", as Synthia.ttsr's "Only what's
                // actually doable"); the one list is kayo's
                let doable: Vec<&str> = include_str!("../../kayo/targets.txt").lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).collect();
                for (t, line) in names {
                    if !doable.contains(&t.as_str()) { return Err(format!("{shown}: line {line}: `{t}` is not a target kayo can build; it builds for {}", doable.iter().map(|d| format!("`{d}`")).collect::<Vec<_>>().join(", "))); }
                }
            }
            "dependencies" => {
                for it in items(value) {
                    let ast::Expr::Construct(shape, fields, line) = it else { return Err(format!("{shown}: line {line}: a dependency is `cyjet::dep[path = …]` or `cyjet::dep[id = …, version = …]`")) };
                    if shape != "::cyjet.dep" { return Err(format!("{shown}: line {line}: a dependency is a `cyjet::dep`, not `{}`", shape.trim_start_matches("::").replacen('.', "::", 1))); }
                    let mut d = Dep { what: String::new(), path: None, version: None, id: None };
                    for (f, b) in &fields {
                        // two kits with one prefix are an error, fixed by editing a kit's source (owner, 2026-09-29:
                        // "Just edit the source code"), so a dependency names no word
                        if f == "name" || f == "prefix" { return Err(format!("{shown}: line {line}: a dependency names no word: code writes the kit's own 'prefix', from its `CyJet.cyr`")); }
                        let t = text(b).ok_or_else(|| format!("{shown}: line {line}: `{f}` is a text"))?;
                        match f.as_str() { "path" => d.path = Some(t), "version" => d.version = Some(t), "id" => d.id = Some(t),
                            other => return Err(format!("{shown}: line {line}: a `cyjet::dep` has no `{other}`; it has `path`, or `id` and `version`")) }
                    }
                    d.what = match (&d.path, &d.id) { (Some(p), _) => format!("the kit at `{p}`"), (None, Some(i)) => format!("the kit `{}…`", &i[..i.len().min(8)]), _ => "the dependency".into() };
                    if d.path.is_some() == d.version.is_some() { return Err(format!("{shown}: line {line}: a dependency says where it comes from once: a `path` on this machine, or an `id` and `version` from the registry")); }
                    // the registry knows a kit by its ID, never its name (owner, 2026-09-29): names may repeat
                    // an ID is 64 hex digits wherever it is written (the owner's definition; compiler claude, 2026-09-29)
                    if let Some(i) = &d.id {
                        if i.len() != 64 || !i.bytes().all(|b| b.is_ascii_hexdigit()) { return Err(format!("{shown}: line {line}: an id is 64 hex digits, 256 random bits")); }
                    }
                    if d.version.is_some() && d.id.is_none() { return Err(format!("{shown}: line {line}: a `version` comes from the registry, which knows a kit by its ID: `id = \"…\"`, 64 hex digits")); }
                    // std is listed like any kit, by its ID and version; kayo uses the installed copy when they
                    // match (owner, 2026-10-06), and the oracle stands in for that copy
                    if d.is_std() {
                        if deps.iter().any(|x: &Dep| x.is_std()) { return Err(format!("{shown}: line {line}: std is listed twice")); }
                        if d.version.as_deref() != Some(parser::STD_VERSION) { return Err(format!("{shown}: line {line}: the oracle stands in for std `{}` only, the installed one; this asks for `{}`", parser::STD_VERSION, d.version.as_deref().unwrap_or(""))); }
                        d.what = "std".into();
                    }
                    deps.push(d);
                }
            }
            // the cores a kit is built for (owner, 2026-10-05: "Limbs should say what cores they are built
            // for"), each by its registry ID and version, or a `path` for one never registered (2026-10-06)
            // a core is a limb (owner, 2026-10-09); `cores` and `cyjet::Core` are the old spellings
            "cores" => return Err(format!("{shown}: line {line}: {}", spellings::message(spellings::old("cores")))),
            "lobes" => return Err(format!("{shown}: line {line}: {}", spellings::message(spellings::old("lobes")))),
            "plugins" => return Err(format!("{shown}: line {line}: {}", spellings::message(spellings::old("plugins")))),
            "limbs" => {
                for it in items(value) {
                    let ast::Expr::Construct(shape, fields, line) = it else { return Err(format!("{shown}: line {line}: a limb is `cyjet::limb[id = \"…\", version = \"…\"]`")) };
                    if shape != "::cyjet.limb" { return Err(format!("{shown}: line {line}: a limb is a `cyjet::limb`, not `{}`", shape.trim_start_matches("::").replacen('.', "::", 1))); }
                    let (mut id, mut version, mut cpath) = (None, None, None);
                    for (f, b) in &fields {
                        let t = text(b).ok_or_else(|| format!("{shown}: line {line}: a limb's `{f}` is a text"))?;
                        match f.as_str() {
                            "id" => id = Some(t), "version" => version = Some(t), "path" => cpath = Some(t),
                            "name" => return Err(format!("{shown}: line {line}: a limb is listed by its registry ID since 2026-10-06 (names repeat, IDs do not): `cyjet::limb[id = \"…\", version = \"…\"]`{}", CORE_IDS.iter().find(|(n, _)| *n == t).map(|(n, i)| format!("; {n}'s is `{i}`")).unwrap_or_default())),
                            other => return Err(format!("{shown}: line {line}: a `cyjet::limb` has no `{other}`; it has `id` and `version`, or a `path`")),
                        }
                    }
                    if cpath.is_some() {
                        if id.is_some() || version.is_some() { return Err(format!("{shown}: line {line}: a limb says where it comes from once: an `id` and `version` from the registry, or a `path` on this machine")); }
                        return Err(format!("{shown}: line {line}: the limb at `{}` is not registered, so not verified; loading another limb is not built yet (a Rust limb, which the oracle cannot load)", cpath.unwrap()));
                    }
                    let id = id.ok_or_else(|| format!("{shown}: line {line}: a limb needs its `id` (64 hex digits) and `version`"))?;
                    if id.len() != 64 || !id.bytes().all(|b| b.is_ascii_hexdigit()) { return Err(format!("{shown}: line {line}: an id is 64 hex digits, 256 random bits")); }
                    let version = version.ok_or_else(|| format!("{shown}: line {line}: a limb needs its `version`, as a registry kit does"))?;
                    if id == parser::MTM_ID { return Err(format!("{shown}: line {line}: MTM is gone (owner, 2026-10-07): every other spelling is refused, naming the one to write; take this limb out")); }
                    let Some((core, _)) = CORE_IDS.iter().find(|(_, i)| *i == id) else {
                        return Err(format!("{shown}: line {line}: the limb `{}…` is not an official one; loading another limb is not built yet (a Rust limb, which the oracle cannot load)", &id[..8]));
                    };
                    // ASP and Genny folded into the base (owner, 2026-10-08: "`a`" to both); their IDs no longer name a limb
                    if *core == "ASP" || *core == "Genny" { return Err(format!("{shown}: line {line}: {core} is retired: since 2026-10-08 it is part of the base; take it out of `limbs`")); }
                    // the limbs regrouped by expertise (owner, 2026-10-12): IEG and TAS dissolved into the base,
                    // WSTN, UniTect and CAPS joined into unicode
                    if *core == "IEG" || *core == "TAS" { return Err(format!("{shown}: line {line}: {core} is retired: since 2026-10-12 it is part of the base; take it out of `limbs`")); }
                    if ["WSTN", "UniTect", "CAPS"].contains(core) { return Err(format!("{shown}: line {line}: {core} is part of unicode since 2026-10-12: list `{}` instead", parser::core_line("unicode"))); }
                    let want = parser::core_version(core);
                    if version != want { return Err(format!("{shown}: line {line}: the oracle stands in for {core} `{want}` only, the installed one; this asks for `{version}`")); }
                    let core = core.to_string();
                    if cores.contains(&core) { return Err(format!("{shown}: line {line}: the limb {core} is listed twice")); }
                    cores.push(core);
                }
            }
            // the powers a kit uses, and what the program takes away (owner, 2026-10-07)
            "powers" | "deny" => {
                let mut list = Vec::new();
                for it in items(value) {
                    let ast::Expr::Text(t, _) = &it else { return Err(format!("{shown}: line {line}: a power is `cyjet::power.c_calls`, narrowed with `.` and naming things in `[ ]`")) };
                    let Some(text) = t.strip_prefix("power:") else { return Err(format!("{shown}: line {line}: a power is `cyjet::power.c_calls`, not a text")) };
                    let pw = parse_power(text).map_err(|m| format!("{shown}: line {line}: {m}"))?;
                    if f == "powers" && (!pw.except.is_empty() || !pw.for_.is_empty()) { return Err(format!("{shown}: line {line}: `.except` and `.for` belong to `deny`; `powers` lists what this kit uses")); }
                    list.push(pw);
                }
                if f == "powers" { powers_seen = list; } else { deny_seen = list; }
            }
            other => return Err(format!("{shown}: line {line}: a `cyjet::manifest` has no field `{other}`; its fields are `name`, `prefix`, `version`, `targets`, `dependencies`, `limbs`, `powers` and `deny`")),
        }
    }
    // a build needs the kit's name, version and a target; only publishing needs more (owner, 2026-09-29:
    // "yea", matching Synthia.ttsr)
    let why = "a build needs the kit's `name`, `prefix`, `version` and at least one target";
    if !name { return Err(format!("{shown}: line {mline}: the manifest has no `name`: {why}: `name = \"shop\"`")); }
    if prefix.is_none() { return Err(format!("{shown}: line {mline}: the manifest has no `prefix`, the word code writes in front of `::` to reach this kit: {why}: `prefix = \"shop\"`")); }
    if !has_version { return Err(format!("{shown}: line {mline}: the manifest has no `version`: {why}: `version = \"1.0.0\"`")); }
    if targets == 0 { return Err(format!("{shown}: line {mline}: the manifest has no target: {why}: `targets = [cyjet::target[name = \"aarch64-macos\"],]`")); }
    // std needed ASP and IEG until 2026-10-08, when the operators and ASP's number rules moved into
    // the base (owner: "`a`" to both); it needs no limb now (design claude's reading)
    // cores depend on cores (owner, 2026-10-05); MCPS "Needs IEG, not std" (owner, 2026-10-06)
    // MCPS needed IEG until 2026-10-12, when IEG was dissolved into the base
    MANIFEST_POWERS.with(|m| *m.borrow_mut() = Some((powers_seen, deny_seen)));
    Ok((prefix, deps, cores))
}

/// Load every kit `dir` depends on, and theirs, each once, under `::prefix`.
fn load_deps(dir: &Path, prog: &mut ast::Program, loaded: &mut Vec<(String, PathBuf)>, chain: &mut Vec<PathBuf>) -> Result<(), String> {
    let (_, deps, _) = read_manifest(dir)?;
    let here = dir.canonicalize().map_err(|e| format!("{}: {e}", dir.display()))?;
    chain.push(here);
    for d in deps {
        let here_file = dir.join("CyJet.cyr");
        if d.is_std() { continue; }
        let Some(rel) = &d.path else {
            return Err(format!("{}: {} comes from the registry; the oracle takes `path` dependencies only", here_file.display(), d.what));
        };
        let depdir = dir.join(rel);
        let canon = depdir.canonicalize().map_err(|_| format!("{}: a dependency is at `{rel}`, and there is no directory there", here_file.display()))?;
        if chain.contains(&canon) { return Err(format!("{}: {} depends, through its dependencies, on itself", here_file.display(), d.what)); }
        if !depdir.join("CyJet.cyr").is_file() { return Err(format!("{}: {} has no `CyJet.cyr`, so no 'prefix' for code to reach it by", here_file.display(), d.what)); }
        let (Some(prefix), _, _) = read_manifest(&depdir)? else { unreachable!() };
        // a kit may pick `std` or `cca`; it just clashes with the built-in one (owner, 2026-09-29)
        // `std` is the built-in's name since 2026-10-09; `std` (2026-10-07) and `std` are its old ones, refused in code
        if ["std", "stdkit", "cord", "cca", "kin", "cyjet"].contains(&prefix.as_str()) { return Err(format!("{}: {} has the prefix `{prefix}`, which names the built-in `{}::`; edit its 'prefix'", here_file.display(), d.what, if prefix == "cca" { "cca" } else { "std" })); }
        if let Some((_, p)) = loaded.iter().find(|(n, _)| *n == prefix) {
            if *p != canon { return Err(format!("{}: two kits have the prefix `{prefix}` ({} and {}); edit one kit's 'prefix'", here_file.display(), p.display(), canon.display())); }
            continue;
        }
        loaded.push((prefix.clone(), canon.clone()));
        let root = vec![format!("::{prefix}")];
        mark_lacks(&depdir, &root)?;
        let mut dep = load_kit(&depdir, root.clone())?;
        // a dependency's MAIN never runs: only the kit being built has one (the oracle's reading)
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

/// The power check (owner, 2026-10-07): each use is granted (the base grants the dangerous powers to
/// every kit, IEG grants vouching and the operator powers to the kits that list it), listed in the
/// using kit's `powers`, and not taken away by the program's `deny`. A kit with no `CyJet.cyr` has
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
    // C variables: a quoted name that is an extern variable of the same kit
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
    KIT_POWERS.with(|lp| {
        let lp = lp.borrow();
        let program = lp.iter().find(|l| l.root.is_empty());
        for (root, path, thing, line) in uses {
            let Some(kit) = lp.iter().find(|l| l.root == root) else { continue };
            let segs: Vec<&str> = path.split('.').collect();
            let who = if root.is_empty() { "this kit".to_string() } else { format!("`{}`", kit.prefix) };
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
            let at = if root.is_empty() { format!("line {line}") } else { format!("line {line} of `{}`", kit.prefix) };
            // 1. granted: every power here is the base's to every kit since IEG was dissolved into it
            // (owner, 2026-10-12), vouching and the operator powers included
            // 2. listed by the kit
            let covers = |p: &PowerRef| p.path.len() <= segs.len() && p.path.iter().zip(&segs).all(|(a, b)| a == b) && (p.things.is_empty() || thing.as_ref().map_or(false, |t| p.things.contains(t)));
            if !kit.powers.iter().any(covers) {
                return Err(format!("{at}: {who} {what}, and its CyJet.cyr does not list `{}`: add `cyjet::power.{}` to its `powers`", segs[0], segs[0]));
            }
            // 3. not denied by the program
            if let Some(prog_kit) = program {
                for d in &prog_kit.deny {
                    let path_ok = d.path.len() <= segs.len() && d.path.iter().zip(&segs).all(|(a, b)| a == b);
                    let things_ok = d.things.is_empty() || thing.as_ref().map_or(false, |t| d.things.contains(t));
                    let for_ok = d.for_.is_empty() || d.for_.contains(&kit.prefix);
                    let except_hit = thing.as_ref().map_or(false, |t| d.except.contains(t));
                    if path_ok && things_ok && for_ok && !except_hit {
                        return Err(format!("{at}: {who} {what}, which this program denies (`deny`: `cyjet::power.{}`)", d.path.join(".")));
                    }
                }
            }
        }
        Ok(())
    })
}

/// Every `pub` thing says how other kits write its path, on itself, its module or its kind, and the
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

/// A shift with no width anywhere is refused in code that runs or not (check claude, 2026-10-16, corpus
/// dfce8e06: the oracle refused it only where it ran, as kayo refuses it when building). A shift's width comes
/// from its left operand or where the value goes (operators.md, 2026-09-23), never from the count (4866c834);
/// this finds the cases where neither can give one: the shifted side is plain numbers, and it sits in a
/// comparison whose other side is plain numbers too, an untyped `let`, a conversion or std's printing.
fn check_widthless_shifts(prog: &ast::Program) -> Result<(), String> {
    use ast::{Expr, Stmt, Handler, Value};
    fn plain(e: &Expr) -> bool {
        match e {
            Expr::Int(..) | Expr::RawInt(..) | Expr::BigInt(..) => true,
            Expr::Paren(i, _) | Expr::Unary(_, i, _) => plain(i),
            Expr::Binary(_, l, r, _) => plain(l) && plain(r),
            _ => false,
        }
    }
    // a shift of plain numbers, reached through what passes the context along
    fn bare_shift(e: &Expr) -> Option<(String, usize)> {
        match e {
            Expr::Binary(op, l, _, line) if op.starts_with("shl") || op.starts_with("shr") => if plain(l) { Some((op.clone(), *line)) } else { None },
            Expr::Paren(i, _) | Expr::Unary(_, i, _) => bare_shift(i),
            Expr::Binary(op, l, r, _) if !is_cmp(op) => bare_shift(l).or_else(|| bare_shift(r)),
            _ => None,
        }
    }
    fn is_cmp(op: &str) -> bool { matches!(op, "equal_to" | "not_equal" | "<" | ">" | "at_most" | "at_least") }
    fn refuse((op, line): (String, usize)) -> Result<(), String> {
        let op = op.split('.').next().unwrap_or("shl").to_string();
        Err(format!("line {line}: `{op}` needs a width, and nothing here gives one: declare where the value goes (`let 'v': uint32 = 1 {op} 3;`) or write `to.uint32[1] {op} 3`"))
    }
    // a written position below 1 in std's text calls is refused when building (owner, 2026-10-17: "(a)")
    fn positions(c: &ast::Call) -> Result<(), String> {
        let p = c.path.join(".");
        if c.kit.as_deref() == Some("std") && (p.starts_with("str.at.") || p.starts_with("str.slice.")) {
            for Value::Expr(v) in c.args.values.iter().skip(1) {
                let lit = match v { Expr::Int(n, _) => Some(*n), Expr::Unary(op, inner, _) if op == "-" => match &**inner { Expr::Int(n, _) => Some(-*n), _ => None }, _ => None };
                if let Some(n) = lit { if n < 1 { return Err(format!("line {}: {n} is no position: a position is `uint64[min 1]`, counting from 1 (owner, 2026-10-17)", c.line)); } }
            }
        }
        Ok(())
    }
    fn printing(c: &ast::Call) -> Result<(), String> {
        if c.kit.as_deref() == Some("std") && c.path.first().map(|p| p.as_str()) == Some("print") {
            for Value::Expr(v) in &c.args.values { if let Some(s) = bare_shift(v) { return refuse(s); } }
        }
        Ok(())
    }
    fn bracket(b: &ast::Bracket) -> Result<(), String> { for Value::Expr(e) in &b.values { expr(e)?; } Ok(()) }
    fn handler(h: &Option<Handler>) -> Result<(), String> {
        match h {
            Some(Handler::Value(b)) => bracket(b),
            Some(Handler::Leave(st)) => stmt(st),
            Some(Handler::Arms(a)) => arms(a),
            _ => Ok(()),
        }
    }
    fn arms(a: &[ast::Arm]) -> Result<(), String> { for arm in a { if let Some(v) = &arm.value { expr(v)?; } body(&arm.body)?; } Ok(()) }
    fn body(b: &[Stmt]) -> Result<(), String> { for st in b { stmt(st)?; } Ok(()) }
    fn expr(e: &Expr) -> Result<(), String> {
        if let Expr::Binary(op, l, r, _) = e {
            if is_cmp(op) {
                if plain(r) { if let Some(s) = bare_shift(l) { return refuse(s); } }
                if plain(l) { if let Some(s) = bare_shift(r) { return refuse(s); } }
            }
        }
        // a conversion and std's printing give what is inside them no width (check claude, 2026-10-16, corpus
        // 909a4a18): `to.int64[5 shr …]` and `std::print.useful[5 shr …]`
        match e {
            Expr::Convert(_, b, _, _, _) => for Value::Expr(v) in &b.values { if let Some(s) = bare_shift(v) { return refuse(s); } },
            Expr::Call(c) => { printing(c)?; positions(c)? }
            _ => {}
        }
        match e {
            Expr::Call(c) => { bracket(&c.args)?; handler(&c.handler) }
            Expr::Nested(b) | Expr::PtrCast(_, b, _) => bracket(b),
            Expr::Index(_, i, _) | Expr::FieldIndex(_, _, i, _) | Expr::OwnedElem(_, _, i, _) | Expr::Paren(i, _) | Expr::Unary(_, i, _) => expr(i),
            Expr::Builtin(_, _, b, _) => bracket(b),
            Expr::Construct(_, fields, _) => { for (_, b) in fields { bracket(b)?; } Ok(()) }
            Expr::Convert(_, b, h, _, _) | Expr::MapFail(_, b, h, _) | Expr::PtrReadChecked(b, h, _) | Expr::CharFromCode(_, b, h, _) | Expr::CallValue(_, b, h, _) => { bracket(b)?; handler(h) }
            Expr::MapLit(items, _) => { for (k, v) in items { expr(k)?; bracket(v)?; } Ok(()) }
            Expr::Match(v, a, _) => { bracket(v)?; arms(a) }
            Expr::Lambda(f, _, _) => body(&f.body),
            Expr::EnumValue(_, _, Some(b), _) => bracket(b),
            Expr::Binary(_, l, r, _) => { expr(l)?; expr(r) }
            _ => Ok(()),
        }
    }
    fn stmt(st: &Stmt) -> Result<(), String> {
        match st {
            Stmt::Decl { ty: None, value, .. } | Stmt::LocalConst { ty: None, value, .. } => {
                if let [Value::Expr(e)] = value.values.as_slice() { if let Some(s) = bare_shift(e) { return refuse(s); } }
                bracket(value)
            }
            Stmt::Decl { value, .. } | Stmt::LocalConst { value, .. } | Stmt::Assign { value, .. } | Stmt::AssignField { value, .. } | Stmt::Gives { value, .. } => bracket(value),
            Stmt::AssignElem { index, value, .. } => { expr(index)?; bracket(value) }
            Stmt::If { arms: a, otherwise, .. } => { for (c, b) in a { bracket(c)?; body(b)?; } if let Some(o) = otherwise { body(o)?; } Ok(()) }
            Stmt::Loop { body: b, .. } | Stmt::Block { body: b, .. } => body(b),
            Stmt::While { cond, body: b, .. } => { bracket(cond)?; body(b) }
            Stmt::For { over, body: b, .. } => { bracket(over)?; body(b) }
            Stmt::Range { bounds, body: b, .. } => { bracket(bounds)?; body(b) }
            Stmt::Return { value: Some(v), .. } => bracket(v),
            Stmt::Call { call, .. } => { printing(call)?; positions(call)?; bracket(&call.args)?; handler(&call.handler) }
            Stmt::BuiltinStmt { args, .. } => bracket(args),
            Stmt::Tier { body: b, .. } => stmt(b),
            Stmt::Fail { payload: Some(p), .. } => bracket(p),
            Stmt::Match { value, arms: a, .. } => { bracket(value)?; arms(a) }
            Stmt::Must { cond: e, .. } | Stmt::MustFail { expr: e, .. } | Stmt::MustStop { expr: e, .. } => expr(e),
            Stmt::Concur { body: b, handler: h, .. } => { body(b)?; handler(h) }
            Stmt::Whichever { arms: a, handler: h, .. } => { for arm in a { bracket(&arm.chan)?; body(&arm.body)?; } handler(h) }
            Stmt::MapFailStmt { args, handler: h, .. } | Stmt::CallValue { args, handler: h, .. } => { bracket(args)?; handler(h) }
            _ => Ok(()),
        }
    }
    for f in &prog.funcs { body(&f.body)?; }
    for c in &prog.consts { bracket(&c.value)?; }
    if let Some(m) = &prog.main { body(m)?; }
    Ok(())
}

/// A plain `let` over a visible name is refused, and `shadow` needs one to shadow, in code that runs
/// or not (check claude, 2026-10-08: the oracle checked only where it ran; owner, 2026-09-14 and
/// 2026-10-08). Scopes as the interpreter keeps them: a function's parameters, then each block;
/// a closure starts afresh with its parameters and captures.
///
/// The same walk checks two promises in code that runs or not (check claude, 2026-10-10): a const is
/// never changed (`change 'r' = …`), and a `mut` variable or parameter is changed by something (owner,
/// 2026-10-10: "`b`"; in a generic function nobody calls too: "`a`"). A change is `change` (whole,
/// field or element), `lend.mut`, a `lend.mut` capture, or an `inline.ASM` output or `lend.mut`, and it
/// counts for the variable its name means where it is written, so a shadow's change is not the
/// shadowed variable's. Hands back the first broken `mut` promise, reported after the other checks.
fn check_redeclared(prog: &ast::Program) -> Result<Option<String>, String> {
    use ast::{Stmt, Expr, Handler, AsmBind, Mutability, Verb};
    type Entry = (String, bool, Option<usize>);   // (name, sealed, which `mut` it is)
    struct W { sc: Vec<Vec<Entry>>, muts: Vec<(String, usize, bool)> }   // (name, line, changed)
    // a const sits in the outermost scope, its name marked `const:`
    fn find<'a>(sc: &'a [Vec<Entry>], n: &str) -> Option<&'a Entry> { sc.iter().rev().flat_map(|s| s.iter().rev()).find(|(m, _, _)| m == n || m.strip_prefix("const:") == Some(n)) }
    fn decl(w: &mut W, n: &str, shadow: bool, sealed: bool, is_mut: bool, line: usize) -> Result<(), String> {
        match (find(&w.sc, n), shadow) {
            (Some(_), false) => return Err(format!("line {line}: '{n}' is already declared in an enclosing scope; to shadow it deliberately write `shadow '{n}' := …;` (owner, 2026-10-08)")),
            (None, true) => return Err(format!("line {line}: `shadow` declares over an existing '{n}', but none is visible here")),
            (Some((m, true, _)), true) if m.starts_with("const:") => return Err(format!("line {line}: '{n}' is a const and cannot be shadowed")),
            (Some((_, true, _)), true) => return Err(format!("line {line}: '{n}' cannot be shadowed: it is `no_shadow` (or a loop's own counter)")),
            _ => {}
        }
        let m = if is_mut { w.muts.push((n.to_string(), line, false)); Some(w.muts.len() - 1) } else { None };
        w.sc.last_mut().unwrap().push((n.to_string(), sealed, m));
        Ok(())
    }
    // something changes the variable `n` means here
    fn changes(w: &mut W, n: &str) {
        if let Some(i) = find(&w.sc, n).and_then(|e| e.2) { w.muts[i].2 = true; }
    }
    // a `lend.mut` in text the walk does not take apart further
    fn lends_mut(w: &mut W, text: &str) {
        // the name as `{:?}` wrote it, its `"` and `\` escaped (check claude, 2026-10-16, corpus 65f59c7a: a
        // name holding either was cut short), and what follows its closing quote
        fn debug_str(rest: &str) -> Option<(String, &str)> {
            let mut out = String::new();
            let mut it = rest.char_indices();
            while let Some((k, c)) = it.next() {
                match c {
                    '"' => return Some((out, &rest[k + 1..])),
                    '\\' => match it.next() {
                        Some((_, 'n')) => out.push('\n'), Some((_, 't')) => out.push('\t'), Some((_, 'r')) => out.push('\r'),
                        Some((_, '0')) => out.push('\0'),
                        Some((_, e)) => out.push(e),
                        None => return None,
                    },
                    c => out.push(c),
                }
            }
            None
        }
        for pat in ["Owned(LendMut, \"", "OwnedField(LendMut, \"", "OwnedElem(LendMut, \""] {
            for rest in text.split(pat).skip(1) { if let Some((n, _)) = debug_str(rest) { changes(w, &n); } }
        }
        for rest in text.split("Spread(\"").skip(1) {
            if let Some((n, after)) = debug_str(rest) { if after.starts_with(", Some(LendMut)") { changes(w, &n); } }
        }
    }
    // the bodies inside an expression that the walk visits in their own scopes
    fn nested(e: &Expr, out: &mut Vec<String>) {
        fn of_handler(h: &Option<Handler>, out: &mut Vec<String>) {
            match h {
                Some(Handler::Arms(a)) => for arm in a { out.push(format!("{:?}", arm.body)); if let Some(v) = &arm.value { nested(v, out); } },
                Some(Handler::Leave(st)) => out.push(format!("{st:?}")),
                Some(Handler::Value(b)) => for ast::Value::Expr(e) in &b.values { nested(e, out); },
                _ => {}
            }
        }
        match e {
            Expr::Call(c) => { for ast::Value::Expr(e) in &c.args.values { nested(e, out); } of_handler(&c.handler, out); }
            Expr::Match(v, a, _) => { for ast::Value::Expr(e) in &v.values { nested(e, out); } for arm in a { out.push(format!("{:?}", arm.body)); if let Some(v) = &arm.value { nested(v, out); } } }
            Expr::Lambda(f, _, _) => out.push(format!("{:?}", f.body)),
            _ => {}
        }
    }
    fn block(w: &mut W, body: &[Stmt], extra: &[Entry]) -> Result<(), String> {
        w.sc.push(extra.to_vec());
        for st in body { stmt(w, st)?; }
        w.sc.pop();
        Ok(())
    }
    fn arms(w: &mut W, arms: &[ast::Arm]) -> Result<(), String> {
        for a in arms {
            if let Some(v) = &a.value { expr(w, v)?; }
            block(w, &a.body, &a.bind.iter().map(|b| (b.clone(), false, None)).collect::<Vec<_>>())?;
        }
        Ok(())
    }
    fn handler(w: &mut W, h: &Option<Handler>) -> Result<(), String> {
        match h { Some(Handler::Arms(a)) => arms(w, a), Some(Handler::Leave(st)) => stmt(w, st), Some(Handler::Value(b)) => bracket(w, b), _ => Ok(()) }
    }
    fn bracket(w: &mut W, b: &ast::Bracket) -> Result<(), String> {
        let mut text = format!("{b:?}");
        let mut bodies = Vec::new();
        for ast::Value::Expr(e) in &b.values { nested(e, &mut bodies); }
        for body in bodies { text = text.replace(&body, ""); }
        lends_mut(w, &text);
        for ast::Value::Expr(e) in &b.values { expr(w, e)?; }
        Ok(())
    }
    fn expr(w: &mut W, e: &Expr) -> Result<(), String> {
        match e {
            Expr::Call(c) => { bracket(w, &c.args)?; handler(w, &c.handler) }
            Expr::Match(v, a, _) => { bracket(w, v)?; arms(w, a) }
            Expr::Lambda(f, caps, _) => {
                for (verb, n) in caps { if *verb == Verb::LendMut { changes(w, n); } }
                let fresh = vec![caps.iter().map(|(_, n)| (n.clone(), false, None)).collect()];
                let outer = std::mem::replace(&mut w.sc, fresh);
                for p in &f.params {
                    let m = if p.mutability == Some(Mutability::Mut) { w.muts.push((p.name.clone(), f.line, false)); Some(w.muts.len() - 1) } else { None };
                    w.sc.last_mut().unwrap().push((p.name.clone(), false, m));
                }
                let r = block(w, &f.body, &[]);
                w.sc = outer;
                r
            }
            _ => Ok(()),
        }
    }
    fn stmt(w: &mut W, st: &Stmt) -> Result<(), String> {
        match st {
            Stmt::Decl { name, mutability, value, shadow, sealed, line, .. } => { bracket(w, value)?; decl(w, name, *shadow, *sealed, *mutability == Some(Mutability::Mut), *line) }
            Stmt::LocalConst { name, value, line, .. } => {
                bracket(w, value)?;
                decl(w, name, false, true, false, *line)?;
                // marked as a const, so a later `shadow` or `change` names it as one
                if let Some(last) = w.sc.last_mut().unwrap().last_mut() { last.0 = format!("const:{name}"); }
                Ok(())
            }
            Stmt::Assign { name, value, line } => {
                bracket(w, value)?;
                if find(&w.sc, name).is_some_and(|(m, _, _)| m.starts_with("const:")) { return Err(format!("line {line}: '{name}' is a const and cannot be assigned to")); }
                changes(w, name);
                Ok(())
            }
            Stmt::AssignElem { name, index, value, .. } => { lends_mut(w, &format!("{index:?}")); expr(w, index)?; bracket(w, value)?; changes(w, name); Ok(()) }
            Stmt::AssignField { name, value, .. } => { bracket(w, value)?; changes(w, name); Ok(()) }
            Stmt::If { arms: a, otherwise, .. } => { for (c, b) in a { bracket(w, c)?; block(w, b, &[])?; } if let Some(o) = otherwise { block(w, o, &[])?; } Ok(()) }
            Stmt::Loop { body, .. } | Stmt::Block { body, .. } => block(w, body, &[]),
            Stmt::While { cond, body, .. } => { bracket(w, cond)?; block(w, body, &[]) }
            Stmt::For { name, index, over, body, .. } => {
                bracket(w, over)?;
                let mut names = vec![(name.clone(), true, None)];
                if let Some(i) = index { names.push((i.clone(), true, None)); }
                block(w, body, &names)
            }
            Stmt::Range { name, bounds, body, .. } => { bracket(w, bounds)?; block(w, body, &[(name.clone(), true, None)]) }
            Stmt::Tier { body, .. } => stmt(w, body),
            Stmt::Match { value, arms: a, .. } => { bracket(w, value)?; arms(w, a) }
            Stmt::Call { call, .. } => { bracket(w, &call.args)?; handler(w, &call.handler) }
            Stmt::CallValue { args, handler: h, .. } | Stmt::MapFailStmt { args, handler: h, .. } => { bracket(w, args)?; handler(w, h) }
            Stmt::Concur { caps, body, handler: h, .. } => {
                for (verb, n) in caps { if *verb == Verb::LendMut { changes(w, n); } }
                block(w, body, &[])?;
                handler(w, h)
            }
            Stmt::Whichever { arms: a, handler: h, .. } => { for x in a { bracket(w, &x.chan)?; block(w, &x.body, &[(x.name.clone(), false, None)])?; } handler(w, h) }
            Stmt::Asm { binds, line, .. } => {
                for b in binds {
                    match b {
                        AsmBind::Out(v, _) => if find(&w.sc, v).is_none() { decl(w, v, false, false, false, *line)?; } else { changes(w, v); },
                        AsmBind::InOut(v, _) | AsmBind::Lend(v, _, true) => changes(w, v),
                        _ => {}
                    }
                }
                Ok(())
            }
            _ => { lends_mut(w, &format!("{st:?}")); Ok(()) }
        }
    }
    let consts = |module: &[String]| -> Vec<Entry> { prog.consts.iter().filter(|c| c.module == module).map(|c| (format!("const:{}", c.name), true, None)).collect() };
    let mut unkept = None;
    let mut walk = |sc: Vec<Vec<Entry>>, params: &[ast::Param], fline: usize, body: &[Stmt]| -> Result<(), String> {
        let mut w = W { sc, muts: Vec::new() };
        for p in params {
            let m = if p.mutability == Some(Mutability::Mut) { w.muts.push((p.name.clone(), fline, false)); Some(w.muts.len() - 1) } else { None };
            w.sc.last_mut().unwrap().push((p.name.clone(), false, m));
        }
        block(&mut w, body, &[])?;
        if unkept.is_none() {
            if let Some((n, line, _)) = w.muts.iter().find(|m| !m.2) {
                unkept = Some(format!("line {line}: '{n}' is `mut`, but nothing changes it: drop `mut`, or change it (`mut` is a promise that it changes, as `immut` is that it does not)"));
            }
        }
        Ok(())
    };
    for f in &prog.funcs { walk(vec![consts(&f.module), Vec::new()], &f.params, f.line, &f.body)?; }
    if let Some(m) = &prog.main {
        let args: Vec<Entry> = prog.main_args.iter().flat_map(|(a, b)| [(a.clone(), false, None), (b.clone(), false, None)]).collect();
        walk(vec![consts(&[]), args], &[], 0, m)?;
    }
    Ok(unkept)
}

/// A C struct CyborgPL never looks inside (owner, 2026-10-10: `ptr.mut[FILE]`) is only pointed at: held
/// anywhere else (a parameter, a return, a field, a variable, a list's element) it is refused, in code
/// that runs or not, since holding it needs its size, which is C's own (compiler claude's reading).
fn check_opaque_held(prog: &ast::Program) -> Result<(), String> {
    use ast::{Stmt, Ty};
    let opaque: Vec<&str> = prog.structs.iter().filter(|d| d.opaque).map(|d| d.name.as_str()).collect();
    if opaque.is_empty() { return Ok(()); }
    fn held<'a>(t: &'a Ty, opaque: &[&str]) -> Option<&'a str> {
        match t {
            Ty::Struct(n) if opaque.contains(&n.as_str()) => Some(n),
            Ty::Ptr(..) => None,
            Ty::List(e) | Ty::Array(e) | Ty::Channel(e) | Ty::Simd(e, _) | Ty::Ranged(e, _, _) => held(e, opaque),
            Ty::Map(k, v, _) => held(k, opaque).or_else(|| held(v, opaque)),
            Ty::Inst(_, ts) => ts.iter().find_map(|t| held(t, opaque)),
            Ty::Fn(ps, r, _) => ps.iter().find_map(|(_, t)| held(t, opaque)).or_else(|| r.as_deref().and_then(|r| held(r, opaque))),
            _ => None,
        }
    }
    let msg = |n: &str, line: usize| format!("line {line}: `{n}` is C's and opaque (owner, 2026-10-10): it is only pointed at, `ptr.mut[{n}]`, never held");
    fn body(stmts: &[Stmt], opaque: &[&str], out: &mut Option<(String, usize)>) {
        for st in stmts {
            if out.is_some() { return; }
            match st {
                Stmt::Decl { ty: Some(t), line, .. } | Stmt::LocalConst { ty: Some(t), line, .. } | Stmt::Range { ty: Some(t), line, .. } => {
                    if let Some(n) = held(t, opaque) { *out = Some((n.to_string(), *line)); }
                }
                Stmt::If { arms, otherwise, .. } => { for (_, b) in arms { body(b, opaque, out); } if let Some(o) = otherwise { body(o, opaque, out); } }
                Stmt::Loop { body: b, .. } | Stmt::Block { body: b, .. } | Stmt::While { body: b, .. } | Stmt::For { body: b, .. } | Stmt::Concur { body: b, .. } => body(b, opaque, out),
                Stmt::Tier { body: b, .. } => body(std::slice::from_ref(&**b), opaque, out),
                Stmt::Match { arms, .. } => { for a in arms { body(&a.body, opaque, out); } }
                Stmt::Whichever { arms, .. } => { for a in arms { body(&a.body, opaque, out); } }
                _ => {}
            }
        }
    }
    for f in &prog.funcs {
        for p in &f.params { if let Some(n) = p.ty.as_ref().and_then(|t| held(t, &opaque)) { return Err(msg(n, f.line)); } }
        if let Some(n) = f.ret.as_ref().and_then(|t| held(t, &opaque)) { return Err(msg(n, f.line)); }
        let mut out = None;
        body(&f.body, &opaque, &mut out);
        if let Some((n, line)) = out { return Err(msg(&n, line)); }
    }
    for d in &prog.structs {
        for fd in &d.fields { if let Some(n) = held(&fd.ty, &opaque) { return Err(msg(n, d.line)); } }
    }
    for c in &prog.consts { if let Some(n) = c.ty.as_ref().and_then(|t| held(t, &opaque)) { return Err(msg(n, c.line)); } }
    for x in &prog.externs { if let Some(n) = held(&x.ty, &opaque) { return Err(msg(n, x.line)); } }
    if let Some(m) = &prog.main {
        let mut out = None;
        body(m, &opaque, &mut out);
        if let Some((n, line)) = out { return Err(msg(&n, line)); }
    }
    Ok(())
}

/// A function's `return` of the wrong kind is refused in code that never runs too (check claude, seed
/// 44056: the oracle checked a return only where it ran), as a `let` over a name is (caba9b84). Judged
/// here: a literal, or a name whose type is known from its declaration (a parameter, `let` / `shadow`
/// with a type, or `:=` of a literal), when its kind (number, `bool`, text) cannot be the return type's;
/// the interpreter checks the rest as it runs.
fn check_return_literals(prog: &ast::Program) -> Result<(), String> {
    use ast::{Expr, Stmt, Ty};
    #[derive(Clone, Copy, PartialEq)]
    enum Kind { Number, Bool, Text }
    fn of_ty(t: &Ty) -> Option<Kind> {
        match t {
            Ty::Int(_) | Ty::Uint(_) | Ty::Bin(_) | Ty::Brain | Ty::Deci(_) => Some(Kind::Number),
            Ty::Bool => Some(Kind::Bool),
            Ty::Str(_) => Some(Kind::Text),
            _ => None,
        }
    }
    fn of_lit(e: &Expr) -> Option<Kind> {
        match e {
            Expr::Int(..) | Expr::Bin(..) | Expr::Exact(..) => Some(Kind::Number),
            Expr::Bool(..) => Some(Kind::Bool),
            Expr::Text(..) => Some(Kind::Text),
            _ => None,
        }
    }
    fn shown(k: Kind) -> &'static str { match k { Kind::Number => "a number", Kind::Bool => "a `bool`", Kind::Text => "a text" } }
    type Scopes = Vec<Vec<(String, Option<Kind>)>>;
    fn find(sc: &Scopes, n: &str) -> Option<Kind> { sc.iter().rev().flat_map(|s| s.iter().rev()).find(|(m, _)| m == n).and_then(|(_, k)| *k) }
    fn walk(sc: &mut Scopes, body: &[Stmt], want: Kind, rt: &Ty, f: &str) -> Result<(), String> {
        sc.push(Vec::new());
        let r = (|| {
            for st in body {
                match st {
                    Stmt::Decl { name, ty, value, .. } => {
                        let k = match ty { Some(t) => of_ty(t), None => match value.values.as_slice() { [ast::Value::Expr(e)] => of_lit(e), _ => None } };
                        sc.last_mut().unwrap().push((name.clone(), k));
                    }
                    Stmt::Return { value: Some(b), line } => {
                        if let [ast::Value::Expr(e)] = b.values.as_slice() {
                            let (k, what) = match e {
                                Expr::Var(n, _) => (find(sc, n), format!("'{n}', which is ")),
                                other => (of_lit(other), String::new()),
                            };
                            if let Some(k) = k { if k != want {
                                return Err(format!("line {line}: `{f}` returns {}, but this `return` gives {what}{}", value::ty_name(rt), shown(k)));
                            } }
                        }
                    }
                    Stmt::If { arms, otherwise, .. } => { for (_, b) in arms { walk(sc, b, want, rt, f)?; } if let Some(o) = otherwise { walk(sc, o, want, rt, f)?; } }
                    Stmt::Loop { body, .. } | Stmt::Block { body, .. } | Stmt::While { body, .. } | Stmt::For { body, .. } | Stmt::Range { body, .. } => walk(sc, body, want, rt, f)?,
                    Stmt::Tier { body, .. } => walk(sc, std::slice::from_ref(&**body), want, rt, f)?,
                    Stmt::Match { arms, .. } => { for a in arms { walk(sc, &a.body, want, rt, f)?; } }
                    _ => {}
                }
            }
            Ok(())
        })();
        sc.pop();
        r
    }
    for func in &prog.funcs {
        let Some(t) = &func.ret else { continue };
        let Some(want) = of_ty(t) else { continue };
        let mut sc: Scopes = vec![func.params.iter().map(|p| (p.name.clone(), p.ty.as_ref().and_then(of_ty))).collect()];
        walk(&mut sc, &func.body, want, t, &func.name)?;
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
        // `shapes.area` in this kit, `areas::shapes.area` in a dependency
        let last = name.rsplit('.').next().unwrap_or(name);
        let mut segs: Vec<String> = module.iter().map(|m| m.trim_start_matches("::").to_string()).collect();
        segs.push(last.to_string());
        let shown = if root.is_empty() { segs.join(".") } else { format!("{}::{}", segs[0], segs[1..].join(".")) };
        let covering: Vec<&ast::BareMark> = prog.bare_marks.iter().filter(|m| match &m.target {
            BareTarget::Item(k, n) => k == kind && *n == module.iter().map(String::as_str).chain(std::iter::once(last)).collect::<Vec<_>>().join("."),
            BareTarget::Block(p) => module.starts_with(p),
            BareTarget::File(p) => module.starts_with(p) && (p.len()..module.len()).all(|i| block(&module[..=i])),
            BareTarget::Kind(k) => k == kind && m.kit_root == root,
        }).collect();
        let Some(first) = covering.first() else {
            return Err(format!("line {line}: `{kind} {shown}` is `pub`, and nothing says how other kits write its path: mark it (`pub bare.never {kind} …`), its module, or its kind in `kit.cy` (`bare.never [{kind}];`) as `bare.never` (path required), `bare.optional` or `bare.only` (path forbidden); there is no default"));
        };
        if let Some(other) = covering.iter().find(|m| m.mode != first.mode) {
            return Err(format!("line {line}: `{kind} {shown}` is marked both `{}` ({}) and `{}` ({}): marks that disagree are an error", word(first.mode), crate::interp::line_of(first.line), word(other.mode), crate::interp::line_of(other.line)));
        }
        marks.insert(interp::bare_key(kind, &module, last), first.mode);
    }
    // two kits forbidding the path to one name clash as soon as both are depended on (owner,
    // 2026-10-06: "`a`"), fixed as a prefix clash is, by editing one kit's source
    let mut only: Vec<(String, String)> = Vec::new();
    for (key, mode) in &marks {
        if *mode != Bare::Only { continue; }
        let Some((kind, path)) = key.split_once(':') else { continue };
        let Some(rest) = path.strip_prefix("::") else { continue };
        let (root, rel) = rest.split_once('.').unwrap_or((rest, ""));
        let rel = format!("{kind} {rel}");
        if let Some((other, _)) = only.iter().find(|(r, x)| *x == rel && r != root) {
            let mut two = [other.clone(), root.to_string()]; two.sort();
            return Err(format!("`{}` and `{}` both forbid the path to `{rel}` (`bare.only`), so it cannot tell them apart: edit one kit's source, as for a prefix clash", two[0], two[1]));
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
        } else if let Some(new) = renamed_file(&name) {
            return Err(format!("{}: `{name}` has the old name: rename it to `{new}`", p.display()));
        } else if let Some(stem) = name.strip_suffix(".cy") {
            if prefix.is_empty() && stem == "kit" { continue; }
            // a kit's `build.cy` is its build program, run by `kayo prepare`, not a module (owner, 2026-10-09: "yea")
            if prefix.is_empty() && stem == "build" { continue; }
            if stem == "kit" { return Err(format!("{}: `kit` names the root; a module cannot be called that", p.display())); }
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

/// `tests`: run every `test fn` instead of MAIN (`kayo test`, owner 2026-09-29).
pub fn run_path_as(path: &Path, out: Box<dyn std::io::Write>, tests: bool) -> Result<u8, String> {
    run_with_args(path, out, tests, Vec::new())
}

/// `args`: the words after `--`, which MAIN reads as C's argc/argv (owner, 2026-09-29).
pub fn run_with_args(path: &Path, out: Box<dyn std::io::Write>, tests: bool, args: Vec<String>) -> Result<u8, String> {
    interp::set_shown_file(&if path.is_dir() { "kit.cy".to_string() } else { path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default() });
    let mut prog = load(path)?;
    // what `kayo generate` wrote must still be what the templates make (owner, 2026-10-13)
    if GENERATE.with(|g| g.get()) {
        let n = generate::write(&mut prog)?;
        println!("generated {n} part(s)");
        return Ok(0);
    }
    generate::check(&mut prog)?;
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
    // `oracle --check-spellings`: `spellings.cyr` checked against itself and the naming rule (owner, 2026-10-13)
    if matches!(head, [_, flag] if flag == "--check-spellings") {
        let bad = spellings::check();
        for b in &bad { eprintln!("spellings.cyr: {b}"); }
        if bad.is_empty() { println!("spellings.cyr: fine"); }
        exit(if bad.is_empty() { 0 } else { 1 });
    }
    let (tests, target) = match head {
        [_, t] if t != "--check-spellings" => (false, t),
        [_, flag, t] if flag == "--test" => (true, t),
        [_, flag, t] if flag == "--generate" => { GENERATE.with(|g| g.set(true)); (false, t) }
        _ => { eprintln!("usage: oracle [--test | --generate] <file.cy | kit-directory> [-- program arguments], or oracle --check-spellings"); exit(2); }
    };
    // the parser and the interpreter recurse, so a call nested a few thousand deep in one expression
    // needs more than the main thread's 8 MB (check claude, seed 32299): 1 GB, only what is used is touched
    let target = target.clone();
    let gen = GENERATE.with(|g| g.get());
    let run = std::thread::Builder::new().stack_size(1 << 30).spawn(move || { GENERATE.with(|g| g.set(gen)); run_with_args(Path::new(&target), Box::new(std::io::stdout()), tests, rest) })
        .expect("the oracle's thread");
    match run.join() {
        Ok(Ok(code)) => exit(code as i32),
        Ok(Err(msg)) => { eprintln!("{msg}"); exit(1); }
        Err(_) => exit(101),
    };
}
