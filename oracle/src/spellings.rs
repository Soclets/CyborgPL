//! `spellings.cyr` (owner, 2026-10-02, and 2026-10-13: "Yeah, if it's correct"): one table of the language's
//! words for every tool. The keywords, each old spelling and what to write instead, the near-misses that are
//! not operators, and the operators free to declare with the hint shown when one is used undeclared. The
//! oracle keeps no lists of its own: the lexer, the parser and the interpreter ask here. Design claude keeps
//! the file, in the same commit as each ruling; `oracle --check-spellings` checks it against itself.

use crate::ast::{Bracket, Expr, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

/// An old spelling: as written, what replaced it, the day, and where it counts
#[derive(Debug)]
pub struct Old {
    pub written: String,
    pub write: Vec<String>,
    pub note: Option<String>,
    pub since: Option<String>,
    /// "anywhere", "after inline.", "outside a path", or "special" (found by the code that names it)
    pub place: String,
}

pub struct Call { pub name: String, pub words: Vec<String>, pub example: String }

pub struct NotOperator { pub written: String, pub says: String, pub since: Option<String> }

pub struct Table {
    pub keywords: Vec<String>,
    /// words taken out of the language and kept reserved, refused by the code with a message of its own
    pub retired: Vec<String>,
    pub capitals: Vec<String>,
    pub old: Vec<Old>,
    pub not_operators: Vec<NotOperator>,
    pub free_ops: Vec<(String, String)>,
    /// the word each argument says, by call (owner, 2026-10-14)
    pub calls: Vec<Call>,
}

const SOURCE: &str = include_str!("../../spellings.cyr");
static TABLE: OnceLock<Table> = OnceLock::new();
/// set while the table itself is read, so the lexer does not ask for it then
static LOADING: AtomicBool = AtomicBool::new(false);

pub fn loading() -> bool { LOADING.load(Ordering::SeqCst) }

pub fn table() -> &'static Table {
    TABLE.get_or_init(|| read(SOURCE).unwrap_or_else(|e| panic!("spellings.cyr: {e}")))
}

fn read(src: &str) -> Result<Table, String> {
    let was = LOADING.swap(true, Ordering::SeqCst);
    let t = read_inner(src);
    LOADING.store(was, Ordering::SeqCst);
    t
}

fn read_inner(src: &str) -> Result<Table, String> {
    let toks = crate::lexer::lex(src).map_err(|e| format!("line {}:{}: {}", e.line, e.col, e.msg))?;
    let (shape, fields, _) = crate::parser::Parser::new(toks).one_value().map_err(|e| format!("line {}: {}", e.line, e.msg))?;
    if shape != "::spellings.table" { return Err(format!("the file is one `spellings::table[ … ]`, found `{shape}`")); }
    let mut t = Table { keywords: Vec::new(), retired: Vec::new(), capitals: Vec::new(), old: Vec::new(), not_operators: Vec::new(), free_ops: Vec::new(), calls: Vec::new() };
    for (name, b) in &fields {
        match name.as_str() {
            "keywords" => t.keywords = texts(b)?,
            "retired" => t.retired = texts(b)?,
            "capitals" => t.capitals = texts(b)?,
            "old" => for (shape, f, line) in constructs(b)? {
                if shape != "::spellings.old" { return Err(format!("line {line}: `old` holds `spellings::old[ … ]`")); }
                t.old.push(Old {
                    written: need(&f, "written", line)?,
                    write: f.iter().find(|(n, _)| n == "write").map(|(_, b)| texts(b)).transpose()?.ok_or(format!("line {line}: an old spelling says what to `write`"))?,
                    note: opt(&f, "note")?, since: opt(&f, "since")?,
                    place: opt(&f, "where")?.unwrap_or_else(|| "anywhere".into()),
                });
            },
            "not_operators" => for (shape, f, line) in constructs(b)? {
                if shape != "::spellings.not_operator" { return Err(format!("line {line}: `not_operators` holds `spellings::not_operator[ … ]`")); }
                t.not_operators.push(NotOperator { written: need(&f, "written", line)?, says: need(&f, "says", line)?, since: opt(&f, "since")? });
            },
            "calls" => for (shape, f, line) in constructs(b)? {
                if shape != "::spellings.call" { return Err(format!("line {line}: `calls` holds `spellings::call[ … ]`")); }
                let words = f.iter().find(|(n, _)| n == "words").map(|(_, b)| texts(b)).transpose()?.ok_or(format!("line {line}: a call lists its `words`"))?;
                t.calls.push(Call { name: need(&f, "name", line)?, words, example: need(&f, "example", line)? });
            },
            "free_ops" => for (shape, f, line) in constructs(b)? {
                if shape != "::spellings.free_op" { return Err(format!("line {line}: `free_ops` holds `spellings::free_op[ … ]`")); }
                t.free_ops.push((need(&f, "op", line)?, need(&f, "hint", line)?));
            },
            other => return Err(format!("a `spellings::table` has no `{other}`")),
        }
    }
    Ok(t)
}

