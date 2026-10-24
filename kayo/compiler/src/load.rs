//! Loading a Kit: one `.cpls` file, or a directory whose `limb.cpls` is
//! the root and whose other files and directories are its modules
//! (`kits.md`).

use std::path::{Path, PathBuf};

use crate::ast::Vis;
use crate::diag::{err, Diagnostic, Result};
use crate::resolve::{Items, ModuleId};
use crate::source::{FileId, SourceMap};

/// What a Kit is loaded for: checking reads it whatever its targets;
/// compiling builds one target — the one named on the command line, or
/// the Kit's default.
pub enum Purpose<'a> {
    Check,
    Compile(Option<&'a str>),
}

/// The file names (owner, 2026-10-16): code ends `.cy`, a kit's root is
/// `kit.cy`, and its manifest, a file that is one value and never runs,
/// is `CyJet.toml` (owner, 2026-10-22; `CyJet.cyr` before).
pub const MANIFEST: &str = "CyJet.toml";
pub const ROOT: &str = "kit.cy";

/// A file or folder still named as it was before 2026-10-16, refused
/// naming its new name.
fn old_names(path: &Path) -> Result<()> {
    let name = |p: &Path| p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    // CyborgPL's file names (owner, 2026-10-16): each Zag and Nerve name
    // refused, naming the new one.
    let renamed = |n: &str| -> Option<String> {
        match n {
            "strike.zg" | "neuron.nrv" => Some("kit.cy".into()),
            "Ricochet.zv" | "Kinesin.nrp" | "CyJet.cyr" => Some(MANIFEST.into()),
            "build.zg" | "build.nrv" => Some("build.cy".into()),
            "zag_version.zv" | "nerve_version.nrp" => Some("cy_ver.cyr".into()),
            _ => [(".zg", ".cy"), (".nrv", ".cy"), (".zv", ".cyr"), (".nrp", ".cyr")].iter().find_map(|(o, n2)| n.strip_suffix(o).map(|s| format!("{}{}", s, n2))),
        }
    };
    if path.is_file() {
        if path.extension().is_some_and(|x| x == "cpls") {
            return err(Diagnostic::error("`.cpls` is the old ending: rename the file to `.cy`").named("old_file_name"));
        }
        if let Some(new) = renamed(&name(path)) {
            return err(Diagnostic::error(format!("`{}` has the old name: rename it to `{}`", name(path), new)).named("old_file_name"));
        }
        return Ok(());
    }
    for old in ["strike.zg", "Ricochet.zv", "build.zg", "zag_version.zv", "neuron.nrv", "Kinesin.nrp", "build.nrv", "nerve_version.nrp"] {
        if path.join(old).is_file() {
            return err(Diagnostic::error(format!("`{}` has the old name: rename it to `{}`", old, renamed(old).unwrap())).named("old_file_name"));
        }
    }
    // The manifest is `CyJet.toml` (owner, 2026-10-22).
    if path.join("CyJet.cyr").is_file() && !path.join(MANIFEST).is_file() {
        return err(Diagnostic::error(format!("`CyJet.cyr` has the old name: rename it to `{}`, written as Tankun's Obvious Minimal Language: {}", MANIFEST, MANIFEST_SHAPE)).named("old_file_name"));
    }
    if path.join("build.cplr").is_file() {
        return err(Diagnostic::error("`build.cplr` has the old name: rename it to `build.cy`").named("old_file_name"));
    }
    if path.join("CCA.cplr").is_file() {
        return err(Diagnostic::error(format!("`CCA.cplr` has the old name: rename it to `{}`", MANIFEST)).named("old_file_name"));
    }
    if !path.join(ROOT).is_file() && path.join("limb.cpls").is_file() {
        return err(Diagnostic::error(format!("`limb.cpls` is the old root file: rename it to `{}`", ROOT)).named("old_file_name"));
    }
    // A leftover old file anywhere in the kit's folders.
    fn under(d: &Path, found: &mut Option<std::path::PathBuf>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        let mut es: Vec<_> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        es.sort();
        for e in es {
            if found.is_some() {
                return;
            }
            let n = e.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            if e.is_dir() {
                if !n.starts_with('.') && n != "target" && !e.join(MANIFEST).is_file() {
                    under(&e, found);
                }
            } else if e.extension().is_some_and(|x| x == "cpls" || x == "nrv" || x == "nrp" || x == "zg" || x == "zv") {
                *found = Some(e);
            }
        }
    }
    let mut found = None;
    under(path, &mut found);
    if let Some(f) = found {
        let stem = f.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let new = renamed(&name(&f)).unwrap_or_else(|| format!("{}.cy", stem));
        return err(Diagnostic::error(format!("`{}` has the old name: rename it to `{}`", name(&f), new)).named("old_file_name"));
    }
    Ok(())
}

/// The std a file with no CyJet.toml gets: the one the machine's package
/// installed, which for now is the repository's `std/` beside kayo's
/// source (`KAYO_STD` names another); none if it is not there, and then
/// the built-in print and exit are all there is.
fn installed_std() -> Option<PathBuf> {
    let dir = match std::env::var_os("KAYO_STD") {
        Some(d) => PathBuf::from(d),
        None => Path::new(env!("CARGO_MANIFEST_DIR")).join("../../std"),
    };
    dir.join(MANIFEST).is_file().then_some(dir)
}

pub fn load(path: &Path, sources: &mut SourceMap) -> Result<Items> {
    load_for(path, sources, Purpose::Check)
}

/// Every `precedence` block of the files this load reaches — the Kit's
/// and its dependencies', all the way down — into one table, before any
/// file is parsed (owner, 2026-10-03, 9cc5e25). One operator given two
/// levels is refused.
fn gather_precedence(sources: &mut SourceMap, path: &Path) -> Result<()> {
    let mut table: std::collections::HashMap<String, (u16, crate::parser::Chain, std::path::PathBuf)> = std::collections::HashMap::new();
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    if path.is_dir() {
        let mut dirs = vec![path.to_path_buf()];
        let mut seen = std::collections::HashSet::new();
        while let Some(d) = dirs.pop() {
            let Ok(canon) = d.canonicalize() else { continue };
            if !seen.insert(canon) {
                continue;
            }
            fn cpls_under(d: &Path, out: &mut Vec<std::path::PathBuf>) {
                let Ok(rd) = std::fs::read_dir(d) else { return };
                let mut es: Vec<_> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
                es.sort();
                for e in es {
                    let name = e.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                    if e.is_dir() {
                        if !name.starts_with('.') && name != "target" && !e.join(MANIFEST).is_file() {
                            cpls_under(&e, out);
                        }
                    } else if e.extension().is_some_and(|x| x == "cy") {
                        out.push(e);
                    }
                }
            }
            cpls_under(&d, &mut files);
            files.retain(|f| *f != d.join("build.cy"));
            if d.join(MANIFEST).is_file() {
                if let Ok(m) = read_manifest(sources, &d) {
                    for dep in &m.deps {
                        dirs.push(d.join(&dep.path));
                    }
                }
            }
        }
    } else {
        files.push(path.to_path_buf());
    }
    for f in files {
        let Ok(text) = std::fs::read_to_string(&f) else { continue };
        if !text.contains("precedence") {
            continue;
        }
        let id = sources.add(&f, text);
        let toks = sources.lex_file(id)?;
        for (name, (level, chain)) in crate::parser::precedence_blocks(&toks)? {
            match table.get(&name) {
                Some((l, c, other)) if (*l, *c) != (level, chain) => {
                    return err(Diagnostic::error(format!("`{}` has two levels: {} and {} each give it one, and an operator binds one way wherever it is used — one block says it", name, other.display(), f.display())).named("operator_level_given_twice"));
                }
                _ => {
                    table.insert(name, (level, chain, f.clone()));
                }
            }
        }
    }
    crate::parser::set_shared_precedence(table.into_iter().map(|(k, (l, c, _))| (k, (l, c))).collect());
    Ok(())
}

