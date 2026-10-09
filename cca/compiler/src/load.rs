//! Loading a Limb: one `.cpls` file, or a directory whose `limb.cpls` is
//! the root and whose other files and directories are its modules
//! (`limbs.md`).

use std::path::{Path, PathBuf};

use crate::ast::Vis;
use crate::diag::{err, Diagnostic, Result};
use crate::resolve::{Items, ModuleId};
use crate::source::{FileId, SourceMap};

/// What a Limb is loaded for: checking reads it whatever its targets;
/// compiling builds one target — the one named on the command line, or
/// the Limb's default.
pub enum Purpose<'a> {
    Check,
    Compile(Option<&'a str>),
}

pub fn load(path: &Path, sources: &mut SourceMap) -> Result<Items> {
    load_for(path, sources, Purpose::Check)
}

/// Every `precedence` block of the files this load reaches — the Limb's
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
                        if !name.starts_with('.') && name != "target" && !e.join("CCA.cplr").is_file() {
                            cpls_under(&e, out);
                        }
                    } else if e.extension().is_some_and(|x| x == "cpls") {
                        out.push(e);
                    }
                }
            }
            cpls_under(&d, &mut files);
            if d.join("CCA.cplr").is_file() {
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
        let toks = crate::lexer::lex(id, &sources.get(id).text)?;
        for (name, (level, chain)) in crate::parser::precedence_blocks(&toks)? {
            match table.get(&name) {
                Some((l, c, other)) if (*l, *c) != (level, chain) => {
                    return err(Diagnostic::error(format!("`{}` has two levels: {} and {} each give it one, and an operator binds one way wherever it is used — one block says it", name, other.display(), f.display())));
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
    gather_precedence(sources, path)?;
    let mut items = Items::default();
    // The root Limb's targets and default, when it has a `CCA.cplr`.
    let mut root: Option<(Vec<String>, Option<(String, crate::source::Span)>)> = None;
    if path.is_dir() {
        let mut state = Loading { stack: Vec::new(), done: std::collections::HashMap::new(), prefixes: std::collections::HashMap::new(), root: None };
        load_limb(sources, &mut items, path, None, &mut state)?;
        root = state.root;
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
                return err(Diagnostic::error(format!("`{}` is not one of this Neuron's targets: its `CCA.cplr` lists {}", t, listed.iter().map(|l| format!("`{}`", l)).collect::<Vec<_>>().join(", "))));
            }
            Some((t.to_string(), None))
        }
        (Purpose::Compile(Some(t)), None) => {
            let known = targets();
            if !known.contains(t) {
                return err(Diagnostic::error(format!("`{}` is not a target CCA can build; it builds for {}", t, known.iter().map(|k| format!("`{}`", k)).collect::<Vec<_>>().join(", "))));
            }
            Some((t.to_string(), None))
        }
        (Purpose::Compile(None), Some((listed, None))) => {
            return err(Diagnostic::error(format!("this Neuron has no default target — its one target says `default = false` — so a build names the target: `--target {}`", listed.first().map(String::as_str).unwrap_or("aarch64-macos"))));
        }
        (Purpose::Compile(None), Some((_, Some((t, s))))) => Some((t.clone(), Some(*s))),
        (Purpose::Compile(None), None) => None,
    };
    if let Some((t, span)) = chosen {
        let here = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
        if t != here {
            let d = Diagnostic::error(format!("not yet: the target is `{}`, and CCA builds it only on a `{}` machine so far — this one is `{}`", t, t, here));
            return err(match span {
                Some(s) => d.at(s),
                None => d,
            });
        }
    }
    if !path.is_dir() {
        let m = new_module(&mut items, "limb", None, Vis::Pub);
        let file = parse_file(sources, path)?;
        check_bare(sources, &mut items, &file, "", true)?;
        check_case(sources, &mut items, &file, true)?;
        items.collect_file(&file, m, true)?;
    }
    items.resolve_all()?;
    Ok(items)
}

/// The targets CCA writes code for: `cca/targets.txt`, the one list,
/// which Synthia copies.
pub fn targets() -> Vec<&'static str> {
    include_str!("../../targets.txt").lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')).collect()
}

/// Which Limbs `cca prepare` runs (owner, 2026-09-29 and 2026-09-30).
pub enum Pick {
    /// `--all`: every Limb reached, dependencies first.
    All,
    /// Limbs named, in the order given.
    Listed(Vec<Wanted>),
}

/// One Limb asked for by `cca prepare`.
pub enum Wanted {
    /// By its `name`: this Limb's own or a dependency's.
    Name(String),
    /// By its prefix (`--prefix geo2`), for a name two Limbs share.
    Prefix(String),
}

