//! Loading a Limb: one `.cpls` file, or a directory whose `limb.cpls` is
//! the root and whose other files and directories are its modules
//! (`limbs.md`).

use std::path::{Path, PathBuf};

use crate::ast::Vis;
use crate::diag::{err, Diagnostic, Result};
use crate::resolve::{Items, ModuleId};
use crate::source::{FileId, SourceMap};

pub fn load(path: &Path, sources: &mut SourceMap) -> Result<Items> {
    let mut items = Items::default();
    if path.is_dir() {
        let mut state = Loading { stack: Vec::new(), done: std::collections::HashMap::new() };
        load_limb(sources, &mut items, path, None, &mut state)?;
    } else {
        let m = new_module(&mut items, "limb", None, Vis::Pub);
        let file = parse_file(sources, path)?;
        items.collect_file(&file, m, true)?;
    }
    items.resolve_all()?;
    Ok(items)
}

/// Where loading has got to: the Limbs being loaded now (a dependency back
/// to one of them is a cycle), and those loaded already (two Limbs that
/// share a dependency share one copy of it).
struct Loading {
    stack: Vec<(PathBuf, String)>,
    done: std::collections::HashMap<PathBuf, ModuleId>,
}

/// One Limb directory and, through its `CCA.cplr`, its dependencies: each
/// a tree of its own, its root's `limb_deps` naming theirs. `as_dep` is the
/// name a depending Limb gives it; the Limb being built has none, and its
/// root is `ROOT`, made first.
fn load_limb(sources: &mut SourceMap, items: &mut Items, dir: &Path, as_dep: Option<&str>, state: &mut Loading) -> Result<ModuleId> {
    let key = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    if let Some(r) = state.done.get(&key) {
        return Ok(*r);
    }
    let shown = match as_dep {
        Some(n) => n.to_string(),
        None => dir.canonicalize().ok().and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string())).unwrap_or_else(|| "this Limb".into()),
    };
    if let Some(at) = state.stack.iter().position(|(p, _)| *p == key) {
        let mut names: Vec<String> = state.stack[at..].iter().map(|(_, n)| n.clone()).collect();
        names.push(shown);
        return err(Diagnostic::error(format!("the dependencies go round in a circle: {} — a Limb cannot depend on itself, however far round", names.join(" → "))));
    }
    let root = dir.join("limb.cpls");
    if !root.is_file() {
        return err(Diagnostic::error(match as_dep {
            None => format!("`{}` is a Limb directory, and a Limb needs its root file `limb.cpls`: none is there", short(dir)),
            Some(n) => format!("the dependency `{}` is at `{}`, and a Limb needs its root file `limb.cpls`: none is there", n, dir.display()),
        }));
    }
    let manifest = read_manifest(sources, dir)?;
    if let (Some(want), Some((have, span))) = (as_dep, &manifest.name) {
        if want != have {
            return err(Diagnostic::error(format!("this Limb calls itself `{}` in its `CCA.cplr`, and the Limb that depends on it names it `{}`: the names must agree", have, want)).at(*span));
        }
    }
    let m = match as_dep {
        None => new_module(items, "limb", None, Vis::Pub),
        Some(n) => {
            let id = ModuleId(items.modules.len() as u32);
            items.modules.push(crate::resolve::Module { name: "limb".to_string(), path: format!("{}::limb", n), parent: None, vis: Vis::Pub, items: Default::default(), consts: Default::default() });
            id
        }
    };
    let mut file = parse_file(sources, &root)?;
    // A dependency's `MAIN` runs only when that Limb itself is run, and
    // is left out when another Limb uses it (owner, 2026-09-29: a).
    if as_dep.is_some() {
        file.items.retain(|i| !matches!(i.kind, crate::ast::ItemKind::Main(..)));
    }
    items.collect_file(&file, m, as_dep.is_none())?;
    load_dir(sources, items, dir, m, true)?;
    state.stack.push((key.clone(), shown));
    let mut deps = std::collections::HashMap::new();
    for d in &manifest.deps {
        let at = dir.join(&d.path);
        let r = load_limb(sources, items, &at, Some(&d.name), state)?;
        deps.insert(d.name.clone(), r);
    }
    state.stack.pop();
    items.limb_deps.insert(m, deps);
    state.done.insert(key, m);
    Ok(m)
}

