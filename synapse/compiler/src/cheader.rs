//! C headers (owner, 2026-10-10): `incl.C.system["stdio.h"];` and
//! `incl.C["vendor/mylib.h"];` read while building, so C's functions need
//! no hand-written `extern.C`. An `incl.C` line is an `extern.C` block that
//! Synapse writes: Clang reads the headers (its AST, as JSON, and its
//! `#define`s, less its own), and what Nerve can say of them is written
//! out as Nerve, which the parser then reads as if it stood on the line.
//!
//! What comes through (owner: functions, types, globals and plain
//! `#define` constants; the picks in oracle/README.md "What of a C header
//! comes through"): a function or `extern` global whose types are numbers,
//! `bool` and pointers to them, `const` before the target making `ptr`,
//! else `ptr.mut`; plain `char` is `uint8`; `void *` points at `uint8`; a
//! type name stands for what it names; a `#define` that is a whole number
//! (in decimal here, so `0644` is 420), possibly negative or in
//! parentheses, or text with no `\` or `"` in it. Left out: structs and
//! unions, function pointers, `long double`, enums, a pointer that is
//! itself `const` (`char *const`), names starting `__`, Nerve's keywords,
//! `static` functions, and function-like macros.

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The Clang beside the LLVM Synapse is built with, else the one on `PATH`.
fn clang() -> PathBuf {
    let beside = Path::new(env!("SYNAPSE_LLVM_PREFIX")).join("bin").join("clang");
    if beside.is_file() {
        beside
    } else {
        PathBuf::from("clang")
    }
}

/// The Nerve declarations an `incl.C` line stands for, as source text, one
/// item a line; or why there are none, said after the line's own spelling.
pub fn declare(system: bool, paths: &[String], root: &Path) -> Result<Vec<Decl>, String> {
    let mut unit = String::new();
    for p in paths {
        if system {
            unit.push_str(&format!("#include <{}>\n", p));
        } else {
            let full = root.join(p);
            if !full.is_file() {
                return Err(format!("cannot read \"{}\"", p));
            }
            let full = full.canonicalize().unwrap_or(full);
            unit.push_str(&format!("#include \"{}\"\n", full.display()));
        }
    }
    // The three runs at once: the AST, the macros, and Clang's own macros
    // to leave out.
    let ast = run(&["-fsyntax-only", "-Xclang", "-ast-dump=json"], &unit)?;
    let defines = run(&["-E", "-dM"], &unit)?;
    let own = run(&["-E", "-dM"], "")?;
    let ast = ast.wait()?;
    let defines = defines.wait()?;
    let own: HashSet<String> = own.wait()?.lines().map(str::to_string).collect();
    let tree = Json::parse(&ast).ok_or("Clang's AST could not be read")?;
    let mut out = Decls::default();
    let defines: Vec<&str> = defines.lines().filter(|l| !own.contains(*l)).collect();
    // An unnamed enum's name a `#define` also makes is the define's.
    out.defined = defines.iter().filter_map(|l| define_value(l)).map(|(n, _)| n.to_string()).collect();
    out.from_ast(&tree);
    for line in defines {
        out.define(line);
    }
    Ok(out.list)
}

/// One declaration: its name, and the Nerve for it.
pub struct Decl {
    pub name: String,
    pub text: String,
}

struct Running(std::process::Child);

impl Running {
    fn wait(self) -> Result<String, String> {
        let out = self.0.wait_with_output().map_err(|e| format!("the C compiler did not run: {}", e))?;
        if !out.status.success() {
            let said = String::from_utf8_lossy(&out.stderr);
            let first = said.lines().find_map(|l| l.split_once("error: ").map(|(_, r)| r.to_string())).unwrap_or_else(|| said.trim().to_string());
            return Err(format!("the C compiler says: {}", first));
        }
        String::from_utf8(out.stdout).map_err(|_| "the C compiler wrote what is not UTF-8".to_string())
    }
}