pub fn load_for(path: &Path, sources: &mut SourceMap, purpose: Purpose) -> Result<Items> {
    crate::parser::forget_wrong_brackets();
    old_names(path)?;
    // The target, known before any file is read (`on.target`): the one
    // named, or else the root Neuron's default, set where its manifest is
    // read; this machine for a lone file.
    crate::target::set(match &purpose {
        Purpose::Compile(Some(t)) => Some(t),
        _ => None,
    });
    crate::target::set_layout(None);
    gather_precedence(sources, path)?;
    let mut items = Items::default();
    // The root Kit's targets and default, when it has a `CCA.cplr`.
    let mut root: Option<(Vec<String>, Option<(String, crate::source::Span)>)> = None;
    let mut layouts: Vec<(String, String, crate::source::Span)> = Vec::new();
    let mut root_dir: Option<PathBuf> = None;
    let mut outputs: Vec<(String, String, crate::source::Span)> = Vec::new();
    let mut exits: Vec<(String, String, crate::source::Span)> = Vec::new();
    if path.is_dir() {
        let mut state = Loading { stack: Vec::new(), done: std::collections::HashMap::new(), prefixes: std::collections::HashMap::new(), root: None, root_layouts: Vec::new(), root_outputs: Vec::new(), root_exits: Vec::new(), root_dir: None, cord: None };
        load_kit(sources, &mut items, path, None, false, &mut state)?;
        root = state.root;
        layouts = state.root_layouts;
        outputs = state.root_outputs;
        exits = state.root_exits;
        root_dir = state.root_dir;
    }
    // What is built is a target from the manifest, never the host
    // (`reproducible-builds.md`): the one named, or the default. A lone
    // target saying `default = false` leaves none, and a compile must then
    // name it (owner, 2026-09-30). Every target is in `cca/targets.txt`
    // (`read_manifest`), and CCA writes code on its own machine only so far.
    let chosen: Option<(String, Option<crate::source::Span>)> = match (&purpose, &root) {
        (Purpose::Check, Some((_, d))) => d.as_ref().map(|(t, s)| (t.clone(), Some(*s))),
        (Purpose::Check, None) => None,
        (Purpose::Compile(Some(t)), Some((listed, _))) => {
            if !listed.iter().any(|l| l == t) {
                return err(Diagnostic::error(format!("`{}` is not one of this kit's targets: its `CyJet.toml` lists {}", t, listed.iter().map(|l| format!("`{}`", l)).collect::<Vec<_>>().join(", "))).named("target_not_listed"));
            }
            Some((t.to_string(), None))
        }
        (Purpose::Compile(Some(t)), None) => {
            let known = targets();
            if !known.contains(t) {
                return err(Diagnostic::error(format!("`{}` is not a target kayo can build; it builds for {}", t, known.iter().map(|k| format!("`{}`", k)).collect::<Vec<_>>().join(", "))).named("target_not_buildable"));
            }
            Some((t.to_string(), None))
        }
        (Purpose::Compile(None), Some((listed, None))) => {
            return err(Diagnostic::error(format!("this kit has no default target — its one target says `default = false` — so a build names the target: `--target {}`", listed.first().map(String::as_str).unwrap_or("aarch64-macos"))).named("manifest_default_unclear"));
        }
        (Purpose::Compile(None), Some((_, Some((t, s))))) => Some((t.clone(), Some(*s))),
        (Purpose::Compile(None), None) => None,
    };
    let mut board: (Option<(String, crate::source::Span)>, Option<(String, crate::source::Span)>) = (None, None);
    // A no-OS target is linked against the Neuron's own script, named by
    // its `layout` (owner, 2026-10-09: "`c` now", then "`b`").
    if let Some((t, span)) = chosen.as_ref().filter(|(t, _)| t.ends_with("-none")) {
        let Some(dir) = &root_dir else {
            let d = Diagnostic::error(format!("a lone file is not built for `{}`: a program with no OS brings a linker script saying where it sits in the board's memory, named in a kit's CyJet.toml — `{} {{` / `layout = boards/virt.ld` / `}}` under `my targets:`", t, t)).named("target_layout_missing");
            return err(match span {
                Some(s) => d.at(*s),
                None => d,
            });
        };
        // Refused when built for it, not when listed: a kit may list a
        // board beside the machine it is built on (`examples/table_limits`).
        let Some((_, file, at)) = layouts.iter().find(|(n, _, _)| n == t) else {
            let d = Diagnostic::error(format!("the target `{t}` has no `layout`: a program with no OS brings its own linker script, saying where it sits in the board's memory — `{t} {{` / `layout = boards/virt.ld` / `}}` under `my targets:`, the path from the kit's folder")).named("target_layout_missing");
            return err(match span {
                Some(s) => d.at(*s),
                None => d,
            });
        };
        let script = dir.join(file);
        if !script.is_file() {
            return err(Diagnostic::error(format!("the linker script `{}`, the `layout` of `{}`, is not there: the path is from the kit's folder", file, t)).named("target_layout_missing").at(*at));
        }
        crate::target::set_layout(Some((script, file.clone())));
        board = (outputs.into_iter().find(|(n, _, _)| n == t).map(|(_, f, s)| (f, s)), exits.into_iter().find(|(n, _, _)| n == t).map(|(_, f, s)| (f, s)));
    }
    if let Some((t, span)) = chosen {
        let here = crate::target::host();
        // A no-OS target is built on another machine (owner, 2026-10-09:
        // `aarch64-none`), from any.
        if t != here && !t.ends_with("-none") {
            let d = Diagnostic::error(format!("not yet: the target is `{}`, and kayo builds it only on a `{}` machine so far — this one is `{}`", t, t, here)).named("not_built_yet");
            return err(match span {
                Some(s) => d.at(s),
                None => d,
            });
        }
    }
    if !path.is_dir() {
        let m = new_module(&mut items, "kit", None, Vis::Pub);
        let dir = path.parent().map(|p| if p.as_os_str().is_empty() { Path::new(".") } else { p }).unwrap_or(Path::new(".")).to_path_buf();
        crate::parser::set_neuron_root(&dir);
        items.neuron_dirs.insert(m, dir.clone());
        let file = parse_file(sources, path)?;
        check_bare(sources, &mut items, &file, "", true)?;
        check_case(sources, &mut items, &file, true)?;
        items.collect_file(&file, m, true)?;
        // A file with no CyJet.toml gets what the machine's package installed
        // (owner, 2026-10-06: "yea"): with the std package, std. Until an
        // installer writes where it is, the repository's `std/` stands in
        // for it, as the oracle stands in for the full package.
        if let Some(std_dir) = installed_std() {
            let mut state = Loading { stack: Vec::new(), done: std::collections::HashMap::new(), prefixes: std::collections::HashMap::new(), root: None, root_layouts: Vec::new(), root_outputs: Vec::new(), root_exits: Vec::new(), root_dir: None, cord: None };
            let from = std_dir.join(MANIFEST);
            let (r, _) = load_kit(sources, &mut items, &std_dir, Some((&from, "std")), true, &mut state)?;
            items.nerve_cord = Some(r);
            items.kit_deps.insert(m, std::collections::HashMap::new());
            crate::parser::set_neuron_root(&dir);
        }
    }
    // The Brainstem's `optional` (owner, 2026-10-08), every program's.
    let base_id = sources.add(Path::new("the base"), BASE_SOURCE.to_string());
    let base = crate::parser::parse_file(&crate::lexer::lex(base_id, BASE_SOURCE)?)?;
    items.add_base(&base)?;
    items.resolve_all()?;
    // Every kit's C libraries, once each, in the order they were read.
    let mut libs: Vec<String> = Vec::new();
    for k in &items.report_kits {
        for l in &k.c_libs {
            if !libs.contains(l) {
                libs.push(l.clone());
            }
        }
    }
    crate::target::set_c_libs(libs);
    // The board's output and exit, now that its functions are known.
    if let Some((f, at)) = board.0 {
        items.board_output = Some(board_fn(&items, &f, at, "output")?);
    }
    if let Some((f, at)) = board.1 {
        items.board_exit = Some(board_fn(&items, &f, at, "exit")?.0);
    }
    Ok(items)
}

/// A function the target names, `my.serial.write`, of the Neuron being
/// built, of the shape its field asks: an `output` takes one `u8` or a
/// `lent` `array[u8]` (owner, 2026-10-09: "Yeah", both); an `exit` takes
/// the code, a `u8` as `cord::exit`'s is. Neither returns a value or fails.
fn board_fn(items: &Items, written: &str, at: crate::source::Span, field: &str) -> Result<(crate::resolve::FnId, crate::resolve::OutputForm)> {
    use crate::resolve::{ItemRef, OutputForm};
    use crate::types::{ParamKind, SeqKind, Ty};
    // This Neuron's (`my.serial.write`) or a board Neuron's it depends on
    // (`virt::serial.write`), written as code writes it.
    let (kit, rest) = match written.split_once("::") {
        Some((l, r)) => (Some(crate::ast::Ident { name: l.to_string(), span: at }), r),
        None => (None, written),
    };
    let segs: Vec<crate::ast::Ident> = rest.split('.').map(|s| crate::ast::Ident { name: s.to_string(), span: at }).collect();
    if segs.iter().any(|s| s.name.is_empty()) || kit.as_ref().is_some_and(|l| l.name.is_empty()) {
        return err(Diagnostic::error(format!("a target's `{}` names a function as code does, `my.serial.write` or `virt::serial.write`; `{}` is not one", field, written)).named("target_function_wrong").at(at));
    }
    let path = crate::ast::Path { kit, segs, span: at };
    let id = match items.lookup_path(&path, crate::resolve::ROOT) {
        Ok(ItemRef::Fn(id)) => id,
        Ok(other) => return err(Diagnostic::error(format!("the target's `{}`, `{}`, is a {}: it names a function", field, written, other.what())).named("target_function_wrong").at(at)),
        Err(e) => return err(Diagnostic::error(format!("the target's `{}`, `{}`, names no function: {}", field, written, e.sentence)).named("target_function_wrong").at(at)),
    };
    let f = &items.fns[id.0 as usize];
    let one = |kind: ParamKind, ty: &Ty| f.params.len() == 1 && f.params[0].kind == kind && f.params[0].ty.as_ref() == Some(ty);
    let plain = f.ret == Ty::Nothing && f.fails.is_empty() && f.blanks.is_empty();
    let byte = Ty::int(false, 8);
    let form = if plain && one(ParamKind::Owned, &byte) {
        OutputForm::Byte
    } else if field == "output" && plain && one(ParamKind::Lent, &Ty::Seq { elem: Box::new(byte.clone()), kind: SeqKind::Array }) {
        OutputForm::Chunk
    } else if field == "output" {
        return err(Diagnostic::error(format!("the target's `output`, `fn {}`, takes the bytes `std::print` writes in one of two forms: `[$b: u8]`, a byte a call, or `[lent $bytes: array[u8]]`, a chunk a call; returning nothing and never failing", written)).named("target_function_wrong").at(at).why_at(f.name_span, "declared here"));
    } else {
        return err(Diagnostic::error(format!("the target's `exit`, `fn {}`, takes the code `std::exit` is given, `[$code: u8]`, returning nothing and never failing", written)).named("target_function_wrong").at(at).why_at(f.name_span, "declared here"));
    };
    Ok((id, form))
}

/// What every program has without writing it (owner, 2026-10-08: "yea").
const BASE_SOURCE: &str = "enum optional <T: any> { value: <T>, none }\n";

/// The targets CCA writes code for: `cca/targets.txt`, the one list,
/// which Synthia copies.
pub fn targets() -> Vec<&'static str> {
    include_str!("../../targets.txt").lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).collect()
}