/// What `cca prepare` runs, in order: each Limb's `build.cplr` with the
/// name it goes by and the folder it runs in. Names run in the order given,
/// each once; `--all` runs dependencies before the Limbs that use them,
/// each once. Nothing is loaded but manifests.
pub fn prepare_plan(sources: &mut SourceMap, dir: &Path, pick: &Pick) -> Result<Vec<(String, PathBuf)>> {
    if !dir.join("limb.cpls").is_file() {
        return err(Diagnostic::error(format!("`cca prepare` acts on a Neuron, and `{}` has no `limb.cpls`", dir.display())));
    }
    // A Limb's name and prefix, from its own `CCA.cplr`; a dependency
    // without one has neither.
    fn words_of(sources: &mut SourceMap, dir: &Path, from: Option<(&Path, &str)>) -> Result<(String, String)> {
        if let Some((manifest_at, written)) = from {
            if !dir.join("CCA.cplr").is_file() {
                return err(Diagnostic::error(format!("the Neuron at `{}` has no `CCA.cplr`, so no `prefix` for code to reach it by", written)).place(manifest_at.display().to_string()));
            }
        }
        let m = read_manifest(sources, dir)?;
        let folder = dir.canonicalize().ok().and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string())).unwrap_or_else(|| "this Neuron".into());
        Ok((m.name.map(|(n, _)| n).unwrap_or_else(|| folder.clone()), m.prefix.map(|(p, _)| p).unwrap_or(folder)))
    }
    // How a Limb is shown: its name, and its prefix too where that differs,
    // since two Limbs may share a name.
    fn label(name: &str, prefix: &str) -> String {
        if name == prefix { name.to_string() } else { format!("{} (prefix `{}`)", name, prefix) }
    }
    let manifest = read_manifest(sources, dir)?;
    let manifest_at = dir.join("CCA.cplr");
    let (own_name, own_prefix) = words_of(sources, dir, None)?;
    let names = match pick {
        Pick::All => {
            // Dependencies first: a Limb is listed after everything under it.
            fn walk(sources: &mut SourceMap, dir: &Path, shown: String, stack: &mut Vec<(PathBuf, String)>, done: &mut Vec<PathBuf>, out: &mut Vec<(String, PathBuf)>) -> Result<()> {
                let key = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
                if done.contains(&key) {
                    return Ok(());
                }
                if let Some(at) = stack.iter().position(|(p, _)| *p == key) {
                    let mut names: Vec<String> = stack[at..].iter().map(|(_, n)| n.clone()).collect();
                    names.push(shown);
                    return err(Diagnostic::error(format!("the dependencies go round in a circle: {} — a Neuron cannot depend on itself, however far round", names.join(" → "))));
                }
                let manifest = read_manifest(sources, dir)?;
                let manifest_at = dir.join("CCA.cplr");
                stack.push((key.clone(), shown.clone()));
                for d in &manifest.deps {
                    let at = dir.join(&d.path);
                    let (n, p) = words_of(sources, &at, Some((&manifest_at, &d.path)))?;
                    walk(sources, &at, label(&n, &p), stack, done, out)?;
                }
                stack.pop();
                done.push(key);
                if dir.join("build.cplr").is_file() {
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
    // This Limb and its dependencies: name, prefix, folder.
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
            [] if by_prefix => return err(Diagnostic::error(format!("no Neuron here has the prefix `{}`: the prefixes here are {}", n, known.iter().map(|(_, p, _)| format!("`{}`", p)).collect::<Vec<_>>().join(", ")))),
            [] => {
                let mut names: Vec<String> = Vec::new();
                for (k, _, _) in &known {
                    if !names.contains(k) {
                        names.push(k.clone());
                    }
                }
                return err(Diagnostic::error(format!("no Neuron here is named `{}`: the names here are {}", n, names.iter().map(|k| format!("`{}`", k)).collect::<Vec<_>>().join(", "))));
            }
            [one] => (label(&one.0, &one.1), one.2.clone()),
            // Two Limbs may share a name, never a prefix: which one is
            // meant is said by prefix (owner, 2026-09-30: a, "flag").
            many => {
                let prefixes: Vec<String> = many.iter().map(|(_, p, _)| format!("`{}`", p)).collect();
                return err(Diagnostic::error(format!("{} Neurons here are named `{}`, with the prefixes {}: pick one with `cca prepare --prefix {}`", many.len(), n, prefixes.join(", "), many[0].1)));
            }
        };
        let (shown, at) = at;
        // Each Limb once, however it was asked for.
        let key = std::fs::canonicalize(&at).unwrap_or_else(|_| at.clone());
        if ran.contains(&key) {
            continue;
        }
        ran.push(key);
        if !at.join("build.cplr").is_file() {
            return err(Diagnostic::error(format!("`{}` has no `build.cplr` to run, at `{}`", n, at.display())));
        }
        out.push((shown, at));
    }
    Ok(out)
}

/// Where loading has got to: the Limbs being loaded now (a dependency back
/// to one of them is a cycle), those loaded already (two Limbs that share a
/// dependency share one copy of it), and each dependency's prefix with the
/// folder that has it.
struct Loading {
    stack: Vec<(PathBuf, String)>,
    done: std::collections::HashMap<PathBuf, (ModuleId, String)>,
    prefixes: std::collections::HashMap<String, PathBuf>,
    /// The Limb being built: its targets and default, from its `CCA.cplr`.
    root: Option<(Vec<String>, Option<(String, crate::source::Span)>)>,
}

/// One Limb directory and, through its `CCA.cplr`, its dependencies: each
/// a tree of its own, its root's `limb_deps` naming theirs by their own
/// 'prefix' (owner, 2026-09-29). `from` is, for a dependency, the depending
/// Limb's `CCA.cplr` and the path it wrote; the Limb being built has none,
/// and its root is `ROOT`, made first. Gives back the Limb's root module
/// and its prefix.
fn load_limb(sources: &mut SourceMap, items: &mut Items, dir: &Path, from: Option<(&Path, &str)>, state: &mut Loading) -> Result<(ModuleId, String)> {
    let key = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    if let Some(r) = state.done.get(&key) {
        return Ok(r.clone());
    }
    if let Some(at) = state.stack.iter().position(|(p, _)| *p == key) {
        let mut names: Vec<String> = state.stack[at..].iter().map(|(_, n)| n.clone()).collect();
        names.push(state.stack[at].1.clone());
        return err(Diagnostic::error(format!("the dependencies go round in a circle: {} — a Neuron cannot depend on itself, however far round", names.join(" → "))));
    }
    let root = dir.join("limb.cpls");
    if !root.is_file() {
        return err(Diagnostic::error(match from {
            None => format!("`{}` is a Neuron directory, and a Neuron needs its root file `limb.cpls`: none is there", short(dir)),
            Some((_, written)) => format!("the Neuron at `{}` needs its root file `limb.cpls`: none is there", written),
        }));
    }
    // A dependency is reached by the prefix its own `CCA.cplr` gives it,
    // and the depending Limb names no word (owner, 2026-09-29).
    if let Some((manifest_at, written)) = from {
        if !dir.join("CCA.cplr").is_file() {
            return err(Diagnostic::error(format!("the Neuron at `{}` has no `CCA.cplr`, so no `prefix` for code to reach it by", written)).place(manifest_at.display().to_string()));
        }
    }
    let manifest = read_manifest(sources, dir)?;
    let prefix = match &manifest.prefix {
        Some((p, _)) => p.clone(),
        None => dir.canonicalize().ok().and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string())).unwrap_or_else(|| "this Neuron".into()),
    };
    if let Some((manifest_at, written)) = from {
        // `cord` is reserved beside the old `std` and `stdkit` (owner,
        // 2026-10-09).
        if matches!(prefix.as_str(), "std" | "stdkit" | "cord" | "cca") {
            return err(Diagnostic::error(format!("the Neuron at `{}` has the prefix `{}`, which names the built-in `{}::`; edit its `prefix`", written, prefix, if prefix == "cca" { "cca" } else { "cord" })).place(manifest_at.display().to_string()));
        }
        // No override: two Limbs with one prefix are fixed by editing one
        // (owner, 2026-09-29: "Just edit the source code").
        if let Some(other) = state.prefixes.get(&prefix) {
            if *other != key {
                return err(Diagnostic::error(format!("two Neurons have the prefix `{}` ({} and {}); edit one Neuron's `prefix`", prefix, other.display(), key.display())).place(manifest_at.display().to_string()));
            }
        }
        state.prefixes.insert(prefix.clone(), key.clone());
    }
    if from.is_none() && dir.join("CCA.cplr").is_file() {
        state.root = Some((manifest.targets.clone(), manifest.default_target.clone()));
    }
    let m = match from {
        None => new_module(items, "limb", None, Vis::Pub),
        Some(_) => {
            let id = ModuleId(items.modules.len() as u32);
            items.modules.push(crate::resolve::Module { name: "limb".to_string(), path: format!("{}::limb", prefix), parent: None, vis: Vis::Pub, items: Default::default(), consts: Default::default(), folded: Default::default(), folded_consts: Default::default() });
            id
        }
    };
    // A Limb without std reaches none of it; its dependencies are judged
    // by their own `CCA.cplr` (owner, 2026-10-06).
    crate::parser::set_no_std(dir.join("CCA.cplr").is_file() && !manifest.has_std);
    crate::parser::set_cores(dir.join("CCA.cplr").is_file().then(|| manifest.cores.clone()));
    let first_file = sources.files.len();
    let parsed = (|| {
        let mut file = parse_file(sources, &root)?;
        check_bare(sources, items, &file, "", true)?;
        check_case(sources, items, &file, true)?;
        // A dependency's `MAIN` runs only when that Limb itself is run, and
        // is left out when another Limb uses it (owner, 2026-09-29: a).
        if from.is_some() {
            file.items.retain(|i| !matches!(i.kind, crate::ast::ItemKind::Main(..)));
        }
        items.collect_file(&file, m, from.is_none())?;
        load_dir(sources, items, dir, m, true)
    })();
    crate::parser::set_no_std(false);
    crate::parser::set_cores(None);
    parsed?;
    // Without ASP a literal is a bit pattern, as the machine's (owner,
    // 2026-10-06).
    if dir.join("CCA.cplr").is_file() && !manifest.cores.iter().any(|c| c == "ASP") {
        items.raw_files.extend((first_file..sources.files.len()).map(|f| FileId(f as u32)));
    }
    if dir.join("CCA.cplr").is_file() && !manifest.has_std {
        items.no_std_roots.insert(m);
    }
    if dir.join("CCA.cplr").is_file() && !manifest.cores.iter().any(|c| c == "IEG") {
        items.no_ieg_roots.insert(m);
    }
    // The power ledger's view of this Limb (owner, 2026-10-07): its list,
    // its cores; and, for the program, its `deny`.
    if dir.join("CCA.cplr").is_file() {
        items.limb_powers.insert(m, crate::resolve::LimbPowers { prefix: prefix.clone(), is_program: from.is_none(), lists: manifest.powers.clone(), cores: manifest.cores.clone() });
        if from.is_none() {
            items.program_deny = manifest.deny.clone();
        }
    }
    state.stack.push((key.clone(), prefix.clone()));
    let mut deps = std::collections::HashMap::new();
    let mut in_order: Vec<(String, ModuleId)> = Vec::new();
    let manifest_at = dir.join("CCA.cplr");
    for d in &manifest.deps {
        let at = dir.join(&d.path);
        let (r, p) = load_limb(sources, items, &at, Some((&manifest_at, &d.path)), state)?;
        deps.insert(p.clone(), r);
        in_order.push((p, r));
    }
    // Two Limbs that both forbid the path to one name cannot be told
    // apart, so depending on both is refused, as for a prefix clash
    // (owner, 2026-10-06).
    let mut only: Vec<(String, String)> = Vec::new();
    for (p, r) in &in_order {
        for f in &items.fns {
            if f.vis != Vis::Pub || items.limb_root(f.module) != *r || items.bare_marks.get(&f.name_span) != Some(&crate::ast::Bare::Only) {
                continue;
            }
            let module = items.modules[f.module.0 as usize].path.clone();
            let rel = module.rsplit("::").next().unwrap_or(&module);
            let path = if items.modules[f.module.0 as usize].parent.is_none() { f.name.clone() } else { format!("{}.{}", rel, f.name) };
            let kind = if f.op.is_some() { "op" } else { "fn" };
            let named = format!("{} {}", kind, path);
            if let Some((other, _)) = only.iter().find(|(o, n)| *n == named && o != p) {
                return err(Diagnostic::error(format!("`{}` and `{}` both forbid the path to `{}` (`bare.only`), so it cannot tell them apart: edit one Neuron's source, as for a prefix clash", other, p, named)).place(manifest_at.display().to_string()));
            }
            only.push((p.clone(), named));
        }
    }
    state.stack.pop();
    items.limb_deps.insert(m, deps);
    state.done.insert(key, (m, prefix.clone()));
    Ok((m, prefix))
}