fn run(args: &[&str], unit: &str) -> Result<Running, String> {
    let mut child = Command::new(clang())
        .args(["-x", "c"])
        .args(args)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("the C compiler (Clang) did not start: {}", e))?;
    child.stdin.take().unwrap().write_all(unit.as_bytes()).map_err(|e| format!("the C compiler did not read the headers: {}", e))?;
    Ok(Running(child))
}

#[derive(Default)]
struct Decls {
    list: Vec<Decl>,
    seen: HashSet<String>,
    typedefs: HashMap<String, String>,
    /// C's structs by tag (`anon…` for one with none), and what Nerve
    /// calls each: C's type name for it, else its tag (owner, 2026-10-10).
    records: HashMap<String, Record>,
    names: HashMap<String, String>,
    /// A type name that names a struct, and which.
    typedef_records: HashMap<String, String>,
    whole: std::cell::RefCell<HashMap<String, bool>>,
    /// C's enums by tag (`anon…` for one with none), their cases, and what
    /// Nerve calls each (owner, 2026-10-10), named as a struct is.
    enums: HashMap<String, Vec<(String, i128)>>,
    enum_names: HashMap<String, String>,
    typedef_enums: HashMap<String, String>,
    /// The names the header's `#define`s make.
    defined: HashSet<String>,
}

#[derive(Default)]
struct Record {
    /// Its fields, once C defines it: name and type.
    fields: Option<Vec<(String, String)>>,
    /// A field Nerve cannot spell however its type reads: a bit field, a
    /// struct or union written inside it.
    odd: bool,
}

/// Whether a field says its own packing or alignment.
fn laid_out_its_own_way(f: &Json) -> bool {
    matches!(f.get("inner"), Some(Json::Arr(v)) if v.iter().any(|a| matches!(a.str("kind"), Some("PackedAttr" | "AlignedAttr"))))
}

/// The key of a struct: its tag, or one made from Clang's id.
fn record_key(d: &Json) -> Option<String> {
    // Clang says `""` for the name of an unnamed one a typedef points at.
    match d.str("name").filter(|n| !n.is_empty()) {
        Some(n) => Some(n.to_string()),
        None => Some(format!("anon{}", d.str("id")?.trim_start_matches("0x"))),
    }
}

impl Decls {
    fn push(&mut self, name: &str, text: String) {
        if self.seen.insert(name.to_string()) {
            self.list.push(Decl { name: name.to_string(), text });
        }
    }