/// Which Kits `cca prepare` runs (owner, 2026-09-29 and 2026-09-30).
pub enum Pick {
    /// `--all`: every Kit reached, dependencies first.
    All,
    /// Kits named, in the order given.
    Listed(Vec<Wanted>),
}

/// One Kit asked for by `cca prepare`.
pub enum Wanted {
    /// By its `name`: this Kit's own or a dependency's.
    Name(String),
    /// By its prefix (`--prefix geo2`), for a name two Kits share.
    Prefix(String),
}

/// What `cca prepare` runs, in order: each Kit's `build.nrv` with the
/// name it goes by and the folder it runs in. Names run in the order given,
/// each once; `--all` runs dependencies before the Kits that use them,
/// each once. Nothing is loaded but manifests.
pub fn prepare_plan(sources: &mut SourceMap, dir: &Path, pick: &Pick) -> Result<Vec<(String, PathBuf)>> {
    if !dir.join(ROOT).is_file() {
        return err(Diagnostic::error(format!("`kayo prepare` acts on a kit, and `{}` has no `kit.cy`", dir.display())).named("root_file_missing"));
    }
    // A Kit's name and prefix, from its own `CCA.cplr`; a dependency
    // without one has neither.
    fn words_of(sources: &mut SourceMap, dir: &Path, from: Option<(&Path, &str)>) -> Result<(String, String)> {
        if let Some((manifest_at, written)) = from {
            if !dir.join(MANIFEST).is_file() {
                return err(Diagnostic::error(format!("the kit at `{}` has no `CyJet.toml`, so no `prefix` for code to reach it by", written)).named("manifest_missing").outside_program().place(manifest_at.display().to_string()));
            }
        }
        let m = read_manifest(sources, dir)?;
        let folder = dir.canonicalize().ok().and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string())).unwrap_or_else(|| "this kit".into());
        Ok((m.name.map(|(n, _)| n).unwrap_or_else(|| folder.clone()), m.prefix.map(|(p, _)| p).unwrap_or(folder)))
    }
    // How a Kit is shown: its name, and its prefix too where that differs,
    // since two Kits may share a name.
    fn label(name: &str, prefix: &str) -> String {
        if name == prefix { name.to_string() } else { format!("{} (prefix `{}`)", name, prefix) }
    }
    let manifest = read_manifest(sources, dir)?;
    let manifest_at = dir.join(MANIFEST);
    let (own_name, own_prefix) = words_of(sources, dir, None)?;
    let names = match pick {
        Pick::All => {
            // Dependencies first: a Kit is listed after everything under it.
            fn walk(sources: &mut SourceMap, dir: &Path, shown: String, stack: &mut Vec<(PathBuf, String)>, done: &mut Vec<PathBuf>, out: &mut Vec<(String, PathBuf)>) -> Result<()> {
                let key = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
                if done.contains(&key) {
                    return Ok(());
                }
                if let Some(at) = stack.iter().position(|(p, _)| *p == key) {
                    let mut names: Vec<String> = stack[at..].iter().map(|(_, n)| n.clone()).collect();
                    names.push(shown);
                    return err(Diagnostic::error(format!("the dependencies go round in a circle: {} — a kit cannot depend on itself, however far round", names.join(" → "))).named("kits_depend_in_circle").outside_program());
                }
                let manifest = read_manifest(sources, dir)?;
                let manifest_at = dir.join(MANIFEST);
                stack.push((key.clone(), shown.clone()));
                for d in &manifest.deps {
                    let at = dir.join(&d.path);
                    let (n, p) = words_of(sources, &at, Some((&manifest_at, &d.path)))?;
                    walk(sources, &at, label(&n, &p), stack, done, out)?;
                }
                stack.pop();
                done.push(key);
                if dir.join("build.cy").is_file() {
                    out.push((shown, dir.to_path_buf()));
                }
                Ok(())
            }
            let mut out = Vec::new();
            walk(sources, dir, label(&own_name, &own_prefix), &mut Vec::new(), &mut Vec::new(), &mut out)?;
            return Ok(out);
        }
        Pick::Listed(w) => w,
    };
    // This Kit and its dependencies: name, prefix, folder.
    let mut known: Vec<(String, String, PathBuf)> = vec![(own_name, own_prefix, dir.to_path_buf())];
    for d in &manifest.deps {
        let at = dir.join(&d.path);
        let (n, p) = words_of(sources, &at, Some((&manifest_at, &d.path)))?;
        known.push((n, p, at));
    }
    let mut out: Vec<(String, PathBuf)> = Vec::new();
    let mut ran: Vec<PathBuf> = Vec::new();
    for w in names {
        let (n, by_prefix) = match w {
            Wanted::Name(n) => (n, false),
            Wanted::Prefix(p) => (p, true),
        };
        let found: Vec<&(String, String, PathBuf)> = known.iter().filter(|(name, prefix, _)| if by_prefix { prefix == n } else { name == n }).collect();
        let at = match found.as_slice() {
            [] if by_prefix => return err(Diagnostic::error(format!("no kit here has the prefix `{}`: the prefixes here are {}", n, known.iter().map(|(_, p, _)| format!("`{}`", p)).collect::<Vec<_>>().join(", "))).named("kit_not_found")),
            [] => {
                let mut names: Vec<String> = Vec::new();
                for (k, _, _) in &known {
                    if !names.contains(k) {
                        names.push(k.clone());
                    }
                }
                return err(Diagnostic::error(format!("no kit here is named `{}`: the names here are {}", n, names.iter().map(|k| format!("`{}`", k)).collect::<Vec<_>>().join(", "))).named("kit_not_found"));
            }
            [one] => (label(&one.0, &one.1), one.2.clone()),
            // Two Kits may share a name, never a prefix: which one is
            // meant is said by prefix (owner, 2026-09-30: a, "flag").
            many => {
                let prefixes: Vec<String> = many.iter().map(|(_, p, _)| format!("`{}`", p)).collect();
                return err(Diagnostic::error(format!("{} kits here are named `{}`, with the prefixes {}: pick one with `kayo prepare --prefix {}`", many.len(), n, prefixes.join(", "), many[0].1)).named("name_ambiguous"));
            }
        };
        let (shown, at) = at;
        // Each Kit once, however it was asked for.
        let key = std::fs::canonicalize(&at).unwrap_or_else(|_| at.clone());
        if ran.contains(&key) {
            continue;
        }
        ran.push(key);
        if !at.join("build.cy").is_file() {
            return err(Diagnostic::error(format!("`{}` has no `build.cy` to run, at `{}`", n, at.display())).named("build_file_missing"));
        }
        out.push((shown, at));
    }
    Ok(out)
}

/// Where loading has got to: the Kits being loaded now (a dependency back
/// to one of them is a cycle), those loaded already (two Kits that share a
/// dependency share one copy of it), and each dependency's prefix with the
/// folder that has it.
struct Loading {
    stack: Vec<(PathBuf, String)>,
    done: std::collections::HashMap<PathBuf, (ModuleId, String)>,
    prefixes: std::collections::HashMap<String, PathBuf>,
    /// The Kit being built: its targets and default, from its `CCA.cplr`.
    root: Option<(Vec<String>, Option<(String, crate::source::Span)>)>,
    /// Its folder and its no-OS targets' linker scripts.
    root_layouts: Vec<(String, String, crate::source::Span)>,
    root_outputs: Vec<(String, String, crate::source::Span)>,
    root_exits: Vec<(String, String, crate::source::Span)>,
    root_dir: Option<PathBuf>,
    /// The Nerve cord's folder, once a Neuron lists it: one per program.
    cord: Option<PathBuf>,
}

/// One Kit directory and, through its `CCA.cplr`, its dependencies: each
/// a tree of its own, its root's `kit_deps` naming theirs by their own
/// 'prefix' (owner, 2026-09-29). `from` is, for a dependency, the depending
/// Kit's `CCA.cplr` and the path it wrote; the Kit being built has none,
/// and its root is `ROOT`, made first. Gives back the Kit's root module
/// and its prefix.
fn load_kit(sources: &mut SourceMap, items: &mut Items, dir: &Path, from: Option<(&Path, &str)>, as_cord: bool, state: &mut Loading) -> Result<(ModuleId, String)> {
    load_kit_inner(sources, items, dir, from, as_cord, state)
}

