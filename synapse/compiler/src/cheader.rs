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
    out.from_ast(&tree);
    for line in defines.lines().filter(|l| !own.contains(*l)) {
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
}

impl Decls {
    fn push(&mut self, name: &str, text: String) {
        if self.seen.insert(name.to_string()) {
            self.list.push(Decl { name: name.to_string(), text });
        }
    }

    fn from_ast(&mut self, tree: &Json) {
        let Some(Json::Arr(top)) = tree.get("inner") else { return };
        for d in top {
            let kind = d.str("kind").unwrap_or("");
            let Some(name) = d.str("name") else { continue };
            if kind == "TypedefDecl" {
                if let Some(t) = d.get("type").and_then(qual) {
                    self.typedefs.insert(name.to_string(), t.to_string());
                }
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
        if whole.contains("(*") || whole.contains("(^") {
            return None;
        }
        let ret = whole[..whole.find('(')?].trim();
        let ret = if self.resolve(ret).trim() == "void" { None } else { Some(self.ty(ret)?) };
        let mut params = Vec::new();
        if let Some(Json::Arr(inner)) = d.get("inner") {
            for p in inner.iter().filter(|p| p.str("kind") == Some("ParmVarDecl")) {
                let t = p.get("type").and_then(qual)?;
                params.push(format!("'p{}': {}", params.len() + 1, self.ty(t)?));
            }
        }
        if d.get("variadic").is_some() {
            params.push(format!("...'p{}'", params.len() + 1));
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
            // Clang's own names for C's `size_t` family, which no header
            // declares (Clang 22's predefined sugar).
            let predefined = match w.as_str() {
                "__size_t" => Some("unsigned long".to_string()),
                "__signed_size_t" | "__ptrdiff_t" => Some("long".to_string()),
                _ => None,
            };
            match self.typedefs.get(&w).or(predefined.as_ref()) {
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
        let words: Vec<String> = split(&self.resolve(t)).into_iter().filter(|w| !IGNORED.contains(&w.as_str())).collect();
        if words.iter().any(|w| w == "[" || w == "(" || w == "struct" || w == "union" || w == "enum" || w == "?") {
            return None;
        }
        // Any `const` after a `*` makes a pointer that is itself fixed.
        let stars = words.iter().filter(|w| *w == "*").count();
        if let Some(first) = words.iter().position(|w| w == "*") {
            if words[first..].iter().any(|w| w == "const") {
                return None;
            }
        }
        let base: Vec<&str> = words.iter().map(String::as_str).filter(|w| *w != "*").collect();
        let pointee_const = base.contains(&"const");
        let base: Vec<&str> = base.into_iter().filter(|w| *w != "const").collect();
        let mut out = match (base_type(&base)?, stars) {
            ("void", 0) => return None,
            ("void", _) => "uint8".to_string(),
            (b, _) => b.to_string(),
        };
        for i in 0..stars {
            out = if i == 0 && pointee_const { format!("ptr[{}]", out) } else { format!("ptr.mut[{}]", out) };
        }
        Some(out)
    }

    fn define(&mut self, line: &str) {
        let Some(rest) = line.strip_prefix("#define ") else { return };
        let (name, value) = match rest.find(|c: char| c == ' ' || c == '(') {
            Some(i) if rest.as_bytes()[i] == b'(' => return,
            Some(i) => (&rest[..i], rest[i + 1..].trim()),
            None => return,
        };
        if !usable(name) || self.seen.contains(name) {
            return;
        }
        if let Some(n) = whole_number(value) {
            self.push(name, format!("const '{}' := {};", name, n));
        } else if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') && !value[1..value.len() - 1].contains(['"', '\\']) {
            self.push(name, format!("const '{}' := {};", name, value));
        }
    }
}

/// A name Nerve can be given: not C's private `__…`, not a keyword.
fn usable(name: &str) -> bool {
    !name.starts_with("__") && !crate::parser::is_keyword(name) && name.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The type as written once its type names are gone.
fn qual(t: &Json) -> Option<&str> {
    t.str("desugaredQualType").or_else(|| t.str("qualType"))
}

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

/// C's arithmetic types, `long` being 64 bits as on the machines Synapse
/// builds for.
fn base_type(words: &[&str]) -> Option<&'static str> {
    let unsigned = words.contains(&"unsigned");
    let signed = words.contains(&"signed");
    let longs = words.iter().filter(|w| **w == "long").count();
    let rest: Vec<&str> = words.iter().copied().filter(|w| !matches!(*w, "unsigned" | "signed" | "long" | "int")).collect();
    let has_int = words.contains(&"int");
    Some(match rest.as_slice() {
        ["void"] if !unsigned && !signed && longs == 0 && !has_int => "void",
        ["_Bool"] | ["bool"] if !unsigned && !signed && longs == 0 && !has_int => "bool",
        ["float"] if !unsigned && !signed && longs == 0 && !has_int => "bin32",
        ["double"] if !unsigned && !signed && longs == 0 && !has_int => "bin64",
        ["char"] if longs == 0 && !has_int => {
            if signed {
                "int8"
            } else {
                "uint8"
            }
        }
        ["short"] if longs == 0 => {
            if unsigned {
                "uint16"
            } else {
                "int16"
            }
        }
        [] if longs == 0 => {
            if unsigned {
                "uint32"
            } else {
                "int32"
            }
        }
        [] if longs <= 2 => {
            if unsigned {
                "uint64"
            } else {
                "int64"
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