/// What a `CCA.cplr` says (owner, 2026-09-29): one plain `const` per
/// setting, in CyborgPL, its shapes from the built-in `cca` Limb.
struct Manifest {
    name: Option<(String, crate::source::Span)>,
    deps: Vec<Dep>,
}

struct Dep {
    name: String,
    path: String,
}

const SETTINGS: [&str; 4] = ["name", "version", "dependencies", "targets"];

fn read_manifest(sources: &mut SourceMap, dir: &Path) -> Result<Manifest> {
    use crate::ast::{Arg as AArg, ExprKind, ItemKind};
    let mut out = Manifest { name: None, deps: Vec::new() };
    let at = dir.join("CCA.cplr");
    if !at.is_file() {
        return Ok(out);
    }
    let file = parse_file(sources, &at)?;
    let mut seen: Vec<String> = Vec::new();
    let text = |e: &crate::ast::Expr, what: &str| -> Result<String> {
        match &e.kind {
            ExprKind::Str(s) => Ok(s.clone()),
            _ => err(Diagnostic::error(format!("{} is text: `\"…\"`", what)).at(e.span)),
        }
    };
    for item in &file.items {
        let ItemKind::Const(c) = &item.kind else {
            return err(Diagnostic::error("`CCA.cplr` holds settings, one plain `const` each: `const 'name' := \"areas\";`").at(item.span));
        };
        if item.vis != Vis::Private {
            return err(Diagnostic::error(format!("`pub` means nothing in CCA.cplr: its settings are read by CCA, never reached from code — write `const '{}' := …;`", c.name.name)).at(item.span));
        }
        let setting = c.name.name.as_str();
        // One target became a list (owner, 2026-09-29).
        if setting == "target" {
            return err(Diagnostic::error("the setting is 'targets', a list: `const 'targets' := [cca::Target[name = \"aarch64-macos\"],];`").at(c.name.span));
        }
        if !SETTINGS.contains(&setting) {
            return err(Diagnostic::error(format!("CCA.cplr has no setting '{}': the settings are {}", setting, SETTINGS.iter().map(|s| format!("'{}'", s)).collect::<Vec<_>>().join(", "))).at(c.name.span));
        }
        if seen.iter().any(|s| s == setting) {
            return err(Diagnostic::error(format!("'{}' is set twice in CCA.cplr: one `const` per setting", setting)).at(c.name.span));
        }
        seen.push(setting.to_string());
        match setting {
            "name" => out.name = Some((text(&c.value, "'name'")?, c.value.span)),
            "version" => {
                text(&c.value, "'version'")?;
            }
            // The targets a Limb is built for (owner, 2026-09-29): each a
            // `cca::Target` with a `name` and, for one this machine cannot
            // run, a `runner`. CCA reads them and still builds for this
            // machine: which one is built when none is named is open.
            "targets" => {
                let ExprKind::List(entries) = &c.value.kind else {
                    return err(Diagnostic::error("'targets' is a list of `cca::Target[…]`: `[cca::Target[name = \"aarch64-macos\"],]`").at(c.value.span));
                };
                let mut names: Vec<String> = Vec::new();
                for e in entries {
                    let ExprKind::Call { path, args, .. } = &e.kind else {
                        return err(Diagnostic::error("each target is a `cca::Target[name = …]`, with a `runner = …` when this machine cannot run it").at(e.span));
                    };
                    if path.limb.as_ref().map(|l| l.name.as_str()) != Some("cca") || !path.segs.iter().map(|s| s.name.as_str()).eq(["Target"]) {
                        return err(Diagnostic::error(format!("each target is a `cca::Target[…]`, and this is `{}`", path.spelled())).at(e.span));
                    }
                    let mut name = None;
                    for a in args {
                        let AArg::Named(field, v) = a else {
                            return err(Diagnostic::error("a `cca::Target` names its fields: `name = …`, and `runner = …` when it needs one").at(a.span()));
                        };
                        match field.name.as_str() {
                            "name" => name = Some((text(v, "a target's `name`")?, field.span)),
                            "runner" => {
                                text(v, "a target's `runner`")?;
                            }
                            other => return err(Diagnostic::error(format!("a `cca::Target` has no field `{}`: it is a `name`, and a `runner` when this machine cannot run it", other)).at(field.span)),
                        }
                    }
                    let Some((n, span)) = name else {
                        return err(Diagnostic::error("a `cca::Target` needs its `name`: `cca::Target[name = \"aarch64-linux\"]`").at(e.span));
                    };
                    if names.contains(&n) {
                        return err(Diagnostic::error(format!("the target `{}` is listed twice in 'targets'", n)).at(span));
                    }
                    names.push(n);
                }
            }
            _ => {
                let ExprKind::List(entries) = &c.value.kind else {
                    return err(Diagnostic::error("'dependencies' is a list of `cca::Dep[…]`: `[cca::Dep[name = \"areas\", path = \"../areas\"],]`").at(c.value.span));
                };
                for e in entries {
                    let ExprKind::Call { path, args, .. } = &e.kind else {
                        return err(Diagnostic::error("each dependency is a `cca::Dep[name = …, path = …]`").at(e.span));
                    };
                    if path.limb.as_ref().map(|l| l.name.as_str()) != Some("cca") || !path.segs.iter().map(|s| s.name.as_str()).eq(["Dep"]) {
                        return err(Diagnostic::error(format!("each dependency is a `cca::Dep[…]`, and this is `{}`", path.spelled())).at(e.span));
                    }
                    let (mut name, mut dpath, mut version) = (None, None, None);
                    for a in args {
                        let AArg::Named(field, v) = a else {
                            return err(Diagnostic::error("a `cca::Dep` names its fields: `name = …`, and a `path = …` or a `version = …`").at(a.span()));
                        };
                        let slot = match field.name.as_str() {
                            "name" => &mut name,
                            "path" => &mut dpath,
                            "version" => &mut version,
                            other => return err(Diagnostic::error(format!("a `cca::Dep` has no field `{}`: it is a `name`, and a `path` or a `version`", other)).at(field.span)),
                        };
                        *slot = Some((text(v, &format!("`{}`", field.name))?, field.span));
                    }
                    let Some((dname, dspan)) = name else {
                        return err(Diagnostic::error("a `cca::Dep` needs its `name`: the name its code reaches it by, `name::…`").at(e.span));
                    };
                    match (dpath, version) {
                        (Some(_), Some((_, vspan))) => {
                            return err(Diagnostic::error(format!("`{}` says where it comes from once: a `path` (a folder on this machine) or a `version` (the registry), not both", dname)).at(vspan));
                        }
                        (None, Some((_, vspan))) => {
                            return err(Diagnostic::error(format!("not yet: `{}` asks the registry for a `version`, and the registry does not exist yet — give its folder with `path = \"…\"`", dname)).at(vspan));
                        }
                        (None, None) => {
                            return err(Diagnostic::error(format!("`{}` needs to say where it comes from: a `path = \"…\"` (a folder on this machine) or a `version` (the registry)", dname)).at(e.span));
                        }
                        (Some((p, _)), None) => {
                            if out.deps.iter().any(|d| d.name == dname) {
                                return err(Diagnostic::error(format!("`{}` is listed twice in 'dependencies'", dname)).at(dspan));
                            }
                            out.deps.push(Dep { name: dname, path: p });
                        }
                    }
                }
            }
        }
    }
    Ok(out)
}

