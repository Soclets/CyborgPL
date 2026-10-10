//! `incl.C.system["stdio.h"];` / `incl.C["vendor/mylib.h"];` (owner, 2026-10-10): a C header read while
//! building, so C's functions need no hand-written `extern.C`. The header's functions, globals and plain
//! `#define` constants come through; a function-like macro is skipped (owner: "(a)"). The oracle asks
//! Clang to read the header and turns each declaration it can say in Nerve into the line a person would
//! write: `extern.C fn strlen ['p1': ptr[uint8]] -> uint64;`, `const 'BUFSIZ' := 1024;`. Everything else
//! (structs, unions, function pointers, `long double`, …) is left out for now, a provisional pick in the
//! oracle's README. The parser reads the lines as if written where the `incl.C` line is.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::io::Write;

thread_local! {
    /// The oracle reads a program more than once; a header is asked of Clang once per run.
    static CACHE: std::cell::RefCell<HashMap<(bool, String, PathBuf), Result<Header, String>>> = std::cell::RefCell::new(HashMap::new());
}

/// What a header stands for: the Nerve lines a person would write, and the C structs Nerve only points
/// at (owner, 2026-10-10: `ptr.mut[FILE]`), which have no spelling of their own.
#[derive(Clone)]
pub struct Header { pub lines: Vec<String>, pub opaque: Vec<String> }

/// What a header stands for, or why it cannot be read.
pub fn lines(system: bool, path: &str, neuron_dir: &Path) -> Result<Header, String> {
    let key = (system, path.to_string(), neuron_dir.to_path_buf());
    if let Some(r) = CACHE.with(|c| c.borrow().get(&key).cloned()) { return r; }
    let r = read(system, path, neuron_dir);
    // `ORACLE_SHOW_C=1` prints the lines a header stands for, to see what came through
    if std::env::var_os("ORACLE_SHOW_C").is_some() { if let Ok(h) = &r { for o in &h.opaque { eprintln!("{path}: (opaque) {o}"); } for l in &h.lines { eprintln!("{path}: {l}"); } } }
    CACHE.with(|c| c.borrow_mut().insert(key, r.clone()));
    r
}

