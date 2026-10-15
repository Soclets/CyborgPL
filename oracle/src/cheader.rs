//! `incl.C.system["stdio.h"];` / `incl.C["vendor/mylib.h"];` (owner, 2026-10-10): a C header read while
//! building, so C's functions need no hand-written `extern.C`. The header's functions, globals and plain
//! `#define` constants come through; a function-like macro is skipped (owner: "(a)"). The oracle asks
//! Clang to read the header and turns each declaration it can say in Zag into the line a person would
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

/// What a header stands for: the Zag lines a person would write, and the C structs Zag only points
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
        // a header in the strike is found from its root, as every `incl` path is
        let full = dir.join(path);
        if !full.is_file() { return Err(format!("cannot read \"{path}\" ({}, from the strike's root)", full.display())); }
        format!("#include \"{}\"\n", full.display())
    };
    // `-fno-builtin`: Clang would otherwise declare `fread`, `calloc`, … itself and merge its own words
    // (`unsigned long`) over the header's (`size_t`) (compiler claude)
    let ast = clang(&["-fno-builtin", "-fsyntax-only", "-Xclang", "-ast-dump"], &include, dir)?;
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
        // `struct tm` / `enum color` stay as written; anything else is followed to what it names
        if !ty.starts_with("struct ") && !ty.starts_with("enum ") {
            if let Some(d) = after[e + 1..].strip_prefix(":'") { if let Some(e2) = d.find('\'') { ty = &d[..e2]; } }
        }
        td.insert(name, ty.to_string());
    }
    // C's enums (owner, 2026-10-10): each one's constants with their values (a value not written is the
    // one before plus 1); a named one becomes a `layout.C enum`, an unnamed one's names plain consts. An
    // unnamed enum given a name by the `typedef` right after it (`typedef enum { … } level_t;`) is named.
    let mut enums: Vec<(Option<String>, Vec<(String, i128)>)> = Vec::new();
    let top: Vec<&str> = ast.lines().collect();
    let mut k = 0;
    while k < top.len() {
        let Some(rest) = top[k].strip_prefix("|-").or_else(|| top[k].strip_prefix("`-")) else { k += 1; continue };
        if !rest.starts_with("EnumDecl ") { k += 1; continue; }
        let last = rest.split_whitespace().last().unwrap_or("");
        let mut name = (!last.contains(':') && !last.contains('>')).then(|| last.to_string());
        let mut cases: Vec<(String, Option<i128>)> = Vec::new();
        k += 1;
        while k < top.len() && (top[k].starts_with("| ") || top[k].starts_with("  ")) {
            let l = top[k];
            if let Some(i) = l.find("EnumConstantDecl ") {
                let rest = &l[i..];
                if let Some(q) = rest.find('\'') { cases.push((rest[..q].split_whitespace().last().unwrap_or("").to_string(), None)); }
            } else if let Some(i) = l.find("value: Int ") {
                if let (Some(c), Ok(v)) = (cases.last_mut(), l[i + 11..].trim().parse::<i128>()) { if c.1.is_none() { c.1 = Some(v); } }
            }
            k += 1;
        }
        if name.is_none() {
            if let Some(next) = top.get(k).and_then(|l| l.strip_prefix("|-").or_else(|| l.strip_prefix("`-"))) {
                if next.starts_with("TypedefDecl ") {
                    if let Some(q) = next.find("'enum ") { let after = &next[q + 6..]; if let Some(e) = after.find('\'') { name = Some(after[..e].to_string()); } }
                }
            }
        }
        let mut v = -1i128;
        let cases: Vec<(String, i128)> = cases.into_iter().map(|(n, x)| { v = x.unwrap_or(v + 1); (n, v) }).collect();
        if !cases.is_empty() { enums.push((name, cases)); }
    }
    // C's structs: each tag's fields (None when some part is beyond a plain field), and its Zag name,
    // C's type name when it has one, else the tag (owner, 2026-10-10: `FILE`, `tm`)
    let mut records: HashMap<String, Option<Vec<(String, String)>>> = HashMap::new();
    let mut current: Option<(String, Option<Vec<(String, String)>>)> = None;
    for l in ast.lines() {
        if let Some(rest) = l.strip_prefix("|-").or_else(|| l.strip_prefix("`-")) {
            if let Some((tag, fields)) = current.take() {
                // an unnamed struct takes the name of the `typedef` right after it (`typedef struct { … } div_t;`,
                // which Clang writes `'struct div_t'`) (compiler claude)
                let tag = if tag.is_empty() {
                    rest.strip_prefix("TypedefDecl ").and_then(|r| r.find("'struct ").map(|q| &r[q + 8..])).and_then(|a| a.find('\'').map(|e| a[..e].to_string())).unwrap_or_default()
                } else { tag };
                if !tag.is_empty() { records.insert(tag, fields); }
            }
            if rest.starts_with("RecordDecl ") && rest.ends_with(" definition") {
                let words: Vec<&str> = rest.split_whitespace().collect();
                let n = words.len();
                if n >= 3 && words[n - 3] == "struct" { current = Some((words[n - 2].to_string(), Some(Vec::new()))); }
                else if n >= 2 && words[n - 2] == "struct" { current = Some((String::new(), Some(Vec::new()))); }
                else if n >= 3 && words[n - 3] == "union" { records.insert(words[n - 2].to_string(), None); }
            }
            continue;
        }
        let Some((_, fields)) = current.as_mut() else { continue };
        // a layout C changes by an attribute, on the struct or one field (`packed`, `aligned`, `#pragma
        // pack`), is beyond a plain field: the struct comes through opaque (compiler claude)
        if ["PackedAttr", "AlignedAttr", "MaxFieldAlignmentAttr"].iter().any(|a| l.contains(a)) { *fields = None; continue; }
        // a struct's fields; the last top-level declaration's are indented with spaces (compiler claude)
        let Some(child) = ["| |-", "| `-", "  |-", "  `-"].iter().find_map(|p| l.strip_prefix(p)) else { continue };
        if !child.starts_with("FieldDecl ") { *fields = None; continue; }   // a nested struct or union, an attribute: beyond a plain field
        let Some(q) = child.find('\'') else { *fields = None; continue };
        let fname = child[..q].split_whitespace().last().unwrap_or("").to_string();
        let after = &child[q + 1..];
        let Some(e) = after.find('\'') else { *fields = None; continue };
        // C's own type names are kept (`size_t` is `C.size_t`); `map` follows any other
        let fty = &after[..e];
        if let Some(f) = fields.as_mut() { f.push((fname, fty.to_string())); }
    }
    if let Some((tag, fields)) = current.take() { records.insert(tag, fields); }
    let mut names: HashMap<String, String> = HashMap::new();
    for (alias, target) in &td {
        if let Some(tag) = target.strip_prefix("struct ") { if good_name(alias) && !alias.starts_with('_') { names.entry(tag.to_string()).or_insert_with(|| alias.clone()); } }
    }
    // a named enum's Zag name: C's type name for it when it has one, else its tag (as for structs)
    let mut enum_names: HashMap<String, String> = HashMap::new();
    for (tag, _) in enums.iter().filter_map(|(n, c)| n.as_ref().map(|n| (n, c))) {
        let alias = td.iter().find(|(a, t)| t.as_str() == format!("enum {tag}") && *a != tag && good_name(a) && !a.starts_with('_')).map(|(a, _)| a.clone());
        let n = alias.unwrap_or_else(|| tag.clone());
        if good_name(&n) && !crate::spellings::is_reserved(&n) { enum_names.insert(tag.clone(), n); }
    }
    let mut cx = Cx { td: &td, records: &records, names: &names, enums: &enum_names, layout: HashMap::new(), opaque: Vec::new(), busy: Vec::new() };
    let mut fns: Vec<(String, String)> = Vec::new();   // (name, line), the last declaration of a name winning
    let mut vars: Vec<(String, String)> = Vec::new();
    for l in ast.lines() {
        let Some(rest) = l.strip_prefix("|-").or_else(|| l.strip_prefix("`-")) else { continue };
        let is_fn = rest.starts_with("FunctionDecl ");
        if !is_fn && !rest.starts_with("VarDecl ") { continue; }
        let Some(q) = rest.find('\'') else { continue };
        // Clang declares a builtin by itself (`implicit`), in its own words; the header's line follows
        if rest[..q].split_whitespace().any(|w| w == "implicit") { continue; }
        let name = rest[..q].split_whitespace().last().unwrap_or("").to_string();
        let after = &rest[q + 1..];
        let Some(e) = after.find('\'') else { continue };
        let ty = &after[..e];
        let mut tail = &after[e + 1..];
        if let Some(d) = tail.strip_prefix(":'") { if let Some(e2) = d.find('\'') { tail = &d[e2 + 1..]; } }
        // 2. `__attribute__((noreturn))` and its kind say nothing Zag's declaration holds (compiler claude)
        let cleaned = strip_attributes(ty);
        let ty = cleaned.as_str();
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
    let mut enum_lines = Vec::new();
    for (tag, cases) in &enums {
        if cases.iter().any(|(n, _)| !good_name(n)) { continue; }
        match tag.as_ref().and_then(|t| enum_names.get(t)) {
            Some(name) => {
                // a later name for a number already named is a const (owner, 2026-10-12: "(a)"): the first
                // is the case, `COLOR_FIRST = RED` a const holding the same number
                let mut seen: Vec<i128> = Vec::new();
                let mut own = Vec::new();
                for (n, v) in cases {
                    if seen.contains(v) {
                        if !consts.iter().any(|c: &String| c.starts_with(&format!("const '{n}' "))) { consts.push(format!("const '{n}' := {v};")); }
                    } else { seen.push(*v); own.push(format!("{n} = {v}")); }
                }
                enum_lines.push(format!("layout.C enum {name} {{ {} }}", own.join(", ")));
            }
            None if tag.is_none() => for (n, v) in cases { if !consts.iter().any(|c: &String| c.starts_with(&format!("const '{n}' "))) { consts.push(format!("const '{n}' := {v};")); } },
            None => {}
        }
    }
    // every struct of the header that has a name, whole when Zag can spell its fields, else opaque
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
        lines: enum_lines.into_iter().chain(structs.into_iter().map(|(_, l)| l)).chain(fns.into_iter().map(|(_, l)| l)).chain(vars.into_iter().map(|(_, l)| l)).chain(consts).collect(),
        opaque,
    })
}