fn load_kit_inner(sources: &mut SourceMap, items: &mut Items, dir: &Path, from: Option<(&Path, &str)>, as_cord: bool, state: &mut Loading) -> Result<(ModuleId, String)> {
    old_names(dir)?;
    let key = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    if let Some(r) = state.done.get(&key) {
        return Ok(r.clone());
    }
    if let Some(at) = state.stack.iter().position(|(p, _)| *p == key) {
        let mut names: Vec<String> = state.stack[at..].iter().map(|(_, n)| n.clone()).collect();
        names.push(state.stack[at].1.clone());
        return err(Diagnostic::error(format!("the dependencies go round in a circle: {} — a kit cannot depend on itself, however far round", names.join(" → "))).named("kits_depend_in_circle").outside_program());
    }
    let root = dir.join(ROOT);
    if !root.is_file() {
        return err(Diagnostic::error(match from {
            None => format!("`{}` is a kit directory, and a kit needs its root file `kit.cy`: none is there", short(dir)),
            Some((_, written)) => format!("the kit at `{}` needs its root file `kit.cy`: none is there", written),
        }).named("root_file_missing"));
    }
    // A dependency is reached by the prefix its own `CCA.cplr` gives it,
    // and the depending Kit names no word (owner, 2026-09-29).
    if let Some((manifest_at, written)) = from {
        if !dir.join(MANIFEST).is_file() {
            return err(Diagnostic::error(format!("the kit at `{}` has no `CyJet.toml`, so no `prefix` for code to reach it by", written)).named("manifest_missing").outside_program().place(manifest_at.display().to_string()));
        }
    }
    let manifest = read_manifest(sources, dir)?;
    let prefix = match &manifest.prefix {
        Some((p, _)) => p.clone(),
        None => dir.canonicalize().ok().and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string())).unwrap_or_else(|| "this kit".into()),
    };
    if let Some((manifest_at, written)) = from {
        // The CyborgPL std is the kit listed with std's ID, and its prefix
        // is `std` (owner, 2026-10-14; `cord` until 2026-10-15).
        if as_cord && prefix != "std" {
            return err(Diagnostic::error(format!("the kit at `{}` is listed with std's ID, and its prefix is `{}`: the CyborgPL std's prefix is `std`", written, prefix)).named("kit_prefix_clash").outside_program().place(manifest_at.display().to_string()));
        }
        // `std` is reserved with its old names, and CyJet's prefixes,
        // but for the CyborgPL std.
        if !as_cord && matches!(prefix.as_str(), "std" | "stdkit" | "cord" | "cca" | "kin" | "ricochet" | "cyjet") {
            return err(Diagnostic::error(format!("the kit at `{}` has the prefix `{}`, which names the built-in `{}::`; edit its `prefix`", written, prefix, match prefix.as_str() { "cca" | "kin" | "ricochet" | "cyjet" => "cyjet", _ => "std" })).named("kit_prefix_clash").outside_program().place(manifest_at.display().to_string()));
        }
        // No override: two Kits with one prefix are fixed by editing one
        // (owner, 2026-09-29: "Just edit the source code").
        if let Some(other) = state.prefixes.get(&prefix) {
            if *other != key && as_cord {
                return err(Diagnostic::error(format!("two CyborgPL stds are listed, `{}` and `{}`: a program has one std (owner, 2026-10-14)", other.display(), key.display())).named("manifest_given_twice").place(manifest_at.display().to_string()));
            }
            if *other != key {
                return err(Diagnostic::error(format!("two kits have the prefix `{}` ({} and {}); edit one kit's `prefix`", prefix, other.display(), key.display())).named("kit_prefix_clash").outside_program().place(manifest_at.display().to_string()));
            }
        }
        state.prefixes.insert(prefix.clone(), key.clone());
    }
    // What the build report lists of it (owner, 2026-10-19).
    items.report_kits.push(crate::resolve::KitReport {
        dir: dir.to_path_buf(),
        name: manifest.name.as_ref().map(|(n, _)| n.clone()),
        prefix: prefix.clone(),
        version: manifest.version.clone(),
        program: from.is_none(),
        std: as_cord,
        limbs: manifest.cores.clone(),
        targets: manifest.targets.clone(),
        default_target: manifest.default_target.as_ref().map(|(t, _)| t.clone()),
        table_bytes: manifest.table_bytes.clone(),
        c_libs: manifest.c_libs.clone(),
    });
    if from.is_none() && dir.join(MANIFEST).is_file() {
        state.root = Some((manifest.targets.clone(), manifest.default_target.clone()));
        state.root_layouts = manifest.layouts.clone();
        state.root_outputs = manifest.outputs.clone();
        state.root_exits = manifest.exits.clone();
        state.root_dir = Some(dir.to_path_buf());
        if !crate::target::is_set() {
            crate::target::set(manifest.default_target.as_ref().map(|(t, _)| t.as_str()));
        }
    }
    let m = match from {
        None => new_module(items, "kit", None, Vis::Pub),
        Some(_) => {
            let id = ModuleId(items.modules.len() as u32);
            items.modules.push(crate::resolve::Module { name: "kit".to_string(), path: format!("{}::kit", prefix), parent: None, vis: Vis::Pub, items: Default::default(), consts: Default::default(), folded: Default::default(), folded_consts: Default::default() });
            id
        }
    };
    items.neuron_dirs.insert(m, dir.to_path_buf());
    // A Kit without std reaches none of it; its dependencies are judged
    // by their own `CCA.cplr` (owner, 2026-10-06).
    // The Nerve cord is cord: lists, maps and text are its own (owner,
    // 2026-10-14).
    let no_cord = dir.join(MANIFEST).is_file() && !manifest.has_std && !as_cord;
    crate::parser::set_no_std(no_cord);
    crate::parser::set_cores(dir.join(MANIFEST).is_file().then(|| manifest.cores.clone()));
    let outer_root = crate::parser::set_neuron_root(dir);
    // The kits it lists: their `pub` handlers, by their prefix, before its
    // own files are read (owner, 2026-10-23).
    let mut kit_handlers = std::collections::HashMap::new();
    for d in manifest.deps.iter().filter(|d| !d.cord) {
        let at = dir.join(&d.path);
        if let Some(prefix) = kit_prefix(sources, &at) {
            kit_handlers.insert(prefix, pub_handlers(sources, &at)?);
        }
    }
    let outer_handlers = crate::parser::set_kit_handlers(kit_handlers);
    let parsed = (|| {
        let mut file = parse_file(sources, &root)?;
        check_bare(sources, items, &file, "", true)?;
        check_case(sources, items, &file, true)?;
        // A dependency's `MAIN` runs only when that Kit itself is run, and
        // is left out when another Kit uses it (owner, 2026-09-29: a).
        if from.is_some() {
            file.items.retain(|i| !matches!(i.kind, crate::ast::ItemKind::Main(..)));
        }
        items.collect_file(&file, m, from.is_none())?;
        load_dir(sources, items, dir, m, true)
    })();
    crate::parser::set_no_std(false);
    crate::parser::set_cores(None);
    crate::parser::set_neuron_root(&outer_root);
    crate::parser::set_kit_handlers(outer_handlers);
    parsed?;
    if no_cord {
        items.no_std_roots.insert(m);
    }
    // The power ledger's view of this Kit (owner, 2026-10-07): its list,
    // its cores; and, for the program, its `deny`.
    if dir.join(MANIFEST).is_file() {
        items.kit_powers.insert(m, crate::resolve::KitPowers { prefix: prefix.clone(), is_program: from.is_none(), lists: manifest.powers.clone(), cores: manifest.cores.clone() });
        if from.is_none() {
            items.program_deny = manifest.deny.clone();
        }
    }
    state.stack.push((key.clone(), prefix.clone()));
    let mut deps = std::collections::HashMap::new();
    let mut in_order: Vec<(String, ModuleId)> = Vec::new();
    let manifest_at = dir.join(MANIFEST);
    for d in &manifest.deps {
        let at = dir.join(&d.path);
        let (r, p) = load_kit(sources, items, &at, Some((&manifest_at, &d.path)), d.cord, state)?;
        // One cord per program (owner, 2026-10-14: "Yes"): every `cord::`
        // in the build goes to the Nerve cord, whichever Neuron listed it.
        if d.cord {
            let found = std::fs::canonicalize(&at).unwrap_or(at.clone());
            if let Some(other) = state.cord.as_ref().filter(|o| **o != found) {
                return err(Diagnostic::error(format!("two CyborgPL stds are listed, `{}` and `{}`: a program has one std (owner, 2026-10-14)", other.display(), found.display())).named("manifest_given_twice").place(manifest_at.display().to_string()));
            }
            state.cord = Some(found);
            items.nerve_cord = Some(r);
            continue;
        }
        deps.insert(p.clone(), r);
        in_order.push((p, r));
    }
    // Two Kits that both forbid the path to one name cannot be told
    // apart, so depending on both is refused, as for a prefix clash
    // (owner, 2026-10-06).
    let mut only: Vec<(String, String)> = Vec::new();
    for (p, r) in &in_order {
        for f in &items.fns {
            if f.vis != Vis::Pub || items.kit_root(f.module) != *r || items.bare_marks.get(&f.name_span) != Some(&crate::ast::Bare::Only) {
                continue;
            }
            let module = items.modules[f.module.0 as usize].path.clone();
            let rel = module.rsplit("::").next().unwrap_or(&module);
            let path = if items.modules[f.module.0 as usize].parent.is_none() { f.name.clone() } else { format!("{}.{}", rel, f.name) };
            let kind = if f.op.is_some() { "op" } else { "fn" };
            let named = format!("{} {}", kind, path);
            if let Some((other, _)) = only.iter().find(|(o, n)| *n == named && o != p) {
                return err(Diagnostic::error(format!("`{}` and `{}` both forbid the path to `{}` (`bare.kit_prefix.only`), so it cannot tell them apart: edit one kit's source, as for a prefix clash", other, p, named)).named("kit_prefix_clash").outside_program().place(manifest_at.display().to_string()));
            }
            only.push((p.clone(), named));
        }
    }
    state.stack.pop();
    items.kit_deps.insert(m, deps);
    state.done.insert(key, (m, prefix.clone()));
    Ok((m, prefix))
}

/// What a `CCA.cplr` says: one CyborgPL value, `cca::Manifest[ … ];`, its
/// fields named (owner, 2026-10-06: "at least it's not much of a
/// roleplay"; one `const` per setting before), its shapes from the
/// built-in `cca` Kit.
struct Manifest {
    name: Option<(String, crate::source::Span)>,
    /// The word code writes in front of `::` to reach this Kit, which its
    /// maker picks; `name` is a label for people and may repeat (owner,
    /// 2026-09-29).
    prefix: Option<(String, crate::source::Span)>,
    deps: Vec<Dep>,
    /// The target built when none is named (owner, 2026-09-29: b).
    default_target: Option<(String, crate::source::Span)>,
    /// Every target listed.
    targets: Vec<String>,
    /// A no-OS target's linker script (owner, 2026-10-09: "`b`"): the
    /// target, the path as written (from the Neuron's folder), and where.
    layouts: Vec<(String, String, crate::source::Span)>,
    /// A no-OS target's output and exit, each a function of the Neuron
    /// named as code names it (owner, 2026-10-09: "`a`", "c"): the target,
    /// the path, where.
    outputs: Vec<(String, String, crate::source::Span)>,
    exits: Vec<(String, String, crate::source::Span)>,
    /// std is listed, by its ID and version (owner, 2026-10-06); a Kit
    /// that does not list it reaches no operator and no `std::`.
    has_std: bool,
    /// `version`, as written.
    version: Option<String>,
    /// `C.libs` (owner, 2026-10-19): the C libraries to link.
    c_libs: Vec<String>,
    /// Each target's `table_bytes` (owner, 2026-10-19), where written.
    table_bytes: Vec<(String, u64)>,
    /// The cores listed (owner, 2026-10-06).
    cores: Vec<String>,
    /// The powers this Kit uses (owner, 2026-10-07).
    powers: Vec<kayo_base::powers::Power>,
    /// What the program takes away; read in every Kit, applied only from
    /// the program's.
    deny: Vec<kayo_base::powers::Deny>,
}