fn clang(args: &[&str], input: &str, dir: &Path) -> Result<String, String> {
    let mut child = Command::new("clang").args(["-x", "c"]).args(args).arg(format!("-I{}", dir.display())).arg("-")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()
        .map_err(|e| format!("the C compiler (clang) could not be started: {e}"))?;
    child.stdin.take().unwrap().write_all(input.as_bytes()).map_err(|e| e.to_string())?;
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let first = err.lines().find(|l| l.contains("error")).unwrap_or("it failed").trim();
        let first = first.split_once("error: ").map_or(first, |(_, m)| m);
        return Err(format!("the C compiler says: {first}"));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn read(system: bool, path: &str, dir: &Path) -> Result<Header, String> {
    let include = if system { format!("#include <{path}>\n") } else {
        // a header in the Neuron is found from its root, as every `incl` path is
        let full = dir.join(path);
        if !full.is_file() { return Err(format!("cannot read \"{path}\" ({}, from the Neuron's root)", full.display())); }
        format!("#include \"{}\"\n", full.display())
    };
    let ast = clang(&["-fsyntax-only", "-Xclang", "-ast-dump"], &include, dir)?;
    // C's type names (`size_t`), each standing for the type it names, read first: a function may use one
    let mut td: HashMap<String, String> = HashMap::new();
    for l in ast.lines() {
        let Some(rest) = l.strip_prefix("|-").or_else(|| l.strip_prefix("`-")) else { continue };
        if !rest.starts_with("TypedefDecl ") { continue; }
        let Some(q) = rest.find('\'') else { continue };
        let name = rest[..q].split_whitespace().last().unwrap_or("").to_string();
        let after = &rest[q + 1..];
        let Some(e) = after.find('\'') else { continue };
        let mut ty = &after[..e];
        if let Some(d) = after[e + 1..].strip_prefix(":'") { if let Some(e2) = d.find('\'') { ty = &d[..e2]; } }
        td.insert(name, ty.to_string());
    }
    // C's structs: each tag's fields (None when some part is beyond a plain field), and its Nerve name,
    // C's type name when it has one, else the tag (owner, 2026-10-10: `FILE`, `tm`)
    let mut records: HashMap<String, Option<Vec<(String, String)>>> = HashMap::new();
    let mut current: Option<(String, Option<Vec<(String, String)>>)> = None;
    for l in ast.lines() {
        if let Some(rest) = l.strip_prefix("|-").or_else(|| l.strip_prefix("`-")) {
            if let Some((tag, fields)) = current.take() { records.insert(tag, fields); }
            if rest.starts_with("RecordDecl ") && rest.ends_with(" definition") {
                let words: Vec<&str> = rest.split_whitespace().collect();
                let n = words.len();
                if n >= 3 && words[n - 3] == "struct" { current = Some((words[n - 2].to_string(), Some(Vec::new()))); }
                else if n >= 3 && words[n - 3] == "union" { records.insert(words[n - 2].to_string(), None); }
            }
            continue;
        }
        let Some((_, fields)) = current.as_mut() else { continue };
        let Some(child) = l.strip_prefix("| |-").or_else(|| l.strip_prefix("| `-")) else { continue };
        if !child.starts_with("FieldDecl ") { *fields = None; continue; }   // a nested struct or union, an attribute: beyond a plain field
        let Some(q) = child.find('\'') else { *fields = None; continue };
        let fname = child[..q].split_whitespace().last().unwrap_or("").to_string();
        let after = &child[q + 1..];
        let Some(e) = after.find('\'') else { *fields = None; continue };
        let mut fty = &after[..e];
        if let Some(d) = after[e + 1..].strip_prefix(":'") { if let Some(e2) = d.find('\'') { fty = &d[..e2]; } }
        if let Some(f) = fields.as_mut() { f.push((fname, fty.to_string())); }
    }
    if let Some((tag, fields)) = current.take() { records.insert(tag, fields); }
    let mut names: HashMap<String, String> = HashMap::new();
    for (alias, target) in &td {
        if let Some(tag) = target.strip_prefix("struct ") { if good_name(alias) && !alias.starts_with('_') { names.entry(tag.to_string()).or_insert_with(|| alias.clone()); } }
    }
    let mut cx = Cx { td: &td, records: &records, names: &names, layout: HashMap::new(), opaque: Vec::new(), busy: Vec::new() };
    let mut fns: Vec<(String, String)> = Vec::new();   // (name, line), the last declaration of a name winning
    let mut vars: Vec<(String, String)> = Vec::new();
    for l in ast.lines() {
        let Some(rest) = l.strip_prefix("|-").or_else(|| l.strip_prefix("`-")) else { continue };
        let is_fn = rest.starts_with("FunctionDecl ");
        if !is_fn && !rest.starts_with("VarDecl ") { continue; }
        let Some(q) = rest.find('\'') else { continue };
        let name = rest[..q].split_whitespace().last().unwrap_or("").to_string();
        let after = &rest[q + 1..];
        let Some(e) = after.find('\'') else { continue };
        let mut ty = &after[..e];
        let mut tail = &after[e + 1..];
        if let Some(d) = tail.strip_prefix(":'") { if let Some(e2) = d.find('\'') { ty = &d[..e2]; tail = &d[e2 + 1..]; } }
        if tail.split_whitespace().any(|w| w == "static") { continue; }   // a header's own inline helpers: nothing to link
        if !good_name(&name) { continue; }
        if is_fn {
            if let Some(line) = function(&name, ty, &mut cx) { fns.retain(|(n, _)| *n != name); fns.push((name, line)); }
        } else if tail.split_whitespace().any(|w| w == "extern") {
            let read_only = ty.split_whitespace().next() == Some("const") && !ty.trim_end().ends_with('*');
            if let Some(t) = map(strip(ty), &mut cx, 0) {
                vars.retain(|(n, _)| *n != name);
                vars.push((name.clone(), format!("extern.C {}'{name}': {t};", if read_only { "" } else { "mut " })));
            }
        }
    }
    // plain `#define` constants: the header's, less what the compiler defines by itself
    let given = clang(&["-E", "-dM"], "", dir)?;
    let given: std::collections::HashSet<&str> = given.lines().filter_map(|l| l.strip_prefix("#define ")?.split_whitespace().next()).collect();
    let defs = clang(&["-E", "-dM"], &include, dir)?;
    let mut consts = Vec::new();
    for l in defs.lines() {
        let Some(d) = l.strip_prefix("#define ") else { continue };
        let (name, value) = d.split_once(' ').unwrap_or((d, ""));
        if name.contains('(') || given.contains(name) || !good_name(name) { continue; }   // a function-like macro, or the compiler's own
        if let Some(v) = constant(value.trim()) { consts.push(format!("const '{name}' := {v};")); }
    }
    // every struct of the header that has a name, whole when Nerve can spell its fields, else opaque
    let mut tags: Vec<&String> = records.keys().collect();
    tags.sort();
    for tag in tags { let _ = map(&format!("struct {tag}"), &mut cx, 0); }
    let mut structs: Vec<(String, String)> = cx.layout.clone().into_iter().collect();
    structs.sort();
    // a struct that points at itself was opaque only while its own fields were being worked out
    let whole: Vec<String> = cx.layout.keys().filter_map(|tag| struct_name(tag, &cx)).collect();
    let mut opaque = cx.opaque.clone();
    opaque.retain(|o| !whole.contains(o));
    opaque.sort(); opaque.dedup();
    Ok(Header {
        lines: structs.into_iter().map(|(_, l)| l).chain(fns.into_iter().map(|(_, l)| l)).chain(vars.into_iter().map(|(_, l)| l)).chain(consts).collect(),
        opaque,
    })
}

/// What mapping a type needs: C's type names, its structs, and the structs met so far.
struct Cx<'a> {
    td: &'a HashMap<String, String>,
    records: &'a HashMap<String, Option<Vec<(String, String)>>>,
    names: &'a HashMap<String, String>,
    /// structs that come through whole: tag → its `layout.C struct` line
    layout: HashMap<String, String>,
    /// Nerve names of structs that come through opaque
    opaque: Vec<String>,
    /// tags being worked out, so a struct that points at itself is not followed forever
    busy: Vec<String>,
}