    fn from_ast(&mut self, tree: &Json) {
        let Some(Json::Arr(top)) = tree.get("inner") else { return };
        // The types first: a function may name a struct defined after it.
        let mut typedef_order = Vec::new();
        let mut enum_typedefs = Vec::new();
        for d in top {
            match d.str("kind") {
                Some("TypedefDecl") => {
                    let Some(name) = d.str("name") else { continue };
                    if let Some(t) = d.get("type").and_then(qual) {
                        self.typedefs.insert(name.to_string(), t.to_string());
                    }
                    // A type name for another type name names what that
                    // one does (`pthread_attr_t`, through
                    // `__darwin_pthread_attr_t`, names the struct).
                    let named = match d.get("inner") {
                        Some(Json::Arr(inner)) => inner.iter().find(|i| i.str("kind") == Some("TypedefType")).and_then(|i| i.get("decl")).and_then(|d| d.str("name")).map(str::to_string),
                        _ => None,
                    };
                    let record = match d.get("inner") {
                        Some(Json::Arr(inner)) => inner.iter().find(|i| i.str("kind") == Some("RecordType")).and_then(|i| i.get("decl")).filter(|d| d.str("kind") == Some("RecordDecl")).and_then(record_key),
                        _ => None,
                    }
                    .or_else(|| named.as_ref().and_then(|n| self.typedef_records.get(n).cloned()));
                    if let Some(k) = record {
                        self.typedef_records.insert(name.to_string(), k.clone());
                        typedef_order.push((name.to_string(), k));
                    }
                    let en = match d.get("inner") {
                        Some(Json::Arr(inner)) => inner.iter().find(|i| i.str("kind") == Some("EnumType")).and_then(|i| i.get("decl")).and_then(record_key),
                        _ => None,
                    }
                    .or_else(|| named.as_ref().and_then(|n| self.typedef_enums.get(n).cloned()));
                    if let Some(k) = en {
                        self.typedef_enums.insert(name.to_string(), k.clone());
                        enum_typedefs.push((name.to_string(), k));
                    }
                }
                Some("EnumDecl") => {
                    let Some(k) = record_key(d) else { continue };
                    let Some(Json::Arr(inner)) = d.get("inner") else { continue };
                    // Each case's number, counted on from the last written.
                    let mut cases = Vec::new();
                    let mut next: i128 = 0;
                    for c in inner.iter().filter(|c| c.str("kind") == Some("EnumConstantDecl")) {
                        let Some(n) = c.str("name") else { continue };
                        let written = match c.get("inner") {
                            Some(Json::Arr(v)) => v.iter().find_map(|e| e.str("value")).and_then(|v| v.parse::<i128>().ok()),
                            _ => None,
                        };
                        let v = written.unwrap_or(next);
                        cases.push((n.to_string(), v));
                        next = v + 1;
                    }
                    self.enums.insert(k, cases);
                }
                // A union is only pointed at, as an opaque struct is (oracle
                // README): Nerve has no spelling for its fields sharing room.
                Some("RecordDecl") if d.str("tagUsed") == Some("union") => {
                    let Some(k) = record_key(d) else { continue };
                    self.records.entry(k).or_default().odd = true;
                }
                Some("RecordDecl") if d.str("tagUsed") == Some("struct") => {
                    let Some(k) = record_key(d) else { continue };
                    let r = self.records.entry(k).or_default();
                    if d.get("completeDefinition").is_some() {
                        let mut fields = Vec::new();
                        if let Some(Json::Arr(inner)) = d.get("inner") {
                            for f in inner {
                                match f.str("kind") {
                                    Some("FieldDecl") => {
                                        let (Some(n), Some(t)) = (f.str("name"), f.get("type").and_then(qual)) else {
                                            r.odd = true;
                                            continue;
                                        };
                                        if f.get("isBitfield").is_some() || laid_out_its_own_way(f) {
                                            r.odd = true;
                                        }
                                        fields.push((n.to_string(), t.to_string()));
                                    }
                                    Some("RecordDecl") => r.odd = true,
                                    // Packed or aligned its own way (`#pragma
                                    // pack`): `layout.C` lays out as C does
                                    // by default, so it is only pointed at.
                                    Some("PackedAttr" | "MaxFieldAlignmentAttr" | "AlignedAttr") => r.odd = true,
                                    _ => {}
                                }
                            }
                        }
                        r.fields = Some(fields);
                    }
                }
                _ => {}
            }
        }
        // C's type name, when one names the struct and is not C's own
        // (`FILE`, not `__sFILE`); else its tag, unless that is C's own.
        for (name, k) in &typedef_order {
            if !name.starts_with('_') && usable_type(name) && !self.names.contains_key(k) {
                self.names.insert(k.clone(), name.clone());
            }
        }
        let mut keys: Vec<&String> = self.records.keys().collect();
        keys.sort();
        for k in keys {
            if !self.names.contains_key(k) && !k.starts_with("__") && !k.starts_with("anon") && usable_type(k) {
                self.names.insert(k.clone(), k.clone());
            }
        }
        // An enum is named as a struct is; one with neither C's type name
        // nor a tag has its names as consts, unless a `#define` made one.
        for (name, k) in &enum_typedefs {
            if !name.starts_with('_') && usable_type(name) && !self.enum_names.contains_key(k) {
                self.enum_names.insert(k.clone(), name.clone());
            }
        }
        for d in top {
            if d.str("kind") != Some("EnumDecl") {
                continue;
            }
            let Some(k) = record_key(d) else { continue };
            if !self.enum_names.contains_key(&k) && !k.starts_with("anon") && !k.starts_with("__") && usable_type(&k) {
                self.enum_names.insert(k.clone(), k.clone());
            }
            let Some(cases) = self.enums.get(&k).cloned() else { continue };
            match self.enum_names.get(&k).cloned() {
                Some(name) => {
                    // The first name for a number is the case; a later name
                    // for it is a const holding it (owner, 2026-10-12: "a"),
                    // unless a const of that name is already there.
                    let mut numbers = HashSet::new();
                    let mut body = Vec::new();
                    let mut also = Vec::new();
                    for (c, v) in &cases {
                        if numbers.insert(*v) {
                            body.push(format!("{} = {}", c, v));
                        } else {
                            also.push((c.clone(), *v));
                        }
                    }
                    self.push(&format!("type {}", name), format!("layout.C enum {} {{ {} }}", name, body.join("  ")));
                    for (c, v) in also {
                        if usable(&c) && !self.defined.contains(&c) {
                            self.push(&c, format!("const '{}' := {};", c, v));
                        }
                    }
                }
                None if k.starts_with("anon") && !enum_typedefs.iter().any(|(_, t)| *t == k) => {
                    for (c, v) in cases {
                        if usable(&c) && !self.defined.contains(&c) {
                            self.push(&c, format!("const '{}' := {};", c, v));
                        }
                    }
                }
                None => {}
            }
        }
        // Each struct Nerve can name, in the order C declares them.
        let mut order: Vec<String> = Vec::new();
        for d in top {
            if d.str("kind") == Some("RecordDecl") {
                if let Some(k) = record_key(d) {
                    if self.names.contains_key(&k) && !order.contains(&k) {
                        order.push(k);
                    }
                }
            }
        }
        for k in order {
            let name = self.names[&k].clone();
            if self.whole(&k) {
                let fields: Vec<String> = self.records[&k].fields.iter().flatten().map(|(f, t)| format!("{}: {}", f, self.ty(t).unwrap())).collect();
                self.push(&format!("type {}", name), format!("layout.C struct {} {{ {} }}", name, fields.join("  ")));
            } else {
                self.push(&format!("type {}", name), format!("layout.C struct {};", name));
            }
        }
        for d in top {
            let kind = d.str("kind").unwrap_or("");
            let Some(name) = d.str("name") else { continue };
            if kind == "TypedefDecl" || kind == "RecordDecl" || kind == "EnumDecl" {
                continue;
            }
            if d.get("isImplicit").is_some() || !usable(name) || self.seen.contains(name) {
                continue;
            }
            match kind {
                "FunctionDecl" if d.str("storageClass") != Some("static") => {
                    if let Some(text) = self.function(name, d) {
                        self.push(name, text);
                    }
                }
                "VarDecl" if d.str("storageClass") == Some("extern") => {
                    let Some(t) = d.get("type").and_then(qual) else { continue };
                    let read_only = top_const(t);
                    if let Some(ty) = self.ty(t) {
                        self.push(name, format!("extern.C {}'{}': {};", if read_only { "" } else { "mut " }, name, ty));
                    }
                }
                _ => {}
            }
        }
    }