struct Dep {
    path: String,
    /// Listed with cord's ID and a `path`: the Nerve cord, which takes over
    /// `cord::` for the whole program (owner, 2026-10-14).
    cord: bool,
}

use crate::ast::OFFICIAL_CORES;

const MANIFEST_SHAPE: &str = "`my details:` / `name = areas` / `prefix = areas` / `ver = 1.0.0`, then `my targets:` / `aarch64-macos {` / `}`";

/// A listed kit's prefix, from its `CyJet.toml`'s `my details:`; none when
/// it cannot be read (its own load says why).
fn kit_prefix(sources: &mut SourceMap, dir: &Path) -> Option<String> {
    let at = dir.join(MANIFEST);
    let text = std::fs::read_to_string(&at).ok()?;
    let id = sources.add_once(&at, text);
    sources.decoded(id).ok()?;
    let doc = crate::cyjet_toml::read(id, &sources.get(id).text).ok()?;
    doc.section("my details")?.pairs.iter().find(|p| p.key == "prefix").map(|p| p.value.clone())
}

/// The `pub coll.handler`s in a kit's files, by name.
fn pub_handlers(sources: &mut SourceMap, dir: &Path) -> Result<std::collections::HashMap<String, Vec<crate::parser::HandlerMember>>> {
    let mut out = std::collections::HashMap::new();
    let mut todo = vec![dir.to_path_buf()];
    while let Some(d) = todo.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else { continue };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                todo.push(p);
                continue;
            }
            if p.extension().is_none_or(|x| x != "cy") || p.file_name().is_some_and(|n| n == "build.cy") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&p) else { continue };
            if !text.contains("coll.handler") {
                continue;
            }
            let id = sources.add_once(&p, text);
            let toks = sources.lex_file(id)?;
            out.extend(crate::parser::pub_handlers(&toks)?);
        }
    }
    Ok(out)
}

/// One part of a manifest that is written once: a key in a section or a
/// block, or a block in a section.
fn once<'a>(seen: &mut Vec<&'a str>, name: &'a str, within: &str, at: crate::source::Span) -> Result<()> {
    if seen.contains(&name) {
        return err(Diagnostic::error(format!("`{}` is given twice in {}", name, within)).named("manifest_given_twice").at(at).fix_edit(at, crate::diag::Edit::DeleteLine, ""));
    }
    seen.push(name);
    Ok(())
}