fn new_module(items: &mut Items, name: &str, parent: Option<ModuleId>, vis: Vis) -> ModuleId {
    let path = crate::resolve::module_path(items, name, parent);
    let id = ModuleId(items.modules.len() as u32);
    items.modules.push(crate::resolve::Module { name: name.to_string(), path, parent, vis, items: Default::default(), consts: Default::default() });
    id
}

fn parse_file(sources: &mut SourceMap, path: &Path) -> Result<crate::ast::File> {
    let text = std::fs::read_to_string(path).map_err(|e| Box::new(Diagnostic::error(format!("cannot read `{}`: {}", path.display(), e))))?;
    let id: FileId = sources.add(path, text);
    let toks = crate::lexer::lex(id, &sources.get(id).text)?;
    crate::parser::parse_file(&toks)
}

/// The entries of `dir` as modules of `parent`.
fn load_dir(sources: &mut SourceMap, items: &mut Items, dir: &Path, parent: ModuleId, is_root_dir: bool) -> Result<()> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir).map_err(|e| Box::new(Diagnostic::error(format!("cannot read `{}`: {}", dir.display(), e))))?.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    // Files first, so a directory `x/` can nest under a file module `x.cpls`.
    let mut file_modules: Vec<(String, ModuleId)> = Vec::new();
    for e in entries.iter().filter(|p| p.is_file()) {
        let Some(ext) = e.extension() else { continue };
        if ext != "cpls" {
            continue;
        }
        let stem = e.file_stem().unwrap().to_string_lossy().to_string();
        if is_root_dir && stem == "limb" {
            continue;
        }
        check_name(&stem, e)?;
        if let Some((existing, _, _)) = items.modules[parent.0 as usize].items.get(&stem) {
            if let crate::resolve::ItemRef::Module(_) = existing {
                return err(Diagnostic::error(format!("`{}` is both a file module (`{}`) and a `module {} {{ … }}` block: a module cannot be both — keep one", stem, e.display(), stem)));
            }
            return err(Diagnostic::error(format!("`{}` names both the file module `{}` and a {} declared in its parent: a module and a type cannot share a name", stem, e.display(), existing.what())));
        }
        let m = new_module(items, &stem, Some(parent), Vis::Pub);
        items.modules[parent.0 as usize].items.insert(stem.clone(), (crate::resolve::ItemRef::Module(m), Vis::Pub, crate::source::Span::new(FileId(0), 0, 0)));
        let file = parse_file(sources, e)?;
        items.collect_file(&file, m, false)?;
        file_modules.push((stem, m));
    }
    for e in entries.iter().filter(|p| p.is_dir()) {
        let name = e.file_name().unwrap().to_string_lossy().to_string();
        if name.starts_with('.') || name == "target" {
            continue;
        }
        // A directory with no `.cpls` inside (recursively) is not a module.
        if !has_cpls(e) {
            continue;
        }
        check_name(&name, e)?;
        let m = match file_modules.iter().find(|(n, _)| *n == name) {
            Some((_, m)) => *m,
            None => {
                let m = new_module(items, &name, Some(parent), Vis::Pub);
                items.modules[parent.0 as usize].items.insert(name.clone(), (crate::resolve::ItemRef::Module(m), Vis::Pub, crate::source::Span::new(FileId(0), 0, 0)));
                m
            }
        };
        load_dir(sources, items, e, m, false)?;
    }
    Ok(())
}

fn has_cpls(dir: &Path) -> bool {
    std::fs::read_dir(dir).map(|rd| rd.filter_map(|e| e.ok()).any(|e| {
        let p = e.path();
        (p.is_file() && p.extension().map(|x| x == "cpls").unwrap_or(false)) || (p.is_dir() && has_cpls(&p))
    })).unwrap_or(false)
}

fn check_name(name: &str, at: &Path) -> Result<()> {
    let ok = name.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false) && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if !ok {
        return err(Diagnostic::error(format!("`{}` cannot name a module: module names are lowercase ASCII letters, digits and `_`, starting with a letter — rename `{}`", name, short(at))));
    }
    if crate::parser::is_device_name(name) {
        return err(Diagnostic::error(format!("`{}` is a reserved device name on Windows and cannot name a module: rename `{}`", name, short(at))));
    }
    if crate::parser::is_keyword(name) {
        return err(Diagnostic::error(format!("`{}` is a keyword and cannot name a module: rename `{}`", name, short(at))));
    }
    Ok(())
}

/// The file's own name, for a message: a full path is noise and differs per machine.
fn short(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| p.display().to_string())
}