/// What mapping a type needs: C's type names, its structs, and the structs met so far.
struct Cx<'a> {
    td: &'a HashMap<String, String>,
    records: &'a HashMap<String, Option<Vec<(String, String)>>>,
    names: &'a HashMap<String, String>,
    /// named enums: tag → Zag name
    enums: &'a HashMap<String, String>,
    /// structs that come through whole: tag → its `layout.C struct` line
    layout: HashMap<String, String>,
    /// Zag names of structs that come through opaque
    opaque: Vec<String>,
    /// tags being worked out, so a struct that points at itself is not followed forever
    busy: Vec<String>,
}

/// A C name Zag can hold: no leading `__` (the C library's own insides). A Zag keyword is held
/// too (owner, 2026-10-10: "(b)"): a field is read after its dot, a function reached by path.
fn good_name(n: &str) -> bool {
    !n.is_empty() && !n.starts_with("__") && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !n.starts_with(|c: char| c.is_ascii_digit())
}

/// A struct's Zag name: C's type name for it, else its tag; None for the C library's own (`__x`).
fn struct_name(tag: &str, cx: &Cx) -> Option<String> {
    let n = cx.names.get(tag).cloned().unwrap_or_else(|| tag.to_string());
    (good_name(&n) && !crate::spellings::is_reserved(&n)).then_some(n)
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
            Some(fs) => { cx.layout.insert(tag.to_string(), format!("layout.C struct {name} {{ {} }}", fs.join(", "))); true }
            None => false,
        }
    };
    if whole { return Some(name); }
    if !cx.opaque.contains(&name) { cx.opaque.push(name.clone()); }
    if by_value { None } else { Some(name) }
}