fn flat(b: &Bracket) -> Vec<Expr> {
    fn each(e: &Expr, out: &mut Vec<Expr>) { match e { Expr::Nested(b) => for v in &b.values { let Value::Expr(x) = v; each(x, out) }, o => out.push(o.clone()) } }
    let mut v = Vec::new();
    for x in &b.values { let Value::Expr(x) = x; each(x, &mut v); }
    v
}
/// A list, as the language writes one: `[a, b]`, or `[a,]` for one item; `[a]` is the value itself (2026-09-14)
fn list(b: &Bracket) -> Result<Vec<Expr>, String> {
    match b.values.as_slice() {
        [Value::Expr(Expr::Nested(inner))] if inner.values.len() != 1 || inner.one_list => Ok(flat(inner)),
        [Value::Expr(Expr::Nested(inner))] => Err(format!("line {}: a list of one is written `[x,]`; `[x]` is the value itself (2026-09-14)", inner.line)),
        _ => Err(format!("line {}: a list was expected, `[ … ]`", b.line)),
    }
}
fn texts(b: &Bracket) -> Result<Vec<String>, String> {
    list(b)?.into_iter().map(|e| match e { Expr::Text(t, _) => Ok(t), _ => Err("a list of texts was expected".to_string()) }).collect()
}
fn constructs(b: &Bracket) -> Result<Vec<(String, Vec<(String, Bracket)>, usize)>, String> {
    list(b)?.into_iter().map(|e| match e { Expr::Construct(s, f, l) => Ok((s, f, l)), _ => Err("a list of entries was expected".to_string()) }).collect()
}
fn one_text(b: &Bracket) -> Result<String, String> {
    match flat(b).as_slice() { [Expr::Text(t, _)] if !matches!(b.values.as_slice(), [Value::Expr(Expr::Nested(i))] if i.one_list) => Ok(t.clone()), _ => Err("one text was expected".to_string()) }
}
fn opt(f: &[(String, Bracket)], name: &str) -> Result<Option<String>, String> {
    match f.iter().find(|(n, _)| n == name) {
        None => Ok(None),
        Some((_, b)) => one_text(b).map(Some).map_err(|_| format!("`{name}` is one text")),
    }
}
fn need(f: &[(String, Bracket)], name: &str, line: usize) -> Result<String, String> {
    opt(f, name)?.ok_or(format!("line {line}: the entry needs its `{name}`"))
}

/// A word that cannot name a function, a type, a module or a collection: a keyword or a retired word (an old
/// spelling in code is refused before names are read)
pub fn is_reserved(w: &str) -> bool {
    if loading() { return false; }
    let t = table();
    t.keywords.iter().any(|k| k == w) || t.retired.iter().any(|k| k == w)
}

/// The old spelling `written`, found where it stands: after `inline.`, inside a path or not
pub fn old_at(written: &str, after_inline: bool, in_path: bool) -> Option<&'static Old> {
    if loading() { return None; }
    table().old.iter().find(|o| o.written == written && match o.place.as_str() {
        "after inline." => after_inline,
        "outside a path" => !in_path && !after_inline,
        "anywhere" => !after_inline,
        _ => false,
    })
}

/// An old spelling the code finds itself (`place = "special"`), named by how it is written
pub fn old(written: &str) -> &'static Old {
    table().old.iter().find(|o| o.written == written).unwrap_or_else(|| panic!("spellings.cyr has no old spelling `{written}`"))
}

/// "`&` is an old spelling: write `lend` / `lent`; a bitwise and is `and.bit` (owner, 2026-10-07; …)"
pub fn message(o: &Old) -> String {
    let shown = if o.place == "after inline." { format!("inline.{}", o.written) } else { o.written.clone() };
    let mut m = format!("`{shown}` is an old spelling: write `{}`", o.write.join("` / `"));
    if let Some(n) = &o.note { m.push_str("; "); m.push_str(n); }
    match o.since.as_deref() {
        Some(d) if d == "2026-10-07" => m.push_str(&format!(" (owner, {d}; MTM, which kept the old spellings, is gone)")),
        Some(d) => m.push_str(&format!(" (owner, {d})")),
        None => {}
    }
    m
}