/// A kit's `CyJet.toml` (owner, 2026-10-22), read by `cyjet_toml` and
/// checked here as the manifest always was.
fn read_manifest(sources: &mut SourceMap, dir: &Path) -> Result<Manifest> {
    use crate::cyjet_toml::{list, Block, Pair};
    let mut out = Manifest { name: None, prefix: None, deps: Vec::new(), default_target: None, targets: Vec::new(), layouts: Vec::new(), outputs: Vec::new(), exits: Vec::new(), has_std: false, version: None, c_libs: Vec::new(), table_bytes: Vec::new(), cores: Vec::new(), powers: Vec::new(), deny: Vec::new() };
    let at = dir.join(MANIFEST);
    if !at.is_file() {
        return Ok(out);
    }
    let text = std::fs::read_to_string(&at).map_err(|e| Box::new(Diagnostic::error(format!("cannot read `{}`: {}", at.display(), e)).named("path_not_readable")))?;
    let id: FileId = sources.add_once(&at, text);
    // A concise manifest is decoded first (design claude's reading of
    // 2026-10-22's "A manifest can be concise too").
    sources.decoded(id)?;
    let f = sources.get(id);
    let doc = crate::cyjet_toml::read(id, &f.text)?;
    let whole = crate::source::Span::new(id, 0, 0);
    // Each section's lines are blocks, or `key = value` lines.
    let no_pairs = |within: &str, pairs: &[Pair], example: &str| -> Result<()> {
        match pairs.first() {
            Some(p) => err(Diagnostic::error(format!("`{}:` holds a block for each: `{}`", within, example)).named("manifest_section_form").at(p.key_span)),
            None => Ok(()),
        }
    };

    // `my details:`: the kit's name, prefix and version.
    let details = doc.section("my details");
    if let Some(s) = details {
        if let Some(b) = s.blocks.first() {
            return err(Diagnostic::error("`my details:` holds `key = value` lines, `name`, `prefix` and `ver`, and no block").named("manifest_section_form").at(b.span));
        }
        let mut seen = Vec::new();
        for p in &s.pairs {
            once(&mut seen, &p.key, "`my details:`", p.key_span)?;
            match p.key.as_str() {
                "name" => {
                    check_word(&p.value, p.value_span, "name a kit")?;
                    out.name = Some((p.value.clone(), p.value_span));
                }
                "prefix" => {
                    check_word(&p.value, p.value_span, &format!("be a prefix, written in code as `{}::`", p.value))?;
                    out.prefix = Some((p.value.clone(), p.value_span));
                }
                // Any text, at most 256 bytes (owner, 2026-09-29).
                "ver" => {
                    if p.value.len() > 256 {
                        return err(Diagnostic::error(format!("`ver` is at most 256 bytes, and this one is {}", p.value.len())).named("manifest_version_too_long").at(p.value_span));
                    }
                    out.version = Some(p.value.clone());
                }
                other => return err(Diagnostic::error(format!("`my details:` holds `name`, `prefix` and `ver`; `{}` is none of them", other)).named("manifest_unknown_field").at(p.key_span).fix_close(p.key_span, other, ["name", "prefix", "ver"])),
            }
        }
    }
    // `my kits:`: a block for each, from the registry (`id` and `ver`) or a
    // folder (`path`); `std { path = … }` is std from its folder (owner,
    // 2026-10-14), taking over `std::` for the whole program.
    if let Some(s) = doc.section("my kits") {
        no_pairs("my kits", &s.pairs, "areas {` / `path = ../areas` / `}")?;
        let mut names = Vec::new();
        for b in &s.blocks {
            once(&mut names, &b.name, "`my kits:`", b.span)?;
            let (mut kid, mut ver, mut path): (Option<&Pair>, Option<&Pair>, Option<&Pair>) = (None, None, None);
            let mut seen = Vec::new();
            for p in &b.pairs {
                once(&mut seen, &p.key, &format!("`{}`", b.name), p.key_span)?;
                match p.key.as_str() {
                    "id" => kid = Some(p),
                    "ver" => ver = Some(p),
                    "path" => path = Some(p),
                    "name" | "prefix" => return err(Diagnostic::error("a kit's block names no word for code: code writes the kit's own `prefix`, from its `CyJet.toml`").named("manifest_kit_named_in_block").at(p.key_span).fix_edit(p.key_span, crate::diag::Edit::DeleteLine, "")),
                    other => return err(Diagnostic::error(format!("a kit's block holds `id` and `ver` (the registry) or `path` (a folder on this machine); `{}` is none of them", other)).named("manifest_unknown_field").at(p.key_span).fix_close(p.key_span, other, ["id", "ver", "path"])),
                }
            }
            if let (Some(_), Some(p)) = (kid, path) {
                return err(Diagnostic::error("a kit says where it comes from once: `id` and `ver` (the registry) or `path` (a folder on this machine), not both").named("manifest_source_unclear").at(p.key_span));
            }
            if let Some(p) = path {
                if let Some(v) = ver {
                    return err(Diagnostic::error("a kit says where it comes from once: a `path` (a folder on this machine) or a `ver` (the registry), not both").named("manifest_source_unclear").at(v.key_span));
                }
                let cord = b.name == "std";
                if cord {
                    out.has_std = true;
                } else if out.deps.iter().any(|d| d.path == p.value) {
                    return err(Diagnostic::error(format!("`{}` is listed twice in `my kits:`", p.value)).named("manifest_given_twice").at(p.value_span).fix_edit(p.value_span, crate::diag::Edit::DeleteLine, ""));
                }
                out.deps.push(Dep { path: p.value.clone(), cord });
                continue;
            }
            // The registry knows a kit by the ID its creator made — 256
            // random bits, 64 hex digits — never by its name (owner,
            // 2026-09-29: "Names could be the same, IDs can't").
            let Some(i) = kid else {
                return match ver {
                    Some(_) => err(Diagnostic::error("this kit comes from the registry, which knows a kit by its ID, never its name: add `id = …`, the 64 hex digits its creator made").named("manifest_field_missing").at(b.span)),
                    None => err(Diagnostic::error(format!("a kit's block says where it comes from: `{n} {{` / `id = …` / `ver = …` / `}}` (the registry) or `{n} {{` / `path = ../{n}` / `}}` (a folder)", n = b.name)).named("manifest_source_unclear").at(b.span)),
                };
            };
            if i.value.len() != 64 || !i.value.chars().all(|c| c.is_ascii_hexdigit()) {
                return err(Diagnostic::error(format!("a kit's `id` is 64 hex digits, the 256 random bits its creator made: `{}` is not", i.value)).named("manifest_id_form").at(i.value_span));
            }
            let Some(v) = ver else {
                return err(Diagnostic::error(format!("a registry kit says which version: `{} {{` / `id = …` / `ver = …` / `}}`", b.name)).named("manifest_field_missing").at(b.span));
            };
            // std, listed like any kit (owner, 2026-10-06): kayo knows one
            // version of it, and downloads nothing.
            if i.value == crate::parser::STD_ID {
                if v.value != crate::parser::STD_VERSION {
                    return err(Diagnostic::error(format!("kayo knows std `{}` only, and this asks for `{}`: write `ver = {}`", crate::parser::STD_VERSION, v.value, crate::parser::STD_VERSION)).named("version_not_available").at(v.value_span).fix_edit(v.value_span, crate::diag::Edit::Replace, crate::parser::STD_VERSION));
                }
                if out.has_std {
                    return err(Diagnostic::error("std is listed twice in `my kits:`").named("manifest_given_twice").at(b.span));
                }
                out.has_std = true;
                continue;
            }
            return err(Diagnostic::error("not yet: this kit asks the registry for a `ver`, and the registry does not exist yet — give its folder with `path = …`").named("not_built_yet").at(v.value_span));
        }
    }

    // `my limbs:`: each official limb by its registry ID and version (owner,
    // 2026-10-06), its block named after it (design claude's reading).
    if let Some(s) = doc.section("my limbs") {
        no_pairs("my limbs", &s.pairs, "mcps {` / `id = …` / `ver = …` / `}")?;
        let mut blocks = Vec::new();
        let mut names: Vec<String> = Vec::new();
        for b in &s.blocks {
            once(&mut blocks, &b.name, "`my limbs:`", b.span)?;
            let (mut lid, mut ver, mut path): (Option<&Pair>, Option<&Pair>, Option<&Pair>) = (None, None, None);
            let mut seen = Vec::new();
            for p in &b.pairs {
                once(&mut seen, &p.key, &format!("`{}`", b.name), p.key_span)?;
                match p.key.as_str() {
                    "id" => lid = Some(p),
                    "ver" => ver = Some(p),
                    "path" => path = Some(p),
                    other => return err(Diagnostic::error(format!("a limb's block holds `id` and `ver`, or a `path`; `{}` is none of them", other)).named("manifest_unknown_field").at(p.key_span).fix_close(p.key_span, other, ["id", "ver", "path"])),
                }
            }
            if let (Some(_), Some(p)) = (lid, path) {
                return err(Diagnostic::error("a limb says where it comes from once: an `id` and a `ver` (the registry) or a `path` (a folder on this machine)").named("manifest_source_unclear").at(p.key_span));
            }
            // Until limbs can be loaded, only the official ones (owner,
            // 2026-10-06: "`a`").
            if let Some(p) = path {
                return err(Diagnostic::error(format!("the limb at `{}` is not registered, so not verified; loading another limb is not built yet (a limb, which kayo cannot load yet)", p.value)).named("not_built_yet").at(p.value_span));
            }
            let Some(i) = lid else {
                return err(Diagnostic::error(format!("a limb's block says which limb: `{} {{` / `id = …` / `ver = …` / `}}`", b.name)).named("manifest_field_missing").at(b.span));
            };
            // MTM's ID is retired (owner, 2026-10-07: "MTM is gone.").
            if i.value == crate::ast::MTM_ID {
                return err(Diagnostic::error("the limb `9c6c7730…` is MTM, which is gone since 2026-10-07: every spelling it held is an old one now — take it out of `my limbs:`").named("limb_retired").at(i.value_span));
            }
            if let Some((n, _, when)) = crate::ast::RETIRED_CORES.iter().find(|(_, rid, _)| *rid == i.value) {
                return err(Diagnostic::error(format!("{} is retired: since {} it is part of the base; take it out of `my limbs:`", n, when)).named("limb_retired").at(i.value_span));
            }
            if let Some((n, _)) = crate::ast::BROCA_CORES.iter().find(|(_, rid)| *rid == i.value) {
                return err(Diagnostic::error(format!("{} is part of unicode since 2026-10-12: list `unicode` instead: `{}`", n, crate::ast::core_line("unicode"))).named("limb_retired").at(i.value_span));
            }
            let Some(&(n, _, known)) = OFFICIAL_CORES.iter().find(|(_, oid, _)| *oid == i.value) else {
                return err(Diagnostic::error(format!("the limb `{}…` is not an official one; loading another limb is not built yet (a limb, which kayo cannot load yet)", &i.value[..i.value.len().min(8)])).named("not_built_yet").at(i.value_span));
            };
            if !b.name.eq_ignore_ascii_case(n) {
                return err(Diagnostic::error(format!("this id is the limb `{}`'s, so its block is `{} {{ … }}`, not `{}`", n.to_lowercase(), n.to_lowercase(), b.name)).named("manifest_limb_block_name").at(b.span));
            }
            let Some(v) = ver else {
                return err(Diagnostic::error("a limb needs its `ver`, as a registry kit does").named("manifest_field_missing").at(b.span));
            };
            if v.value != known {
                return err(Diagnostic::error(format!("kayo knows {} `{}` only, the installed one; this asks for `{}`", n, known, v.value)).named("version_not_available").at(v.value_span));
            }
            if names.iter().any(|x| x == n) {
                return err(Diagnostic::error(format!("the limb {} is listed twice", n)).named("manifest_given_twice").at(b.span));
            }
            names.push(n.to_string());
        }
        out.cores = names;
    }

    // `my C libs:` (owner, 2026-10-19: "Isn't it `C.lib`?"): each named as
    // C's linker names it, `sqlite3`, never `libsqlite3.dylib`.
    if let Some(s) = doc.section("my C libs") {
        no_pairs("my C libs", &s.pairs, "sqlite3 {` / `}")?;
        let mut seen = Vec::new();
        for b in &s.blocks {
            once(&mut seen, &b.name, "`my C libs:`", b.span)?;
            if let Some(p) = b.pairs.first() {
                return err(Diagnostic::error(format!("a C library's block is empty: `{} {{` / `}}`", b.name)).named("cnc::c_lib_name_form").at(p.key_span));
            }
            let n = &b.name;
            if n.starts_with("lib") || n.contains('.') || n.contains('/') {
                let bare = n.rsplit('/').next().unwrap_or(n).trim_start_matches("lib").split('.').next().unwrap_or("").to_string();
                return err(Diagnostic::error(format!("a C library is named as the linker names it, without `lib` or a file ending: `{} {{`, not `{}`", bare, n)).named("cnc::c_lib_name_form").at(b.span));
            }
            out.c_libs.push(n.clone());
        }
    }

    // `my powers:` and `my denied powers:` (owner, 2026-10-07; 2026-10-22):
    // a block per power, named after its head; any other key narrows it,
    // `functions = write` being `c_calls.functions["write"]`, `yes` the
    // narrowing alone; `except` and `for` only where powers are denied.
    for (section, denied) in [("my powers", false), ("my denied powers", true)] {
        let Some(s) = doc.section(section) else { continue };
        no_pairs(section, &s.pairs, "c_calls {` / `}")?;
        let mut seen = Vec::new();
        for b in &s.blocks {
            once(&mut seen, &b.name, &format!("`{}:`", section), b.span)?;
            let (mut except, mut for_kits) = (Vec::new(), Vec::new());
            let mut narrowed: Vec<(crate::source::Span, kayo_base::powers::Power)> = Vec::new();
            let mut keys = Vec::new();
            for p in &b.pairs {
                once(&mut keys, &p.key, &format!("`{}`", b.name), p.key_span)?;
                match p.key.as_str() {
                    "except" | "for" if !denied => return err(Diagnostic::error(format!("`{}` belongs to `my denied powers:`; `my powers:` lists what this kit uses", p.key)).named("power_except_misplaced").at(p.key_span)),
                    "except" => except = list(&p.value),
                    "for" => for_kits = list(&p.value),
                    key => {
                        let things = if p.value == "yes" { Vec::new() } else { list(&p.value) };
                        narrowed.push((p.key_span, kayo_base::powers::Power { path: vec![b.name.clone(), key.to_string()], things }));
                    }
                }
            }
            if narrowed.is_empty() {
                narrowed.push((b.span, kayo_base::powers::Power { path: vec![b.name.clone()], things: Vec::new() }));
            }
            for (at, power) in narrowed {
                known_power(&power, at)?;
                if denied {
                    out.deny.push(kayo_base::powers::Deny { power, except: except.clone(), for_kits: for_kits.clone() });
                } else {
                    out.powers.push(power);
                }
            }
        }
    }

    // `my targets:` (owner, 2026-09-29): a block for each, holding its own
    // settings, an empty one when it has none.
    let Some(s) = doc.section("my targets").filter(|s| !s.blocks.is_empty()) else {
        return err(Diagnostic::error(format!("the manifest has no target under `my targets:`: a build needs the kit's `name`, `prefix`, `ver` and at least one target, {}", MANIFEST_SHAPE)).named("manifest_field_missing").at(doc.section("my targets").map_or(whole, |s| s.span)));
    };
    no_pairs("my targets", &s.pairs, "aarch64-macos {` / `}")?;
    let mut names: Vec<&str> = Vec::new();
    let mut lone_says_not = false;
    let mut defaults: Vec<(&Block, crate::source::Span)> = Vec::new();
    for b in &s.blocks {
        once(&mut names, &b.name, "`my targets:`", b.span)?;
        let n = &b.name;
        let mut keys = Vec::new();
        let mut said_not = false;
        for p in &b.pairs {
            once(&mut keys, &p.key, &format!("`{}`", n), p.key_span)?;
            let no_os = |field: &str| -> Result<()> {
                if n.ends_with("-none") {
                    return Ok(());
                }
                let what = match field {
                    "output" => format!("`output` is for a target with no OS, which names its own; on `{}` the OS takes what `std::print` writes", n),
                    "exit" => format!("`exit` is for a target with no OS, which names its own; on `{}` the OS takes `std::exit`'s code", n),
                    _ => format!("`layout` is for a target with no OS, whose program says where it sits in memory; on `{}` the OS places it", n),
                };
                err(Diagnostic::error(what).named("manifest_field_misplaced").at(p.value_span))
            };
            match p.key.as_str() {
                "runner" => {}
                // A no-OS target's linker script, from the kit's folder
                // (owner, 2026-10-09: "`b`").
                "layout" => {
                    no_os("layout")?;
                    out.layouts.push((n.clone(), p.value.clone(), p.value_span));
                }
                // Where `std::print`'s bytes go, and what `std::exit` calls,
                // with no OS (owner, 2026-10-09: "f", "`a`", "c").
                "output" => {
                    no_os("output")?;
                    out.outputs.push((n.clone(), p.value.clone(), p.value_span));
                }
                "exit" => {
                    no_os("exit")?;
                    out.exits.push((n.clone(), p.value.clone(), p.value_span));
                }
                // The one built when none is named (owner, 2026-09-29: b).
                "default" => match p.value.as_str() {
                    "yes" => defaults.push((b, p.key_span)),
                    "no" => said_not = true,
                    _ => return err(Diagnostic::error("a target's `default` is `yes` or `no`").named("manifest_value_form").at(p.value_span)),
                },
                // The largest table Precog may make, in bytes (owner,
                // 2026-10-19).
                "table_bytes" => {
                    let digits = p.value.replace('_', "");
                    match digits.parse::<u64>() {
                        Ok(t) if !p.value.starts_with('_') && !p.value.ends_with('_') && !p.value.contains("__") => out.table_bytes.push((n.clone(), t)),
                        _ => return err(Diagnostic::error(format!("`table_bytes` is a whole number of bytes, 0 or more: `{} {{` / `table_bytes = 4_096` / `}}`", n)).named("table_bytes_form").at(p.value_span)),
                    }
                }
                other => return err(Diagnostic::error(format!("a target's block has no `{}`: it holds a `runner` when this machine cannot run it, `default = yes` on the one built when none is named, `table_bytes`, and on one with no OS a `layout`, an `output` and an `exit`", other)).named("manifest_unknown_field").at(p.key_span)),
            }
        }
        lone_says_not = said_not;
    }
    // A single target is the default by itself; of several, exactly one
    // says so.
    if defaults.len() > 1 {
        return err(Diagnostic::error(format!("`{}` and `{}` both say `default = yes`: of several targets, exactly one says `default = yes`", defaults[0].0.name, defaults[1].0.name)).named("manifest_default_unclear").at(defaults[1].1));
    }
    if names.len() > 1 && defaults.is_empty() {
        return err(Diagnostic::error(format!("{} targets, and none is the one built when none is named: of several targets, exactly one says `default = yes`", names.len())).named("manifest_default_unclear").at(s.span));
    }
    // Only a target kayo can build (owner, 2026-09-30: "refuse").
    let known = targets();
    for b in &s.blocks {
        if !known.contains(&b.name.as_str()) {
            return err(Diagnostic::error(format!("`{}` is not a target kayo can build; it builds for {}", b.name, known.iter().map(|k| format!("`{}`", k)).collect::<Vec<_>>().join(", "))).named("target_not_buildable").at(b.span));
        }
    }
    // A lone target is the default unless it says `default = no` (owner,
    // 2026-09-30): then there is none.
    out.default_target = match defaults.first() {
        Some((b, sp)) => Some((b.name.clone(), *sp)),
        None if names.len() == 1 && lone_says_not => None,
        None => s.blocks.first().map(|b| (b.name.clone(), b.span)),
    };
    out.targets = names.iter().map(|n| n.to_string()).collect();
    // What every manifest says (owner, 2026-09-29, as a Synthia.ttsr does).
    for (need, have) in [("name", out.name.is_some()), ("prefix", out.prefix.is_some()), ("ver", out.version.is_some())] {
        if !have {
            let what = if need == "prefix" { ", the word code writes in front of `::` to reach this kit" } else { "" };
            return err(Diagnostic::error(format!("the manifest has no `{}` under `my details:`{}: a build needs the kit's `name`, `prefix`, `ver` and at least one target, {}", need, what, MANIFEST_SHAPE)).named("manifest_field_missing").at(details.map_or(whole, |s| s.span)));
        }
    }
    Ok(out)
}