/// `int (const char *, ...)` as an `extern.C fn` line, or None when some part has no Zag spelling yet.
fn function(name: &str, ty: &str, cx: &mut Cx) -> Option<String> {
    let open = ty.find('(')?;
    if !ty.ends_with(')') || ty[..open].contains(')') { return None; }
    let ret = strip(&ty[..open]);
    let ret = if ret == "void" { String::new() } else { format!(" -> {}", map(ret, cx, 0)?) };
    let inside = ty[open + 1..ty.len() - 1].trim();
    let mut params = Vec::new();
    if !inside.is_empty() && inside != "void" {
        for (i, p) in split_top(inside).into_iter().enumerate() {
            if p == "..." { params.push("...'args'".to_string()); continue; }
            params.push(format!("'p{}': {}", i + 1, map(strip(&p), cx, 0)?));
        }
    }
    Some(format!("extern.C fn {name} [{}]{ret};", params.join(", ")))
}

/// A C type without the words that change nothing Zag can say (`restrict`, the nullability marks).
fn strip(t: &str) -> &str { t.trim() }

/// A C type as Zag writes it at the C boundary; `const` before a pointer's target makes it `ptr`, else
/// `ptr.mut`. `void *` points at bytes, `uint8`.
fn map(t: &str, cx: &mut Cx, depth: usize) -> Option<String> {
    if depth > 16 { return None; }
    let mut t = t.to_string();
    for mark in ["__restrict", "restrict", "_Nullable", "_Nonnull", "_Null_unspecified", "volatile", "__unsafe_indexable", "__single"] { t = t.replace(mark, " "); }
    let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
    // a function pointer, `int (*)(const void *, const void *)`: Zag's callback type, `fn[…] -> T`
    // (2026-09-21), parameters by value
    if let Some(p) = t.find("(*") {
        let close = p + t[p..].find(')')?;
        if !t[p + 2..close].trim().is_empty() { return None; }
        let rest = t[close + 1..].trim();
        let inside = rest.strip_prefix('(')?.strip_suffix(')')?.trim();
        let ret = t[..p].trim();
        let mut ps = Vec::new();
        if !inside.is_empty() && inside != "void" {
            for q in split_top(inside) { if q == "..." { return None; } ps.push(map(&q, cx, depth + 1)?); }
        }
        let ret = if ret == "void" { String::new() } else { format!(" -> {}", map(ret, cx, depth + 1)?) };
        return Some(format!("fn[{}]{ret}", ps.join(", ")));
    }
    let t = t.replace(" *", "*").replace("* ", "*");
    if let Some(inner) = t.strip_suffix('*') {
        let inner = inner.trim();
        // what this pointer points at is `const` when the word stands next to it: `const char *`, or
        // `char *const *` for a pointer to a pointer; in `const char **` the `const` is the innermost
        // target's, so this pointer is `ptr.mut` (compiler claude)
        let (is_const, base) = if let Some(b) = inner.strip_suffix("*const") { (true, format!("{b}*")) }
            else if inner.ends_with('*') { (false, inner.to_string()) }
            else { match inner.strip_prefix("const ") { Some(b) => (true, b.trim().to_string()), None => match inner.strip_suffix(" const") { Some(b) => (true, b.trim().to_string()), None => (false, inner.to_string()) } } };
        let base = base.as_str();
        let target = if base == "void" || cx.td.get(base).is_some_and(|t| t == "void") { "uint8".to_string() }
            else if let Some(tag) = struct_tag(base, cx) { record(&tag, cx, depth, false)? }
            else { map(base, cx, depth + 1)? };
        return Some(format!("ptr{}[{target}]", if is_const { "" } else { ".mut" }));
    }
    let base = t.strip_prefix("const ").unwrap_or(&t).trim();
    if let Some(tag) = struct_tag(base, cx) { return record(&tag, cx, depth, true); }
    // a named enum is its Zag enum (owner, 2026-10-10); one Zag has no name for is an `int`, as C's are
    if let Some(tag) = base.strip_prefix("enum ") { return Some(cx.enums.get(tag.trim()).cloned().unwrap_or_else(|| "C.int".to_string())); }
    // C's machine-sized types are the `C.` types (owner, 2026-10-10), C's words kept; a type the same size
    // everywhere is Zag's own (`int32_t` is `int32`, `double` is `bin64`)
    Some(match base {
        "char" => "C.char", "signed char" => "C.signed_char", "unsigned char" => "C.unsigned_char",
        "short" | "short int" | "signed short" | "signed short int" => "C.short", "unsigned short" | "unsigned short int" => "C.unsigned_short",
        "int" | "signed int" | "signed" => "C.int", "unsigned int" | "unsigned" => "C.unsigned_int",
        "long" | "long int" | "signed long" | "signed long int" => "C.long", "unsigned long" | "unsigned long int" => "C.unsigned_long",
        "long long" | "long long int" | "signed long long" | "signed long long int" => "C.long_long",
        "unsigned long long" | "unsigned long long int" => "C.unsigned_long_long",
        "long double" => "C.long_double",
        "size_t" | "ssize_t" | "ptrdiff_t" | "intptr_t" | "uintptr_t" | "wchar_t" => return Some(format!("C.{base}")),
        "int8_t" => "int8", "int16_t" => "int16", "int32_t" => "int32", "int64_t" => "int64",
        "uint8_t" => "uint8", "uint16_t" => "uint16", "uint32_t" => "uint32", "uint64_t" => "uint64",
        "float" => "bin32", "double" => "bin64", "_Bool" | "bool" => "bool",
        // any other C type name stands for the type it names
        name => { let t = cx.td.get(name)?.clone(); return map(&t, cx, depth + 1); }
    }.to_string())
}