/// "`<=` is not an operator; less-or-equal is `at_most` (owner, 2026-10-07)"
pub fn not_operator(written: &str) -> Option<String> {
    let n = table().not_operators.iter().find(|n| n.written == written)?;
    Some(match &n.since { Some(d) => format!("`{written}` is not an operator; {} (owner, {d})", n.says), None => format!("`{written}` is not an operator; {}", n.says) })
}

pub fn free_op_hint(op: &str) -> Option<&'static str> {
    table().free_ops.iter().find(|(o, _)| o == op).map(|(_, h)| h.as_str())
}

/// The table checked against itself and against the naming rule (owner, 2026-10-13): every problem found
pub fn check() -> Vec<String> {
    let t = match read(SOURCE) { Ok(t) => t, Err(e) => return vec![e] };
    let mut bad = Vec::new();
    let lower = |w: &str| w.chars().next().map_or(false, |c| c.is_ascii_lowercase()) && w.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    for (i, k) in t.keywords.iter().enumerate() {
        if !lower(k) && !t.capitals.contains(k) { bad.push(format!("keyword `{k}` breaks the naming rule: lowercase, words joined by `_` (capitals only for {})", t.capitals.join(", "))); }
        if t.keywords[..i].contains(k) { bad.push(format!("keyword `{k}` is listed twice")); }
        if t.retired.contains(k) { bad.push(format!("`{k}` is both a keyword and retired")); }
    }
    let olds: Vec<&str> = t.old.iter().map(|o| o.written.as_str()).collect();
    for (i, o) in t.old.iter().enumerate() {
        if t.old[..i].iter().any(|p| p.written == o.written && p.place == o.place) { bad.push(format!("old spelling `{}` is listed twice", o.written)); }
        if o.place != "after inline." && t.keywords.contains(&o.written) { bad.push(format!("`{}` is both a keyword and an old spelling", o.written)); }
        for w in &o.write { if olds.contains(&w.as_str()) { bad.push(format!("`{}` says to write `{w}`, itself an old spelling", o.written)); } }
        if !["anywhere", "after inline.", "outside a path", "special"].contains(&o.place.as_str()) { bad.push(format!("`{}` has an unknown `where`: `{}`", o.written, o.place)); }
        if let Some(d) = &o.since { if !is_date(d) { bad.push(format!("`{}`'s `since` is not a date: `{d}`", o.written)); } }
    }
    let quoted = |s: &str| -> Vec<String> { s.split('`').skip(1).step_by(2).map(|x| x.to_string()).collect() };
    let mut hints: Vec<(String, String)> = Vec::new();
    for o in &t.old { if let Some(n) = &o.note { hints.push((o.written.clone(), n.clone())); } }
    for n in &t.not_operators { hints.push((n.written.clone(), n.says.clone())); }
    for (op, h) in &t.free_ops { hints.push((op.clone(), h.clone())); }
    for (who, h) in &hints {
        for q in quoted(h) { if olds.contains(&q.as_str()) { bad.push(format!("the hint for `{who}` names `{q}`, an old spelling")); } }
    }
    for (i, c) in t.calls.iter().enumerate() {
        if t.calls[..i].iter().any(|p| p.name == c.name) { bad.push(format!("call `{}` is listed twice", c.name)); }
        if !c.example.starts_with(&format!("{}(", c.name)) { bad.push(format!("call `{}`'s example does not show it: `{}`", c.name, c.example)); }
        for w in c.words.iter().filter(|w| !w.is_empty()) {
            // the word with its colon (owner, 2026-10-20)
            if !c.example.contains(&format!(" {w}: ")) { bad.push(format!("call `{}`'s example does not say `{w}:`", c.name)); }
            if !lower(w) { bad.push(format!("call `{}`'s word `{w}` breaks the naming rule", c.name)); }
        }
    }
    for n in &t.not_operators { if let Some(d) = &n.since { if !is_date(d) { bad.push(format!("`{}`'s `since` is not a date: `{d}`", n.written)); } } }
    bad
}

fn is_date(d: &str) -> bool {
    let b = d.as_bytes();
    b.len() == 10 && b[4] == b'-' && b[7] == b'-' && d.chars().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}