/// The powers there are (owner, 2026-10-07): Signatar's three and their
/// narrowings, IEG's vouching, and the extension powers built so far.
fn known_power(power: &kayo_base::powers::Power, at: crate::source::Span) -> Result<()> {
    let known: &[&[&str]] = &[&["inline_assembly"], &["c_calls"], &["c_calls", "functions"], &["c_calls", "variables"], &["raw_memory"], &["raw_memory", "read"], &["raw_memory", "write"], &["raw_memory", "cast"], &["vouch"], &["vouch", "c"], &["operators_on_builtin_types"], &["prefix_operators"]];
    if !known.iter().any(|k| k.iter().copied().eq(power.path.iter().map(String::as_str))) {
        return err(Diagnostic::error(format!("there is no power `{}`: the powers are {}", power.path.join("."), known.iter().map(|k| format!("`{}`", k.join("."))).collect::<Vec<_>>().join(", "))).named("unknown_power").at(at));
    }
    Ok(())
}

thread_local! {
    /// The kind lines of the Kit whose files are being read: they cover
    /// every file of it.
    static BARE_KINDS: std::cell::RefCell<Vec<(crate::ast::Bare, Vec<String>, crate::source::Span)>> = const { std::cell::RefCell::new(Vec::new()) };
    /// `bare.enum_prefix.… [enum];` lines of the kit's root (owner, 2026-10-20).
    static ENUM_BARE_KINDS: std::cell::RefCell<Vec<(crate::ast::Bare, crate::source::Span)>> = const { std::cell::RefCell::new(Vec::new()) };
}