/// A `#define`'s value as a Zag literal, when it is a plain number (in decimal: C's `0644` is octal)
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
    // `typedef struct _opaque_pthread_t *pthread_t;` names a pointer, which `map` follows (compiler claude)
    if target.contains('*') { return None; }
    target.strip_prefix("struct ").map(|tag| tag.trim().to_string())
}

/// A C type without its `__attribute__((…))` parts.
fn strip_attributes(t: &str) -> String {
    let mut out = t.to_string();
    while let Some(i) = out.find("__attribute__((") {
        let mut depth = 0;
        let mut end = out.len();
        for (j, c) in out[i..].char_indices() {
            match c { '(' => depth += 1, ')' => { depth -= 1; if depth == 0 { end = i + j + 1; break; } } _ => {} }
        }
        out.replace_range(i..end, "");
    }
    out.trim().to_string()
}

/// A C parameter list split at its top-level commas, so a function pointer's own list stays whole.
fn split_top(s: &str) -> Vec<String> {
    let (mut out, mut depth, mut cur) = (Vec::new(), 0i32, String::new());
    for c in s.chars() {
        match c {
            '(' => { depth += 1; cur.push(c); }
            ')' => { depth -= 1; cur.push(c); }
            ',' if depth == 0 => { out.push(cur.trim().to_string()); cur.clear(); }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() { out.push(cur.trim().to_string()); }
    out
}