    fn function(&self, name: &str, d: &Json) -> Option<String> {
        let whole = d.get("type")?.str("qualType")?;
        // A function that hands back a function pointer, or a block, is
        // left out.
        let open = whole.find('(')?;
        if whole[open..].starts_with("(*") || whole[open..].starts_with("(^") {
            return None;
        }
        let ret = whole[..open].trim();
        let ret = if self.resolve(ret).trim() == "void" { None } else { Some(self.ty(ret)?) };
        let mut params = Vec::new();
        if let Some(Json::Arr(inner)) = d.get("inner") {
            for p in inner.iter().filter(|p| p.str("kind") == Some("ParmVarDecl")) {
                let t = p.get("type").and_then(qual)?;
                params.push(format!("'p{}': {}", params.len() + 1, self.ty(t)?));
            }
        }
        if d.get("variadic").is_some() {
            params.push("...'args'".to_string());
        }
        let ret = ret.map(|r| format!(" -> {}", r)).unwrap_or_default();
        Some(format!("extern.C fn {} [{}]{};", name, params.join(", "), ret))
    }

    /// A type name replaced by what it names, until only C's own words are
    /// left.
    fn resolve(&self, t: &str) -> String {
        let mut words: Vec<String> = Vec::new();
        for w in split(t) {
            // `struct X` names a tag, which a typedef may share.
            let tagged = words.last().is_some_and(|x| matches!(x.as_str(), "struct" | "union" | "enum"));
            if !tagged {
                if let Some(k) = self.typedef_records.get(&w) {
                    words.push("struct".into());
                    words.push(k.clone());
                    continue;
                }
                if let Some(k) = self.typedef_enums.get(&w) {
                    words.push("enum".into());
                    words.push(k.clone());
                    continue;
                }
                // `size_t` and its kind keep their names, and a type the
                // same size everywhere is Nerve's own (owner, 2026-10-10).
                if KEPT.contains(&w.as_str()) {
                    words.push(w);
                    continue;
                }
                // Clang's own names for them, which no header declares
                // (Clang 22's predefined sugar).
                let predefined = match w.as_str() {
                    "__size_t" => Some("size_t"),
                    "__signed_size_t" => Some("ssize_t"),
                    "__ptrdiff_t" => Some("ptrdiff_t"),
                    _ => None,
                };
                if let Some(p) = predefined {
                    words.push(p.into());
                    continue;
                }
            }
            match self.typedefs.get(&w) {
                Some(under) if !tagged && !is_c_word(&w) => {
                    // `const T` where T is a pointer is a `const` pointer.
                    let under = self.resolve(under);
                    if words.iter().any(|x| x == "const") && under.contains('*') {
                        return "?".into();
                    }
                    words.extend(split(&under));
                }
                _ => words.push(w),
            }
        }
        words.join(" ")
    }