thread_local! {
    /// The `case` kind lines of the Kit whose files are being read.
    static CASE_LINES: std::cell::RefCell<Vec<(bool, Vec<String>, crate::source::Span)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Which names match in any case (owner, 2026-10-06): a name is covered by
/// its item's mark, the block modules around it, its file's top line, and
/// its kind in `limb.cpls`; the default is exact. Marks that cover one name
/// and disagree are an error, and so are two names of one module that
/// differ only in case when either is insensitive. Fields and cases take
/// the `field` and `case` kinds, variables `var`: no item mark covers them.
fn check_case(sources: &SourceMap, items: &mut Items, file: &crate::ast::File, root: bool) -> Result<()> {
    use crate::ast::{fold_case, Item, ItemKind};
    use crate::source::Span;
    if root {
        CASE_LINES.with(|k| *k.borrow_mut() = file.case_kinds.clone());
    } else if let Some((_, _, at)) = file.case_kinds.first() {
        return err(Diagnostic::error("a `case` mark on kinds (`case.any [fn, struct];`) covers the whole kit, so it is written in `kit.cy`").named("unicode::case_mark_misplaced").at(*at));
    }
    let lines = CASE_LINES.with(|k| k.borrow().clone());
    let line = |at: Span| sources.get(at.file).line_of(at.start);
    let spelled = |b: bool| if b { "case.any" } else { "case.exact" };
    let decide = |marks: &[(bool, Span)], what: &str, at: Span| -> Result<bool> {
        if let (Some(i), Some(s)) = (marks.iter().find(|m| m.0), marks.iter().find(|m| !m.0)) {
            return err(Diagnostic::error(format!("{} is marked both `{}` ({}) and `{}` ({}): marks that disagree are an error (owner, 2026-10-06)", what, spelled(true), line(i.1), spelled(false), line(s.1))).named("unicode::case_marks_disagree").at(at));
        }
        Ok(marks.first().is_some_and(|m| m.0))
    };
    let of_kind = |k: &str| lines.iter().filter(|(_, ks, _)| ks.iter().any(|x| x == k)).map(|(b, _, at)| (*b, *at)).collect::<Vec<_>>();
    // One name space: its names, folded, with where and whether insensitive.
    let one_name = |seen: &mut Vec<(String, Span, bool)>, name: &str, shown: &dyn Fn(&str) -> String, at: Span, ins: bool| -> Result<()> {
        let f = fold_case(name);
        if let Some((other, other_at, other_ins)) = seen.iter().find(|(n, _, _)| *n != name && fold_case(n) == f) {
            if ins || *other_ins {
                let which = if *other_ins { other.as_str() } else { name };
                return err(Diagnostic::error(format!("{} and {} ({}) differ only in case, and {} is `case.any`: they would be one name", shown(name), shown(other), line(*other_at), shown(which))).named("unicode::names_differ_only_in_case").at(at));
            }
        }
        seen.push((name.to_string(), at, ins));
        Ok(())
    };
    let plain = |n: &str| format!("`{}`", n);
    let quoted = |n: &str| format!("`${}`", n);
    fn walk(items: &mut Items, list: &[Item], chain: &[(bool, Span)], cx: &dyn Fn(&mut Items, &Item, &[(bool, Span)], &mut Vec<(String, Span, bool)>, &mut Vec<(String, Span, bool)>) -> Result<()>) -> Result<()> {
        let (mut names, mut consts) = (Vec::new(), Vec::new());
        for it in list {
            if let ItemKind::Module { items: inner, .. } = &it.kind {
                let mut c: Vec<(bool, Span)> = it.case.into_iter().collect();
                c.extend_from_slice(chain);
                walk(items, inner, &c, cx)?;
                continue;
            }
            cx(items, it, chain, &mut names, &mut consts)?;
        }
        Ok(())
    }
    let chain: Vec<(bool, Span)> = file.case_file.into_iter().collect();
    walk(items, &file.items, &chain, &|items, it, chain, names, consts| {
        let marks = |kind: Option<&str>, own: bool| {
            let mut m: Vec<(bool, Span)> = if own { it.case.into_iter().collect() } else { Vec::new() };
            m.extend_from_slice(chain);
            if let Some(k) = kind {
                m.extend(of_kind(k));
            }
            m
        };
        let (word, name) = match &it.kind {
            ItemKind::Fn(d) | ItemKind::ExternFn(d) => ("fn", &d.name),
            ItemKind::Struct(d) => ("struct", &d.name),
            ItemKind::Enum(d) => ("enum", &d.name),
            ItemKind::Interface(d) => ("interface", &d.name),
            ItemKind::Const(d) => {
                let ins = decide(&marks(Some("const"), true), &format!("`const ${}`", d.name.name), d.name.span)?;
                if ins {
                    items.case_names.insert(d.name.span);
                }
                return one_name(consts, &d.name.name, &quoted, d.name.span, ins);
            }
            ItemKind::Op(d) => ("op", &d.op),
            ItemKind::Main(..) => {
                if decide(&marks(Some("var"), false), "`MAIN`'s variables", it.span)? {
                    items.case_vars.insert(it.span);
                }
                return Ok(());
            }
            _ => return Ok(()),
        };
        let kind = (word != "op").then_some(word);
        let ins = decide(&marks(kind, true), &format!("`{} {}`", word, name.name), name.span)?;
        if ins {
            items.case_names.insert(name.span);
        }
        if word != "op" {
            one_name(names, &name.name, &plain, name.span, ins)?;
        }
        match &it.kind {
            ItemKind::Struct(d) => {
                if decide(&marks(Some("field"), false), &format!("the fields of `{}`", name.name), name.span)? {
                    items.case_fields.insert(name.span);
                    let mut seen = Vec::new();
                    for f in &d.fields {
                        one_name(&mut seen, &f.name.name, &plain, f.name.span, true)?;
                    }
                }
            }
            ItemKind::Enum(d) => {
                if decide(&marks(Some("case"), false), &format!("the cases of `{}`", name.name), name.span)? {
                    items.case_cases.insert(name.span);
                    let mut seen = Vec::new();
                    for c in &d.cases {
                        one_name(&mut seen, &c.name.name, &plain, c.name.span, true)?;
                    }
                }
            }
            ItemKind::Fn(_) | ItemKind::Op(_) => {
                if decide(&marks(Some("var"), false), &format!("the variables of `{}`", name.name), name.span)? {
                    items.case_vars.insert(name.span);
                }
            }
            _ => {}
        }
        Ok(())
    })
}

/// The `bare.` marks are gone (owner, 2026-10-22: "What IF there's no
/// bare.optional? It's always not bare."): another kit's names always take
/// its path, and your own stay bare. A mark left in a kit is refused, std/'s
/// too since std claude took its own out (oracle 421c9e98).
fn check_bare(_sources: &SourceMap, _items: &mut Items, file: &crate::ast::File, _base: &str, _root: bool) -> Result<()> {
    use crate::ast::{Item, ItemKind};
    fn first_mark(items: &[Item]) -> Option<crate::source::Span> {
        for it in items {
            if let Some((_, at)) = it.bare.or(it.enum_bare) {
                return Some(at);
            }
            if let ItemKind::Module { items, .. } = &it.kind {
                if let Some(at) = first_mark(items) {
                    return Some(at);
                }
            }
        }
        None
    }
    let at = file.bare_file.map(|(_, a)| a).or(file.bare_kinds.first().map(|(_, _, a)| *a)).or(file.enum_bare_file.map(|(_, a)| a)).or(file.enum_bare_kinds.first().map(|(_, a)| *a)).or_else(|| first_mark(&file.items));
    match at {
        Some(at) => err(Diagnostic::error(BARE_MARKS_GONE).named("spelling_removed").at(at)),
        None => Ok(()),
    }
}

/// What a `bare.` mark is told (owner, 2026-10-22).
pub const BARE_MARKS_GONE: &str = "the `bare.` marks are gone (owner, 2026-10-22): another kit's names always take its path, and your own stay bare; drop the mark";

fn new_module(items: &mut Items, name: &str, parent: Option<ModuleId>, vis: Vis) -> ModuleId {
    let path = crate::resolve::module_path(items, name, parent);
    let id = ModuleId(items.modules.len() as u32);
    items.modules.push(crate::resolve::Module { name: name.to_string(), path, parent, vis, items: Default::default(), consts: Default::default(), folded: Default::default(), folded_consts: Default::default() });
    id
}

fn parse_file(sources: &mut SourceMap, path: &Path) -> Result<crate::ast::File> {
    let text = std::fs::read_to_string(path).map_err(|e| Box::new(Diagnostic::error(format!("cannot read `{}`: {}", path.display(), e)).named("path_not_readable")))?;
    let id: FileId = sources.add_once(path, text);
    let toks = sources.lex_file(id)?;
    let file = crate::parser::parse_file(&toks)?;
    // A template's generated part is read, never written, by a build
    // (owner, 2026-10-13): it must be the one the template makes now.
    crate::template::check(&file, &sources.get(id).text)?;
    Ok(file)
}

/// The entries of `dir` as modules of `parent`.
fn load_dir(sources: &mut SourceMap, items: &mut Items, dir: &Path, parent: ModuleId, is_root_dir: bool) -> Result<()> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir).map_err(|e| Box::new(Diagnostic::error(format!("cannot read `{}`: {}", dir.display(), e)).named("path_not_readable")))?.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    // Files first, so a directory `x/` can nest under a file module `x.cpls`.
    let mut file_modules: Vec<(String, ModuleId)> = Vec::new();
    for e in entries.iter().filter(|p| p.is_file()) {
        let Some(ext) = e.extension() else { continue };
        if ext != "cy" {
            continue;
        }
        let stem = e.file_stem().unwrap().to_string_lossy().to_string();
        // `build.nrv` is the program `cca prepare` runs, not a module
        // (owner, 2026-10-09: "yea"; design claude's reading).
        if is_root_dir && (stem == "kit" || stem == "build") {
            continue;
        }
        check_name(&stem, e)?;
        if let Some((existing, _, _)) = items.modules[parent.0 as usize].items.get(&stem) {
            if let crate::resolve::ItemRef::Module(_) = existing {
                return err(Diagnostic::error(format!("`{}` is both a file module (`{}`) and a `module {} {{ … }}` block: a module cannot be both — keep one", stem, e.display(), stem)).named("defined_twice"));
            }
            return err(Diagnostic::error(format!("`{}` names both the file module `{}` and a {} declared in its parent: a module and a type cannot share a name", stem, e.display(), existing.what())).named("defined_twice"));
        }
        let m = new_module(items, &stem, Some(parent), Vis::Pub);
        items.modules[parent.0 as usize].items.insert(stem.clone(), (crate::resolve::ItemRef::Module(m), Vis::Pub, crate::source::Span::new(FileId(0), 0, 0)));
        let file = parse_file(sources, e)?;
        let base = items.modules[m.0 as usize].path.clone();
        check_bare(sources, items, &file, base.rsplit("::").next().unwrap_or(&base), false)?;
        check_case(sources, items, &file, false)?;
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
        (p.is_file() && p.extension().map(|x| x == "cy").unwrap_or(false)) || (p.is_dir() && has_cpls(&p))
    })).unwrap_or(false)
}

fn check_name(name: &str, at: &Path) -> Result<()> {
    // `neuron.nrv` is a Neuron's root file (owner, 2026-10-09; `kit`
    // before it).
    if name == "kit" {
        return err(Diagnostic::error(format!("`neuron` names a kit's root file (`kit.cy`), so it cannot name a module: rename `{}`", short(at))).named("file_name_not_module"));
    }
    let ok = name.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false) && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if !ok {
        return err(Diagnostic::error(format!("`{}` cannot name a module: module names are lowercase ASCII letters, digits and `_`, starting with a letter — rename `{}`", name, short(at))).named("file_name_not_module"));
    }
    if crate::parser::is_device_name(name) {
        return err(Diagnostic::error(format!("`{}` is a reserved device name on Windows and cannot name a module: rename `{}`", name, short(at))).named("file_name_not_module"));
    }
    if crate::parser::reserved_module_name(name) {
        return err(Diagnostic::error(format!("`{}` is a keyword and cannot name a module: rename `{}`", name, short(at))).named("file_name_not_module"));
    }
    Ok(())
}

/// A Kit's 'name' and 'prefix' follow the module-name rule (owner,
/// 2026-09-29 for the prefix, written in code as `prefix::`; 2026-09-30
/// for the name, "strict"). `what` finishes "cannot …".
fn check_word(w: &str, span: crate::source::Span, what: &str) -> Result<()> {
    let why = if w.chars().any(|c| c.is_ascii_uppercase()) {
        "lowercase only — macOS and Windows file systems do not tell `Shapes` from `shapes`".to_string()
    } else if !w.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false) {
        "it must start with a lowercase ASCII letter".to_string()
    } else if let Some(c) = w.chars().find(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_')) {
        format!("`{}` is not allowed — lowercase ASCII letters, digits, and `_` only", c)
    } else if crate::parser::is_device_name(w) {
        "it is a reserved device name on Windows".to_string()
    } else if crate::parser::is_keyword(w) {
        "it is a keyword".to_string()
    } else {
        return Ok(());
    };
    err(Diagnostic::error(format!("`{}` cannot {}: {}", w, what, why)).named("manifest_name_form").at(span))
}

/// The file's own name, for a message: a full path is noise and differs per machine.
fn short(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| p.display().to_string())
}