/// What a `CCA.cplr` says: one CyborgPL value, `cca::Manifest[ … ];`, its
/// fields named (owner, 2026-10-06: "at least it's not much of a
/// roleplay"; one `const` per setting before), its shapes from the
/// built-in `cca` Limb.
struct Manifest {
    name: Option<(String, crate::source::Span)>,
    /// The word code writes in front of `::` to reach this Limb, which its
    /// maker picks; `name` is a label for people and may repeat (owner,
    /// 2026-09-29).
    prefix: Option<(String, crate::source::Span)>,
    deps: Vec<Dep>,
    /// The target built when none is named (owner, 2026-09-29: b).
    default_target: Option<(String, crate::source::Span)>,
    /// Every target listed.
    targets: Vec<String>,
    /// std is listed, by its ID and version (owner, 2026-10-06); a Limb
    /// that does not list it reaches no operator and no `std::`.
    has_std: bool,
    /// The cores listed (owner, 2026-10-06).
    cores: Vec<String>,
    /// The powers this Limb uses (owner, 2026-10-07).
    powers: Vec<cca_base::powers::Power>,
    /// What the program takes away; read in every Limb, applied only from
    /// the program's.
    deny: Vec<cca_base::powers::Deny>,
}

struct Dep {
    path: String,
}