    /// The Nerve type of a C type, or `None` if Nerve does not say it yet.
    /// A value's own `const` (`const int x`) changes nothing here.
    fn ty(&self, t: &str) -> Option<String> {
        // A function pointer is Nerve's callback type (owner, 2026-10-10),
        // named directly or through a type name.
        let t = t.replace(" _Nullable", "").replace(" _Nonnull", "").replace(" _Null_unspecified", "");
        let t = t.trim();
        if let Some(under) = self.typedefs.get(t) {
            if under.contains("(*)") || under.contains("(* _") {
                return self.ty(under);
            }
        }
        if let Some(at) = t.find("(*)") {
            return self.fn_pointer(&t[..at], &t[at + 3..]);
        }
        let words: Vec<String> = split(&self.resolve(t)).into_iter().filter(|w| !IGNORED.contains(&w.as_str())).collect();
        if words.iter().any(|w| w == "[" || w == "(" || w == "?") {
            return None;
        }
        // Each pointer is `ptr` when `const` stands next to its target:
        // `const T *`, or `T *const *` for the outer one. A `const` after
        // the last `*` is a pointer that is itself fixed, which Nerve has
        // no spelling for yet.
        let first = words.iter().position(|w| w == "*").unwrap_or(words.len());
        let mut target_const = vec![words[..first].iter().any(|w| w == "const")];
        for (i, w) in words.iter().enumerate().skip(first) {
            match w.as_str() {
                "*" => target_const.push(words.get(i + 1).is_some_and(|n| n == "const")),
                "const" => {}
                _ => return None,
            }
        }
        if target_const.pop() == Some(true) && first < words.len() {
            return None;
        }
        let stars = target_const.len();
        let base: Vec<&str> = words[..first].iter().map(String::as_str).filter(|w| *w != "const").collect();
        // The pointers from the innermost out, each `ptr` or `ptr.mut`.
        let wrap = |mut out: String| {
            for c in &target_const {
                out = if *c { format!("ptr[{}]", out) } else { format!("ptr.mut[{}]", out) };
            }
            out
        };
        // A struct: by value only when Nerve sees inside it; pointed at,
        // whole or opaque.
        if let ["struct" | "union", key] = base.as_slice() {
            let name = self.names.get(*key)?;
            if stars == 0 && !self.whole(key) {
                return None;
            }
            return Some(wrap(name.clone()));
        }
        if base.contains(&"struct") || base.contains(&"union") {
            return None;
        }
        // An enum: its name, or C's `int` when Nerve has none for it.
        if let ["enum", key] = base.as_slice() {
            return Some(wrap(self.enum_names.get(*key).cloned().unwrap_or_else(|| "C.int".into())));
        }
        if base.contains(&"enum") {
            return None;
        }
        let out = match (base_type(&base)?, stars) {
            ("void", 0) => return None,
            ("void", _) => "uint8".to_string(),
            (b, _) => b.to_string(),
        };
        Some(wrap(out))
    }