/// A C name Nerve can hold: no leading `__` (the C library's own insides). A Nerve keyword is held
/// too (owner, 2026-10-10: "(b)"): a field is read after its dot, a function reached by path.
fn good_name(n: &str) -> bool {
    !n.is_empty() && !n.starts_with("__") && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !n.starts_with(|c: char| c.is_ascii_digit())
}

/// A struct's Nerve name: C's type name for it, else its tag; None for the C library's own (`__x`).
fn struct_name(tag: &str, cx: &Cx) -> Option<String> {
    let n = cx.names.get(tag).cloned().unwrap_or_else(|| tag.to_string());
    (good_name(&n) && !crate::parser::KEYWORDS.contains(&n.as_str())).then_some(n)
}

/// `struct tag` by value: its name when it comes through whole (its fields all spelled), else None.
/// Through a pointer: its name either way, opaque when not whole.
fn record(tag: &str, cx: &mut Cx, depth: usize, by_value: bool) -> Option<String> {
    let name = struct_name(tag, cx)?;
    if cx.layout.contains_key(tag) { return Some(name); }
    let whole = if cx.busy.iter().any(|b| b == tag) { false } else {
        cx.busy.push(tag.to_string());
        let fields = cx.records.get(tag).cloned().flatten();
        let spelled: Option<Vec<String>> = fields.filter(|f| !f.is_empty()).and_then(|f| f.iter().map(|(n, t)| {
            if !good_name(n) { return None; }
            map(t, cx, depth + 1).map(|t| format!("{n}: {t}"))
        }).collect());
        cx.busy.pop();
        match spelled {
            Some(fs) => { cx.layout.insert(tag.to_string(), format!("layout.C struct {name} {{ {} }}", fs.join("  "))); true }
            None => false,
        }
    };
    if whole { return Some(name); }
    if !cx.opaque.contains(&name) { cx.opaque.push(name.clone()); }
    if by_value { None } else { Some(name) }
}

/// `int (const char *, ...)` as an `extern.C fn` line, or None when some part has no Nerve spelling yet.
fn function(name: &str, ty: &str, cx: &mut Cx) -> Option<String> {
    let open = ty.find('(')?;
    if !ty.ends_with(')') || ty[open + 1..ty.len() - 1].contains('(') || ty[..open].contains(')') { return None; }   // function pointers
    let ret = strip(&ty[..open]);
    let ret = if ret == "void" { String::new() } else { format!(" -> {}", map(ret, cx, 0)?) };
    let inside = ty[open + 1..ty.len() - 1].trim();
    let mut params = Vec::new();
    if !inside.is_empty() && inside != "void" {
        for (i, p) in inside.split(',').map(str::trim).enumerate() {
            if p == "..." { params.push("...'args'".to_string()); continue; }
            params.push(format!("'p{}': {}", i + 1, map(strip(p), cx, 0)?));
        }
    }
    Some(format!("extern.C fn {name} [{}]{ret};", params.join(", ")))
}