const FIELDS: [&str; 8] = ["name", "prefix", "version", "targets", "dependencies", "cores", "powers", "deny"];

use crate::ast::OFFICIAL_CORES;

const MANIFEST_SHAPE: &str = "`cca::Manifest[name = \"areas\", prefix = \"areas\", version = \"1.0.0\", targets = [cca::Target[name = \"aarch64-macos\"],]];`";

fn read_manifest(sources: &mut SourceMap, dir: &Path) -> Result<Manifest> {
    use crate::ast::{Arg as AArg, ExprKind};
    use crate::lexer::Tok;
    let mut out = Manifest { name: None, prefix: None, deps: Vec::new(), default_target: None, targets: Vec::new(), has_std: false, cores: Vec::new(), powers: Vec::new(), deny: Vec::new() };
    let at = dir.join("CCA.cplr");
    if !at.is_file() {
        return Ok(out);
    }
    let text = std::fs::read_to_string(&at).map_err(|e| Box::new(Diagnostic::error(format!("cannot read `{}`: {}", at.display(), e))))?;
    let id: FileId = sources.add(&at, text);
    let toks = crate::lexer::lex(id, &sources.get(id).text)?;
    if matches!(&toks[0].tok, Tok::Word(w) if w == "const" || w == "pub") {
        return err(Diagnostic::error(format!("CCA.cplr's settings are no longer `const`s: the file is one value, {}", MANIFEST_SHAPE)).at(toks[0].span));
    }
    let value = crate::parser::parse_value(&toks)?;
    let fields = match &value.kind {
        ExprKind::Call { path, args, .. } if path.limb.as_ref().map(|l| l.name.as_str()) == Some("cca") && path.segs.iter().map(|s| s.name.as_str()).eq(["Manifest"]) => args,
        _ => return err(Diagnostic::error(format!("CCA.cplr is one value: {}", MANIFEST_SHAPE)).at(value.span)),
    };
    let mut seen: Vec<String> = Vec::new();
    let text = |e: &crate::ast::Expr, what: &str| -> Result<String> {
        match &e.kind {
            ExprKind::Str(s) => Ok(s.clone()),
            _ => err(Diagnostic::error(format!("{} is text: `\"…\"`", what)).at(e.span)),
        }
    };
    for a in fields {
        let AArg::Named(field, v) = a else {
            return err(Diagnostic::error("a `cca::Manifest` names its fields: `name = …`, `prefix = …`, `version = …`, `targets = […]`").at(a.span()));
        };
        let setting = field.name.as_str();
        // One target became a list (owner, 2026-09-29).
        if setting == "target" {
            return err(Diagnostic::error("a Neuron lists its targets, so the field is `targets`: `targets = [cca::Target[name = \"aarch64-macos\"],]`").at(field.span));
        }
        // What a Limb may do (`design/cores.md`): which powers exist is
        // not decided.
        if !FIELDS.contains(&setting) {
            return err(Diagnostic::error(format!("a `cca::Manifest` has no field `{}`: its fields are {}", setting, FIELDS.iter().map(|s| format!("`{}`", s)).collect::<Vec<_>>().join(", "))).at(field.span));
        }
        if seen.iter().any(|s| s == setting) {
            return err(Diagnostic::error(format!("`{}` is given twice in the `cca::Manifest`", setting)).at(field.span));
        }
        seen.push(setting.to_string());
        match setting {
            "name" => {
                let n = text(v, "`name`")?;
                check_word(&n, v.span, "name a Neuron")?;
                out.name = Some((n, v.span));
            }
            "prefix" => {
                let p = text(v, "`prefix`")?;
                check_word(&p, v.span, &format!("be a prefix, written in code as `{}::`", p))?;
                out.prefix = Some((p, v.span));
            }
            // Any text, at most 256 bytes (owner, 2026-09-29), as in a
            // Synthia.ttsr.
            "version" => {
                let version = text(v, "`version`")?;
                if version.len() > 256 {
                    return err(Diagnostic::error(format!("`version` is at most 256 bytes, and this one is {}", version.len())).at(v.span));
                }
            }
            // The targets a Limb is built for (owner, 2026-09-29): each a
            // `cca::Target` with a `name` and, for one this machine cannot
            // run, a `runner`. CCA reads them and still builds for this
            // machine: which one is built when none is named is open.
            "targets" => {
                let ExprKind::List(entries) = &v.kind else {
                    return err(Diagnostic::error("`targets` is a list of `cca::Target[…]`: `[cca::Target[name = \"aarch64-macos\"],]`").at(v.span));
                };
                let mut names: Vec<String> = Vec::new();
                let mut name_spans: Vec<crate::source::Span> = Vec::new();
                let mut lone_says_not = false;
                let mut defaults: Vec<(String, crate::source::Span)> = Vec::new();
                for e in entries {
                    let ExprKind::Call { path, args, .. } = &e.kind else {
                        return err(Diagnostic::error("each target is a `cca::Target[name = …]`, with a `runner = …` when this machine cannot run it").at(e.span));
                    };
                    if path.limb.as_ref().map(|l| l.name.as_str()) != Some("cca") || !path.segs.iter().map(|s| s.name.as_str()).eq(["Target"]) {
                        return err(Diagnostic::error(format!("each target is a `cca::Target[…]`, and this is `{}`", path.spelled())).at(e.span));
                    }
                    let mut name = None;
                    let mut is_default = false;
                    let mut said_not = false;
                    for a in args {
                        let AArg::Named(field, v) = a else {
                            return err(Diagnostic::error("a `cca::Target` names its fields: `name = …`, and `runner = …` when it needs one").at(a.span()));
                        };
                        match field.name.as_str() {
                            "name" => name = Some((text(v, "a target's `name`")?, field.span)),
                            "runner" => {
                                text(v, "a target's `runner`")?;
                            }
                            // The one built when none is named (owner,
                            // 2026-09-29: b).
                            "default" => match &v.kind {
                                ExprKind::Bool(b) => {
                                    is_default = *b;
                                    said_not = !*b;
                                }
                                _ => return err(Diagnostic::error("a target's `default` is `true` or `false`").at(v.span)),
                            },
                            other => return err(Diagnostic::error(format!("a `cca::Target` has no field `{}`: it is a `name`, a `runner` when this machine cannot run it, and `default = true` on the one built when none is named", other)).at(field.span)),
                        }
                    }
                    let Some((n, span)) = name else {
                        return err(Diagnostic::error("a `cca::Target` needs its `name`: `cca::Target[name = \"aarch64-linux\"]`").at(e.span));
                    };
                    if names.contains(&n) {
                        return err(Diagnostic::error(format!("the target `{}` is listed twice in `targets`", n)).at(span));
                    }
                    if is_default {
                        defaults.push((n.clone(), span));
                    }
                    names.push(n);
                    name_spans.push(span);
                    lone_says_not = said_not;
                }
                // A single target is the default by itself; of several,
                // exactly one says so.
                if defaults.len() > 1 {
                    return err(Diagnostic::error(format!("`{}` and `{}` both say `default = true`: of several targets, exactly one says `default = true`", defaults[0].0, defaults[1].0)).at(defaults[1].1));
                }
                if names.len() > 1 && defaults.is_empty() {
                    return err(Diagnostic::error(format!("{} targets, and none is the one built when none is named: of several targets, exactly one says `default = true`", names.len())).at(v.span));
                }
                // Only a target CCA can build (owner, 2026-09-30: "refuse"),
                // as in a Synthia.ttsr; the default count is said first.
                let known = targets();
                for (n, span) in names.iter().zip(&name_spans) {
                    if !known.contains(&n.as_str()) {
                        return err(Diagnostic::error(format!("`{}` is not a target CCA can build; it builds for {}", n, known.iter().map(|k| format!("`{}`", k)).collect::<Vec<_>>().join(", "))).at(*span));
                    }
                }
                // A lone target is the default unless it says
                // `default = false` (owner, 2026-09-30): then there is none.
                out.default_target = match defaults.first() {
                    Some((n, s)) => Some((n.clone(), *s)),
                    None if names.len() == 1 && lone_says_not => None,
                    None => names.first().map(|n| (n.clone(), v.span)),
                };
                out.targets = names.clone();
            }
            // `powers = [cca::Power.c_calls,]` and `deny = [cca::Power.c_calls
            // .except["write"],]` (owner, 2026-10-07).
            "powers" | "deny" => {
                let ExprKind::List(entries) = &v.kind else {
                    return err(Diagnostic::error(format!("`{}` is a list of `cca::Power.…`: `[cca::Power.c_calls,]`", setting)).at(v.span));
                };
                for e in entries {
                    let (power, except, for_limbs) = read_power(e)?;
                    if setting == "powers" {
                        if except.is_some() || for_limbs.is_some() {
                            return err(Diagnostic::error("`.except` and `.for` belong to `deny`; `powers` lists what this Neuron uses").at(e.span));
                        }
                        out.powers.push(power);
                    } else {
                        out.deny.push(cca_base::powers::Deny { power, except: except.unwrap_or_default(), for_limbs: for_limbs.unwrap_or_default() });
                    }
                }
            }
            // The cores a Limb's code is built by (`design/cores.md`):
            // official ones only, each once. A Limb reaches only the words
            // of the cores it lists; CCA's code is built the same either way.
            "cores" => {
                let ExprKind::List(entries) = &v.kind else {
                    return err(Diagnostic::error(format!("`cores` is a list of `cca::Core[…]`: `[{},]`", crate::ast::core_line("IEG"))).at(v.span));
                };
                let mut names: Vec<String> = Vec::new();
                for e in entries {
                    let core = match &e.kind {
                        ExprKind::Call { path, args, .. } if path.limb.as_ref().map(|l| l.name.as_str()) == Some("cca") && path.segs.iter().map(|s| s.name.as_str()).eq(["Core"]) => args,
                        _ => return err(Diagnostic::error("each core is a `cca::Core[id = …, version = …]`").at(e.span)),
                    };
                    // A core is listed by its registry ID and a version, or
                    // by a path when it is not registered (owner,
                    // 2026-10-06: "ID", "`a`", "`a`").
                    let (mut id, mut version, mut where_) = (None, None, None);
                    for a in core {
                        let AArg::Named(f, cv) = a else {
                            return err(Diagnostic::error("a `cca::Core` names its fields: `cca::Core[id = \"…\", version = \"…\"]`").at(a.span()));
                        };
                        match f.name.as_str() {
                            "name" => return err(Diagnostic::error(format!("a core is listed by its registry ID since 2026-10-06 (names repeat, IDs do not): `cca::Core[id = \"…\", version = \"…\"]`; IEG's is `{}`", OFFICIAL_CORES[0].1)).at(f.span)),
                            "id" => id = Some((text(cv, "a core's `id`")?, cv.span)),
                            "version" => version = Some((text(cv, "a core's `version`")?, cv.span)),
                            "path" => where_ = Some((text(cv, "a core's `path`")?, cv.span)),
                            other => return err(Diagnostic::error(format!("a `cca::Core` has no field `{}`: it is an `id` and a `version`, or a `path`", other)).at(f.span)),
                        }
                    }
                    if let (Some(_), Some((_, at))) = (&id, &where_) {
                        return err(Diagnostic::error("a core says where it comes from once: an `id` and a `version` (the registry) or a `path` (a folder on this machine)").at(*at));
                    }
                    // Until cores can be loaded, only the official ones
                    // (owner, 2026-10-06: "`a`").
                    if let Some((p, at)) = where_ {
                        return err(Diagnostic::error(format!("the core at `{}` is not registered, so not verified; loading another core is not built yet (a plugin, which CCA cannot load yet)", p)).at(at));
                    }
                    let Some((i, ispan)) = id else {
                        return err(Diagnostic::error("a `cca::Core` says which core: `cca::Core[id = \"…\", version = \"…\"]`").at(e.span));
                    };
                    // MTM's ID is retired (owner, 2026-10-07: "MTM is gone.").
                    if i == crate::ast::MTM_ID {
                        return err(Diagnostic::error("the core `9c6c7730…` is MTM, which is gone since 2026-10-07: every spelling it held is an old one now — take it out of `cores`").at(ispan));
                    }
                    let Some(&(n, _, known)) = OFFICIAL_CORES.iter().find(|(_, oid, _)| *oid == i) else {
                        return err(Diagnostic::error(format!("the core `{}…` is not an official one; loading another core is not built yet (a plugin, which CCA cannot load yet)", &i[..i.len().min(8)])).at(ispan));
                    };
                    let Some((v, vspan)) = version else {
                        return err(Diagnostic::error("a core needs its `version`, as a registry limb does").at(e.span));
                    };
                    if v != known {
                        return err(Diagnostic::error(format!("CCA knows {} `{}` only, the installed one; this asks for `{}`", n, known, v)).at(vspan));
                    }
                    if names.iter().any(|x| x == n) {
                        return err(Diagnostic::error(format!("the core {} is listed twice", n)).at(ispan));
                    }
                    names.push(n.to_string());
                }
                // MCPS works on IEG (owner: "Needs IEG, not std").
                if names.iter().any(|n| n == "MCPS") && !names.iter().any(|n| n == "IEG") {
                    return err(Diagnostic::error(format!("the core MCPS needs IEG: add `{}` to `cores`", crate::ast::core_line("IEG"))).at(v.span));
                }
                out.cores = names;
            }
            _ => {
                let ExprKind::List(entries) = &v.kind else {
                    return err(Diagnostic::error("`dependencies` is a list of `cca::Dep[…]`: `[cca::Dep[path = \"../areas\"],]`").at(v.span));
                };
                for e in entries {
                    let ExprKind::Call { path, args, .. } = &e.kind else {
                        return err(Diagnostic::error("each dependency is a `cca::Dep[path = …]`, or a `cca::Dep[id = …, version = …]`").at(e.span));
                    };
                    if path.limb.as_ref().map(|l| l.name.as_str()) != Some("cca") || !path.segs.iter().map(|s| s.name.as_str()).eq(["Dep"]) {
                        return err(Diagnostic::error(format!("each dependency is a `cca::Dep[…]`, and this is `{}`", path.spelled())).at(e.span));
                    }
                    let (mut dpath, mut version, mut id) = (None, None, None);
                    for a in args {
                        let AArg::Named(field, v) = a else {
                            return err(Diagnostic::error("a `cca::Dep` names its fields: a `path = …`, or an `id = …` and a `version = …`").at(a.span()));
                        };
                        let slot = match field.name.as_str() {
                            "name" | "prefix" => return err(Diagnostic::error("a dependency names no word: code writes the Neuron's own `prefix`, from its `CCA.cplr`").at(field.span)),
                            "path" => &mut dpath,
                            "version" => &mut version,
                            "id" => &mut id,
                            other => return err(Diagnostic::error(format!("a `cca::Dep` has no field `{}`: it is a `path`, or an `id` and a `version`", other)).at(field.span)),
                        };
                        *slot = Some((text(v, &format!("`{}`", field.name))?, field.span));
                    }
                    // The registry knows a Limb by the ID its creator made —
                    // 256 random bits, 64 hex digits — never by its name
                    // (owner, 2026-09-29: "Names could be the same, IDs
                    // can't").
                    if let Some((i, ispan)) = &id {
                        if i.len() != 64 || !i.chars().all(|c| c.is_ascii_hexdigit()) {
                            return err(Diagnostic::error(format!("a dependency's `id` is 64 hex digits, the 256 random bits its creator made: `{}` is not", i)).at(*ispan));
                        }
                    }
                    // std, listed like any Limb (owner, 2026-10-06): CCA
                    // knows one version of it, and downloads nothing.
                    if let (Some((i, ispan)), Some((v, vspan))) = (&id, &version) {
                        if i == crate::parser::STD_ID {
                            if v != crate::parser::STD_VERSION {
                                return err(Diagnostic::error(format!("CCA knows cord `{}` only, and this asks for `{}`: write `version = \"{}\"`", crate::parser::STD_VERSION, v, crate::parser::STD_VERSION)).at(*vspan));
                            }
                            if out.has_std {
                                return err(Diagnostic::error("cord is listed twice in `dependencies`").at(*ispan));
                            }
                            out.has_std = true;
                            continue;
                        }
                    }
                    match (dpath, version) {
                        (Some(_), Some((_, vspan))) => {
                            return err(Diagnostic::error("a dependency says where it comes from once: a `path` (a folder on this machine) or a `version` (the registry), not both").at(vspan));
                        }
                        (None, Some(_)) if id.is_none() => {
                            return err(Diagnostic::error("this dependency comes from the registry, which knows a Neuron by its ID, never its name: add `id = \"…\"`, the 64 hex digits its creator made").at(e.span));
                        }
                        (None, Some((_, vspan))) => {
                            return err(Diagnostic::error("not yet: this dependency asks the registry for a `version`, and the registry does not exist yet — give its folder with `path = \"…\"`").at(vspan));
                        }
                        (None, None) => {
                            return err(Diagnostic::error("a dependency says where it comes from: a `path = \"…\"` (a folder on this machine), or an `id` and a `version` (the registry)").at(e.span));
                        }
                        (Some((p, pspan)), None) => {
                            if out.deps.iter().any(|d| d.path == p) {
                                return err(Diagnostic::error(format!("`{}` is listed twice in `dependencies`", p)).at(pspan));
                            }
                            out.deps.push(Dep { path: p });
                        }
                    }
                }
            }
        }
    }
    // What every manifest says (owner, 2026-09-29, as a Synthia.ttsr
    // does): a folder with no `CCA.cplr` is a plain program, and one with
    // it is a Limb with a name, a version and a target to build.
    for need in ["name", "prefix", "version"] {
        if !seen.iter().any(|s| s == need) {
            let what = if need == "prefix" { ", the word code writes in front of `::` to reach this Neuron" } else { "" };
            return err(Diagnostic::error(format!("the manifest has no `{}`{}: a build needs the Neuron's `name`, `prefix`, `version` and at least one target, {}", need, what, MANIFEST_SHAPE)).at(value.span));
        }
    }
    // std's maths always has true values and checks (owner, 2026-10-06):
    // a Limb that lists std lists ASP.
    if out.has_std && !out.cores.iter().any(|c| c == "ASP") {
        return err(Diagnostic::error(format!("cord needs the core ASP, so its maths has true values and checks: add `{}` to `cores`", crate::ast::core_line("ASP"))).at(value.span));
    }
    // And IEG, which grants it the operators on built-in types (owner,
    // 2026-10-06: "`a`").
    if out.has_std && !out.cores.iter().any(|c| c == "IEG") {
        return err(Diagnostic::error(format!("cord needs the core IEG, which grants it the operators on built-in types: add `{}` to `cores`", crate::ast::core_line("IEG"))).at(value.span));
    }
    if out.targets.is_empty() {
        return err(Diagnostic::error(format!("the manifest has no target: a build needs the Neuron's `name`, `prefix`, `version` and at least one target, {}", MANIFEST_SHAPE)).at(value.span));
    }
    Ok(out)
}