    /// `R (*)(P, …)`: `fn[P, …] -> R`, `fn[]` taking nothing; a variadic
    /// one is left out.
    fn fn_pointer(&self, ret: &str, rest: &str) -> Option<String> {
        let rest = rest.trim();
        if ret.contains('(') || !rest.starts_with('(') || !rest.ends_with(')') {
            return None;
        }
        let inner = &rest[1..rest.len() - 1];
        let mut params = Vec::new();
        let (mut depth, mut from) = (0, 0);
        let bytes = inner.as_bytes();
        for i in 0..=bytes.len() {
            let end = i == bytes.len();
            if !end {
                match bytes[i] {
                    b'(' => depth += 1,
                    b')' => depth -= 1,
                    _ => {}
                }
            }
            if end || (bytes[i] == b',' && depth == 0) {
                let p = inner[from..i].trim();
                if p == "..." {
                    return None;
                }
                if !p.is_empty() && p != "void" {
                    params.push(self.ty(p)?);
                }
                from = i + 1;
            }
        }
        let ret = ret.trim();
        let ret = if self.resolve(ret).trim() == "void" { String::new() } else { format!(" -> {}", self.ty(ret)?) };
        Some(format!("fn[{}]{}", params.join(", "), ret))
    }

    /// Whether every field of the struct is one Nerve can spell (a number,
    /// `bool`, a pointer, or a struct whole in turn), so it comes through as
    /// a `layout.C struct`; else it is opaque, only pointed at.
    fn whole(&self, key: &str) -> bool {
        if let Some(w) = self.whole.borrow().get(key) {
            return *w;
        }
        self.whole.borrow_mut().insert(key.to_string(), false);
        let ok = match self.records.get(key) {
            Some(Record { fields: Some(fields), odd: false }) => !fields.is_empty() && fields.iter().all(|(_, t)| self.ty(t).is_some()),
            _ => false,
        };
        self.whole.borrow_mut().insert(key.to_string(), ok);
        ok
    }

    fn define(&mut self, line: &str) {
        let Some((name, value)) = define_value(line) else { return };
        if !self.seen.contains(name) {
            self.push(name, format!("const '{}' := {};", name, value));
        }
    }
}

/// A `#define` that makes a const: its name and its value as Nerve writes
/// it — a whole number, or text with no `\` or `"` inside.
fn define_value(line: &str) -> Option<(&str, String)> {
    let rest = line.strip_prefix("#define ")?;
    let i = rest.find(|c: char| c == ' ' || c == '(')?;
    if rest.as_bytes()[i] == b'(' {
        return None;
    }
    let (name, value) = (&rest[..i], rest[i + 1..].trim());
    if !usable(name) {
        return None;
    }
    if let Some(n) = whole_number(value) {
        return Some((name, n));
    }
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') && !value[1..value.len() - 1].contains(['"', '\\']) {
        return Some((name, value.to_string()));
    }
    None
}