/// A C type without the words that change nothing Nerve can say (`restrict`, the nullability marks).
fn strip(t: &str) -> &str { t.trim() }

/// A C type as Nerve writes it at the C boundary; `const` before a pointer's target makes it `ptr`, else
/// `ptr.mut`. Plain `char` is a byte, `uint8`, as Nerve's text is bytes; `void *` points at bytes.
fn map(t: &str, cx: &mut Cx, depth: usize) -> Option<String> {
    if depth > 16 { return None; }
    let mut t = t.to_string();
    for mark in ["__restrict", "restrict", "_Nullable", "_Nonnull", "_Null_unspecified", "volatile", "__unsafe_indexable", "__single"] { t = t.replace(mark, " "); }
    let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
    let t = t.replace(" *", "*").replace("* ", "*");
    if let Some(inner) = t.strip_suffix('*') {
        let inner = inner.trim();
        let (is_const, base) = match inner.strip_prefix("const ") { Some(b) => (true, b.trim()), None => match inner.strip_suffix(" const") { Some(b) => (true, b.trim()), None => (false, inner) } };
        let target = if base == "void" || cx.td.get(base).is_some_and(|t| t == "void") { "uint8".to_string() }
            else if let Some(tag) = struct_tag(base, cx) { record(&tag, cx, depth, false)? }
            else { map(base, cx, depth + 1)? };
        return Some(format!("ptr{}[{target}]", if is_const { "" } else { ".mut" }));
    }
    let base = t.strip_prefix("const ").unwrap_or(&t).trim();
    if let Some(tag) = struct_tag(base, cx) { return record(&tag, cx, depth, true); }
    Some(match base {
        "char" | "unsigned char" => "uint8", "signed char" => "int8",
        "short" | "short int" | "signed short" => "int16", "unsigned short" | "unsigned short int" => "uint16",
        "int" | "signed int" | "signed" => "int32", "unsigned int" | "unsigned" => "uint32",
        "long" | "long int" | "signed long" | "long long" | "long long int" | "signed long long" => "int64",
        "unsigned long" | "unsigned long int" | "unsigned long long" | "unsigned long long int" => "uint64",
        "float" => "bin32", "double" => "bin64", "_Bool" | "bool" => "bool",
        // a C type name stands for the type it names (`size_t` is `unsigned long` here)
        name => { let t = cx.td.get(name)?.clone(); return map(&t, cx, depth + 1); }
    }.to_string())
}

/// A `#define`'s value as a Nerve literal, when it is a plain number (in decimal: C's `0644` is octal)
/// or plain text; anything that names something else or computes is left out.
fn constant(v: &str) -> Option<String> {
    let mut v = v.trim();
    while let Some(inner) = v.strip_prefix('(').and_then(|s| s.strip_suffix(')')) { v = inner.trim(); }
    let (neg, digits) = match v.strip_prefix('-') { Some(d) => (true, d.trim()), None => (false, v) };
    if digits.starts_with(|c: char| c.is_ascii_digit()) {
        let d = digits.trim_end_matches(['u', 'U', 'l', 'L']);
        let n = if let Some(h) = d.strip_prefix("0x").or_else(|| d.strip_prefix("0X")) { u128::from_str_radix(h, 16).ok() }
            else if d.len() > 1 && d.starts_with('0') && d.chars().all(|c| c.is_ascii_digit()) { u128::from_str_radix(&d[1..], 8).ok() }
            else if d.chars().all(|c| c.is_ascii_digit()) { d.parse().ok() } else { None };
        return n.map(|n| if neg { format!("-{n}") } else { n.to_string() });
    }
    if !neg && v.len() >= 2 && v.starts_with('"') && v.ends_with('"') && !v[1..v.len() - 1].contains(['"', '\\']) { return Some(v.to_string()); }
    None
}

/// The tag when a type is a C struct, written `struct tm` or through C's name for it (`FILE`).
fn struct_tag(t: &str, cx: &Cx) -> Option<String> {
    if let Some(tag) = t.strip_prefix("struct ") { return Some(tag.trim().to_string()); }
    let target = cx.td.get(t)?;
    target.strip_prefix("struct ").map(|tag| tag.trim().to_string())
}