thread_local! {
    /// The kind lines of the Limb whose files are being read: they cover
    /// every file of it.
    static BARE_KINDS: std::cell::RefCell<Vec<(crate::ast::Bare, Vec<String>, crate::source::Span)>> = const { std::cell::RefCell::new(Vec::new()) };
}

thread_local! {
    /// The `case` kind lines of the Limb whose files are being read.
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
        return err(Diagnostic::error("a `case` mark on kinds (`case.insensitive [fn, struct];`) covers the whole Neuron, so it is written in `limb.cpls`").at(*at));
    }
    let lines = CASE_LINES.with(|k| k.borrow().clone());
    let line = |at: Span| sources.get(at.file).line_of(at.start);
    let spelled = |b: bool| if b { "case.insensitive" } else { "case.sensitive" };
    let decide = |marks: &[(bool, Span)], what: &str, at: Span| -> Result<bool> {
        if let (Some(i), Some(s)) = (marks.iter().find(|m| m.0), marks.iter().find(|m| !m.0)) {
            return err(Diagnostic::error(format!("{} is marked both `{}` ({}) and `{}` ({}): marks that disagree are an error (owner, 2026-10-06)", what, spelled(true), line(i.1), spelled(false), line(s.1))).at(at));
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
                return err(Diagnostic::error(format!("{} and {} ({}) differ only in case, and {} is `case.insensitive`: they would be one name", shown(name), shown(other), line(*other_at), shown(which))).at(at));
            }
        }
        seen.push((name.to_string(), at, ins));
        Ok(())
    };
    let plain = |n: &str| format!("`{}`", n);
    let quoted = |n: &str| format!("`'{}'`", n);
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
                let ins = decide(&marks(Some("const"), true), &format!("`const '{}'`", d.name.name), d.name.span)?;
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

