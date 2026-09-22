//! Loading a Limb: one `.cpls` file, or a directory whose `limb.cpls` is
//! the root and whose other files and directories are its modules
//! (`limbs.md`).

use std::path::{Path, PathBuf};

use crate::ast::Vis;
use crate::diag::{err, Diagnostic, Result};
use crate::resolve::{Items, ModuleId, ROOT};
use crate::source::{FileId, SourceMap};

pub fn load(path: &Path, sources: &mut SourceMap) -> Result<Items> {
    let mut items = Items::default();
    if path.is_dir() {
        let root = path.join("limb.cpls");
        if !root.is_file() {
            return err(Diagnostic::error(format!("`{}` is a Limb directory, and a Limb needs its root file `limb.cpls`: none is there", short(path))));
        }
        let m = new_module(&mut items, "limb", None, Vis::Pub);
        let file = parse_file(sources, &root)?;
        items.collect_file(&file, m, true)?;
        load_dir(sources, &mut items, path, m, true)?;
    } else {
        let m = new_module(&mut items, "limb", None, Vis::Pub);
        let file = parse_file(sources, path)?;
        items.collect_file(&file, m, true)?;
    }
    items.resolve_all()?;
    Ok(items)
}

fn new_module(items: &mut Items, name: &str, parent: Option<ModuleId>, vis: Vis) -> ModuleId {
    let path = match parent {
        None => "limb".to_string(),
        Some(p) if p == ROOT => name.to_string(),
        Some(p) => format!("{}.{}", items.modules[p.0 as usize].path, name),
    };
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