/// A name Nerve can be given: not C's private `__…`. A keyword is one too
/// (owner, 2026-10-10): a function so named is reached by path,
/// `my.size[…]`, and a const's name is quoted.
fn usable(name: &str) -> bool {
    !name.starts_with("__") && name.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// A name a type can be given: a type is written bare where no path
/// stands, so not a keyword.
fn usable_type(name: &str) -> bool {
    usable(name) && !crate::parser::is_keyword(name)
}

/// The type as C writes it, its type names kept: `resolve` follows them,
/// stopping at the ones Nerve keeps (`size_t`, `int32_t`).
fn qual(t: &Json) -> Option<&str> {
    t.str("qualType")
}

/// C's type names that come through as named, not as what they name.
const KEPT: [&str; 14] = ["size_t", "ssize_t", "ptrdiff_t", "intptr_t", "uintptr_t", "wchar_t", "int8_t", "int16_t", "int32_t", "int64_t", "uint8_t", "uint16_t", "uint32_t", "uint64_t"];

/// Whether a global is `const` itself: the last `const` after the last `*`,
/// or any `const` when there is no `*`.
fn top_const(t: &str) -> bool {
    let words = split(t);
    match words.iter().rposition(|w| w == "*") {
        Some(i) => words[i..].iter().any(|w| w == "const"),
        None => words.iter().any(|w| w == "const"),
    }
}

/// Qualifiers that change nothing Nerve says.
const IGNORED: [&str; 7] = ["restrict", "__restrict", "volatile", "_Nonnull", "_Nullable", "_Null_unspecified", "__unsafe_unretained"];

fn is_c_word(w: &str) -> bool {
    matches!(w, "void" | "char" | "short" | "int" | "long" | "signed" | "unsigned" | "float" | "double" | "_Bool" | "bool" | "const" | "*")
}

/// The words of a C type, `*`, `[`, `(` each its own.
fn split(t: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut word = String::new();
    for c in t.chars() {
        if c.is_ascii_alphanumeric() || c == '_' {
            word.push(c);
            continue;
        }
        if !word.is_empty() {
            out.push(std::mem::take(&mut word));
        }
        if matches!(c, '*' | '[' | '(') {
            out.push(c.to_string());
        } else if !c.is_whitespace() && !matches!(c, ']' | ')') {
            out.push("?".into());
        }
    }
    if !word.is_empty() {
        out.push(word);
    }
    out
}

/// C's arithmetic types: the machine-sized ones as C's `C.` types (owner,
/// 2026-10-10), what is the same size everywhere as Nerve's own.
fn base_type(words: &[&str]) -> Option<&'static str> {
    if let [one] = words {
        if let Some(t) = match *one {
            "size_t" => Some("C.size_t"),
            "ssize_t" => Some("C.ssize_t"),
            "ptrdiff_t" => Some("C.ptrdiff_t"),
            "intptr_t" => Some("C.intptr_t"),
            "uintptr_t" => Some("C.uintptr_t"),
            "wchar_t" => Some("C.wchar_t"),
            "int8_t" => Some("int8"),
            "int16_t" => Some("int16"),
            "int32_t" => Some("int32"),
            "int64_t" => Some("int64"),
            "uint8_t" => Some("uint8"),
            "uint16_t" => Some("uint16"),
            "uint32_t" => Some("uint32"),
            "uint64_t" => Some("uint64"),
            _ => None,
        } {
            return Some(t);
        }
    }
    let unsigned = words.contains(&"unsigned");
    let signed = words.contains(&"signed");
    let longs = words.iter().filter(|w| **w == "long").count();
    let rest: Vec<&str> = words.iter().copied().filter(|w| !matches!(*w, "unsigned" | "signed" | "long" | "int")).collect();
    let has_int = words.contains(&"int");
    let plain = !unsigned && !signed && !has_int;
    Some(match rest.as_slice() {
        ["void"] if plain && longs == 0 => "void",
        ["_Bool"] | ["bool"] if plain && longs == 0 => "bool",
        ["float"] if plain && longs == 0 => "bin32",
        ["double"] if plain && longs == 0 => "bin64",
        ["double"] if plain && longs == 1 => "C.long_double",
        ["char"] if longs == 0 && !has_int => {
            if signed {
                "C.signed_char"
            } else if unsigned {
                "C.unsigned_char"
            } else {
                "C.char"
            }
        }
        ["short"] if longs == 0 => {
            if unsigned {
                "C.unsigned_short"
            } else {
                "C.short"
            }
        }
        [] if longs == 0 => {
            if unsigned {
                "C.unsigned_int"
            } else {
                "C.int"
            }
        }
        [] if longs == 1 => {
            if unsigned {
                "C.unsigned_long"
            } else {
                "C.long"
            }
        }
        [] if longs == 2 => {
            if unsigned {
                "C.unsigned_long_long"
            } else {
                "C.long_long"
            }
        }
        _ => return None,
    })
}