/// Every `pub` thing says how other Limbs write its path, and the marks
/// that cover it agree (owner, 2026-10-06: no default; "`c`"). A mark
/// covers from the item itself, the block modules around it, its file's
/// first line, and its kind in `limb.cpls`. `base` is the file's module
/// path, empty at the root; `root` is whether this is the root file, the
/// only one whose kind lines count.
fn check_bare(sources: &SourceMap, items: &mut Items, file: &crate::ast::File, base: &str, root: bool) -> Result<()> {
    use crate::ast::{Bare, Item, ItemKind};
    if root {
        BARE_KINDS.with(|k| *k.borrow_mut() = file.bare_kinds.clone());
    } else if let Some((m, _, at)) = file.bare_kinds.first() {
        return err(Diagnostic::error(format!("`{} [ … ];` covers the whole Neuron, so it is written in `limb.cpls`", m.spelled())).at(*at));
    }
    let kinds = BARE_KINDS.with(|k| k.borrow().clone());
    let line = |at: crate::source::Span| sources.get(at.file).line_of(at.start);
    fn walk(items: &[Item], path: &str, chain: &mut Vec<(Bare, crate::source::Span)>, each: &mut dyn FnMut(&Item, String, &[(Bare, crate::source::Span)]) -> Result<()>) -> Result<()> {
        for it in items {
            let named = |n: &str| if path.is_empty() { n.to_string() } else { format!("{}.{}", path, n) };
            if let ItemKind::Module { name, items } = &it.kind {
                let pushed = it.bare.is_some();
                if let Some(b) = it.bare {
                    chain.insert(0, b);
                }
                walk(items, &named(&name.name), chain, each)?;
                if pushed {
                    chain.remove(0);
                }
                continue;
            }
            let name = match &it.kind {
                ItemKind::Fn(d) | ItemKind::ExternFn(d) => named(&d.name.name),
                ItemKind::Struct(d) => named(&d.name.name),
                ItemKind::Enum(d) => named(&d.name.name),
                ItemKind::Interface(d) => named(&d.name.name),
                ItemKind::Const(d) => named(&d.name.name),
                ItemKind::Op(d) => named(&d.op.name),
                _ => continue,
            };
            each(it, name, chain)?;
        }
        Ok(())
    }
    let mut chain = Vec::new();
    walk(&file.items, base, &mut chain, &mut |it, name, modules| {
        if it.vis != Vis::Pub {
            return Ok(());
        }
        let name_span = match &it.kind {
            ItemKind::Fn(d) | ItemKind::ExternFn(d) => d.name.span,
            ItemKind::Struct(d) => d.name.span,
            ItemKind::Enum(d) => d.name.span,
            ItemKind::Interface(d) => d.name.span,
            ItemKind::Const(d) => d.name.span,
            ItemKind::Op(d) => d.op.span,
            _ => it.span,
        };
        let kind = match &it.kind {
            ItemKind::Fn(_) | ItemKind::ExternFn(_) => "fn",
            ItemKind::Struct(_) => "struct",
            ItemKind::Enum(_) => "enum",
            ItemKind::Interface(_) => "interface",
            // A const has no mark (owner, 2026-10-06: "`b`").
            ItemKind::Const(_) => return Ok(()),
            // A glyph operator has no path, so nothing to mark.
            ItemKind::Op(d) if !d.op.name.starts_with(|c: char| c.is_ascii_alphabetic()) => return Ok(()),
            _ => "op",
        };
        let both = |a: (Bare, crate::source::Span), b: (Bare, crate::source::Span)| {
            err(Diagnostic::error(format!("`{} {}` is marked both `{}` ({}) and `{}` ({}): marks that disagree are an error", kind, name, a.0.spelled(), line(a.1), b.0.spelled(), line(b.1))).at(it.span))
        };
        // Its own mark, then the block modules around it, inner first.
        let chain: Vec<(Bare, crate::source::Span)> = it.bare.iter().copied().chain(modules.iter().copied()).collect();
        if let Some(&first) = chain.first() {
            if let Some(&other) = chain.iter().find(|m| m.0 != first.0) {
                return both(first, other);
            }
        }
        // That, the file's and the kind's, in the order they are written.
        let mut all: Vec<(Bare, crate::source::Span)> = chain.first().copied().into_iter().chain(file.bare_file).chain(kinds.iter().filter(|(_, ks, _)| ks.iter().any(|k| k == kind)).map(|(m, _, at)| (*m, *at))).collect();
        all.sort_by_key(|m| (m.1.file != it.span.file, line(m.1)));
        match all.first() {
            None => err(Diagnostic::error(format!("`{k} {n}` is `pub`, and nothing says how other Neurons write its path: mark it (`pub bare.never {k} …`), its module, or its kind in `limb.cpls` (`bare.never [{k}];`) as `bare.never` (path required), `bare.optional` or `bare.only` (path forbidden); there is no default", k = kind, n = name)).at(it.span)),
            Some(&first) => match all.iter().find(|m| m.0 != first.0) {
                Some(&other) => both(first, other),
                None => {
                    items.bare_marks.insert(name_span, first.0);
                    Ok(())
                }
            },
        }
    })
}

/// One `cca::Power.…` of a manifest: the path, the things in `[ ]`, and a
/// `deny`'s `.except[…]` and `.for[…]`.
type ReadPower = (cca_base::powers::Power, Option<Vec<String>>, Option<Vec<String>>);

fn read_power(e: &crate::ast::Expr) -> Result<ReadPower> {
    use crate::ast::{Arg as AArg, ExprKind};
    enum Step {
        Word(String),
        Things(Vec<String>),
    }
    fn texts(args: &[AArg]) -> Result<Vec<String>> {
        args.iter()
            .map(|a| match a {
                AArg::Positional(crate::ast::Expr { kind: ExprKind::Str(s), .. }) => Ok(s.clone()),
                other => err(Diagnostic::error("a power names exact things as texts: `cca::Power.c_calls[\"sqrt\"]`").at(other.span())),
            })
            .collect()
    }
    fn flat(e: &crate::ast::Expr, out: &mut Vec<Step>) -> Result<()> {
        match &e.kind {
            ExprKind::Path(p) | ExprKind::Call { path: p, args: _, .. } if p.limb.as_ref().map(|l| l.name.as_str()) == Some("cca") && p.segs.first().is_some_and(|s| s.name == "Power") => {
                out.extend(p.segs[1..].iter().map(|s| Step::Word(s.name.clone())));
                if let ExprKind::Call { args, .. } = &e.kind {
                    out.push(Step::Things(texts(args)?));
                }
                Ok(())
            }
            ExprKind::Field { base, field } => {
                flat(base, out)?;
                out.push(Step::Word(field.name.clone()));
                Ok(())
            }
            ExprKind::CallValue { callee, args, .. } => {
                flat(callee, out)?;
                out.push(Step::Things(texts(args)?));
                Ok(())
            }
            ExprKind::Index { base, index } => {
                flat(base, out)?;
                out.push(Step::Things(texts(&[AArg::Positional((**index).clone())])?));
                Ok(())
            }
            _ => err(Diagnostic::error("a power is `cca::Power.` and its name: `cca::Power.c_calls`, `cca::Power.raw_memory.read`").at(e.span)),
        }
    }
    let mut steps = Vec::new();
    flat(e, &mut steps)?;
    let mut power = cca_base::powers::Power { path: Vec::new(), things: Vec::new() };
    let (mut except, mut for_limbs) = (None, None);
    let mut i = 0;
    while i < steps.len() {
        match &steps[i] {
            Step::Word(w) if w == "except" || w == "for" => {
                let Some(Step::Things(t)) = steps.get(i + 1) else {
                    return err(Diagnostic::error(format!("`.{}` names things in `[ ]`: `.{}[\"…\"]`", w, w)).at(e.span));
                };
                if w == "except" { except = Some(t.clone()) } else { for_limbs = Some(t.clone()) }
                i += 2;
            }
            Step::Word(w) if except.is_none() && for_limbs.is_none() && power.things.is_empty() => {
                power.path.push(w.clone());
                i += 1;
            }
            Step::Things(t) if power.things.is_empty() && except.is_none() && for_limbs.is_none() => {
                power.things = t.clone();
                i += 1;
            }
            _ => return err(Diagnostic::error("a power's things come last, after its name, and only `.except[…]` or `.for[…]` follow them").at(e.span)),
        }
    }
    if power.path.is_empty() {
        return err(Diagnostic::error("a power has a name: `cca::Power.c_calls`").at(e.span));
    }
    // The powers there are (owner, 2026-10-07): Signatar's three and their
    // narrowings, IEG's vouching, and the extension powers built so far.
    let known: &[&[&str]] = &[&["inline_assembly"], &["c_calls"], &["c_calls", "functions"], &["c_calls", "variables"], &["raw_memory"], &["raw_memory", "read"], &["raw_memory", "write"], &["raw_memory", "cast"], &["vouch"], &["vouch", "c"], &["operators_on_builtin_types"], &["prefix_operators"]];
    if !known.iter().any(|k| k.iter().copied().eq(power.path.iter().map(String::as_str))) {
        return err(Diagnostic::error(format!("there is no power `{}`: the powers are {}", power.path.join("."), known.iter().map(|k| format!("`{}`", k.join("."))).collect::<Vec<_>>().join(", "))).at(e.span));
    }
    Ok((power, except, for_limbs))
}

fn new_module(items: &mut Items, name: &str, parent: Option<ModuleId>, vis: Vis) -> ModuleId {
    let path = crate::resolve::module_path(items, name, parent);
    let id = ModuleId(items.modules.len() as u32);
    items.modules.push(crate::resolve::Module { name: name.to_string(), path, parent, vis, items: Default::default(), consts: Default::default(), folded: Default::default(), folded_consts: Default::default() });
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

/// A Limb's 'name' and 'prefix' follow the module-name rule (owner,
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
    err(Diagnostic::error(format!("`{}` cannot {}: {}", w, what, why)).at(span))
}

/// The file's own name, for a message: a full path is noise and differs per machine.
fn short(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| p.display().to_string())
}