/// A `#define`'s value as a whole number in decimal: in parentheses or not,
/// possibly negative, written in decimal, hex or octal, with C's `u`/`l`
/// suffixes.
fn whole_number(v: &str) -> Option<String> {
    let mut s = v.trim();
    while s.len() >= 2 && s.starts_with('(') && s.ends_with(')') {
        s = s[1..s.len() - 1].trim();
    }
    let neg = s.starts_with('-');
    if neg {
        s = s[1..].trim_start();
    }
    let s = s.trim_end_matches(['u', 'U', 'l', 'L']);
    let n = if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u128::from_str_radix(h, 16).ok()?
    } else if s.len() > 1 && s.starts_with('0') {
        u128::from_str_radix(&s[1..], 8).ok()?
    } else if !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) {
        s.parse().ok()?
    } else {
        return None;
    };
    Some(if neg && n != 0 { format!("-{}", n) } else { n.to_string() })
}

/// Just enough JSON for Clang's AST.
enum Json {
    Null,
    Bool,
    Num,
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(fields) => fields.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    fn str(&self, key: &str) -> Option<&str> {
        match self.get(key)? {
            Json::Str(s) => Some(s),
            _ => None,
        }
    }

    fn parse(text: &str) -> Option<Json> {
        let mut r = Reader { b: text.as_bytes(), at: 0 };
        r.value()
    }
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn space(&mut self) {
        while self.at < self.b.len() && self.b[self.at].is_ascii_whitespace() {
            self.at += 1;
        }
    }

    fn value(&mut self) -> Option<Json> {
        self.space();
        match *self.b.get(self.at)? {
            b'{' => {
                self.at += 1;
                let mut fields = Vec::new();
                loop {
                    self.space();
                    if self.b.get(self.at) == Some(&b'}') {
                        self.at += 1;
                        return Some(Json::Obj(fields));
                    }
                    let Json::Str(k) = self.value()? else { return None };
                    self.space();
                    if self.b.get(self.at) != Some(&b':') {
                        return None;
                    }
                    self.at += 1;
                    let v = self.value()?;
                    fields.push((k, v));
                    self.space();
                    if self.b.get(self.at) == Some(&b',') {
                        self.at += 1;
                    }
                }
            }
            b'[' => {
                self.at += 1;
                let mut items = Vec::new();
                loop {
                    self.space();
                    if self.b.get(self.at) == Some(&b']') {
                        self.at += 1;
                        return Some(Json::Arr(items));
                    }
                    items.push(self.value()?);
                    self.space();
                    if self.b.get(self.at) == Some(&b',') {
                        self.at += 1;
                    }
                }
            }
            b'"' => {
                self.at += 1;
                let mut out = Vec::new();
                loop {
                    let c = *self.b.get(self.at)?;
                    self.at += 1;
                    match c {
                        b'"' => return String::from_utf8(out).ok().map(Json::Str),
                        b'\\' => {
                            let e = *self.b.get(self.at)?;
                            self.at += 1;
                            match e {
                                b'n' => out.push(b'\n'),
                                b't' => out.push(b'\t'),
                                b'r' => out.push(b'\r'),
                                b'b' => out.push(8),
                                b'f' => out.push(12),
                                b'u' => {
                                    let hex = std::str::from_utf8(self.b.get(self.at..self.at + 4)?).ok()?;
                                    self.at += 4;
                                    let c = char::from_u32(u32::from_str_radix(hex, 16).ok()?).unwrap_or('?');
                                    out.extend(c.to_string().bytes());
                                }
                                other => out.push(other),
                            }
                        }
                        _ => out.push(c),
                    }
                }
            }
            b't' | b'f' => {
                while self.at < self.b.len() && self.b[self.at].is_ascii_alphabetic() {
                    self.at += 1;
                }
                Some(Json::Bool)
            }
            b'n' => {
                self.at += 4;
                Some(Json::Null)
            }
            _ => {
                while self.at < self.b.len() && matches!(self.b[self.at], b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9') {
                    self.at += 1;
                }
                Some(Json::Num)
            }
        }
    }
}
