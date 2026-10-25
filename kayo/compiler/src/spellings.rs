//! `spellings.cyr`, at the repository's top (owner, 2026-10-13: "Yes"): the
//! one table of the language's keywords, its old spellings and the words
//! to write instead, what is not an operator, and the hint for each glyph
//! free to declare as one. Synapse reads it, built in, rather than lists of
//! its own; design claude keeps it, and the oracle reads the same file.

use std::sync::OnceLock;

use crate::ast::{Arg, Expr, ExprKind};

/// The table as Synapse reads it.
pub struct Spellings {
    pub keywords: Vec<String>,
    pub retired: Vec<String>,
    pub capitals: Vec<String>,
    pub old: Vec<Old>,
    pub not_operators: Vec<(String, String)>,
    pub free_ops: Vec<(String, String)>,
    pub calls: Vec<Call>,
}

/// A call whose arguments after the first say their roles (owner,
/// 2026-10-14): one word per position, "" where none is asked (the first,
/// or one with syntax of its own, as SIMD's `pos`), and the example the
/// refusal quotes.
pub struct Call {
    pub name: String,
    pub words: Vec<String>,
    pub example: String,
}

/// One old spelling: what was written, what to write, a note after it, and
/// where it applies (`special` ones are found by code, each its own way).
pub struct Old {
    pub written: String,
    pub write: Vec<String>,
    pub note: Option<String>,
    pub since: Option<String>,
    pub where_: Option<String>,
}

impl Old {
    /// The sentence with the ruling's date after it, as the oracle says it:
    /// "`x` is an old spelling: write `times` (owner, 2026-10-03)".
    pub fn dated(&self) -> String {
        match &self.since {
            Some(d) => format!("{} (owner, {})", self.sentence(), d),
            None => self.sentence(),
        }
    }

    /// "`x` is an old spelling: write `y`" (or "`y` / `z`"), the note after.
    pub fn sentence(&self) -> String {
        let write = self.write.iter().map(|w| format!("`{}`", w)).collect::<Vec<_>>().join(" / ");
        match &self.note {
            Some(n) => format!("`{}` is an old spelling: write {}; {}", self.written, write, n),
            None => format!("`{}` is an old spelling: write {}", self.written, write),
        }
    }
}

const TABLE: &str = include_str!("../../../spellings.cyr");

static ONCE: OnceLock<Spellings> = OnceLock::new();

thread_local! {
    /// Reading the table runs the parser, which asks what a keyword is:
    /// while it reads, the answer is the parser's own, not the table's.
    static READING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

pub fn get() -> &'static Spellings {
    ONCE.get_or_init(|| {
        READING.with(|r| r.set(true));
        let t = read(TABLE);
        READING.with(|r| r.set(false));
        t.unwrap_or_else(|why| panic!("spellings.cyr could not be read: {}", why))
    })
}

fn reading() -> bool {
    READING.with(|r| r.get())
}

fn read(text: &str) -> Result<Spellings, String> {
    let toks = crate::lexer::lex(crate::source::FileId(0), text).map_err(|d| d.sentence.clone())?;
    let value = crate::parser::parse_value(&toks).map_err(|d| d.sentence.clone())?;
    let fields = named(&value, "table")?;
    let mut out = Spellings { keywords: Vec::new(), retired: Vec::new(), capitals: Vec::new(), old: Vec::new(), not_operators: Vec::new(), free_ops: Vec::new(), calls: Vec::new() };
    for (name, v) in fields {
        match name.as_str() {
            "keywords" => out.keywords = texts(v)?,
            "retired" => out.retired = texts(v)?,
            "capitals" => out.capitals = texts(v)?,
            "old" => {
                for e in list(v)? {
                    let f = named(e, "old")?;
                    out.old.push(Old {
                        written: text_of(field(&f, "written")?)?,
                        write: texts(field(&f, "write")?)?,
                        note: f.iter().find(|(n, _)| n == "note").map(|(_, v)| text_of(v)).transpose()?,
                        since: f.iter().find(|(n, _)| n == "since").map(|(_, v)| text_of(v)).transpose()?,
                        where_: f.iter().find(|(n, _)| n == "where").map(|(_, v)| text_of(v)).transpose()?,
                    });
                }
            }
            "not_operators" => {
                for e in list(v)? {
                    let f = named(e, "not_operator")?;
                    out.not_operators.push((text_of(field(&f, "written")?)?, text_of(field(&f, "says")?)?));
                }
            }
            "calls" => {
                for e in list(v)? {
                    let f = named(e, "call")?;
                    out.calls.push(Call { name: text_of(field(&f, "name")?)?, words: texts(field(&f, "words")?)?, example: text_of(field(&f, "example")?)? });
                }
            }
            "free_ops" => {
                for e in list(v)? {
                    let f = named(e, "free_op")?;
                    out.free_ops.push((text_of(field(&f, "op")?)?, text_of(field(&f, "hint")?)?));
                }
            }
            _ => {}
        }
    }
    Ok(out)
}

/// `spellings::name[a = …, b = …]`: its fields.
fn named<'a>(e: &'a Expr, name: &str) -> Result<Vec<(String, &'a Expr)>, String> {
    match &e.kind {
        ExprKind::Call { path, args, .. } if path.last().name == name => args
            .iter()
            .map(|a| match a {
                Arg::Named(n, v) => Ok((n.name.clone(), v)),
                _ => Err(format!("`spellings::{}` names its fields", name)),
            })
            .collect(),
        _ => Err(format!("expected a `spellings::{}[…]`", name)),
    }
}

fn field<'a>(f: &[(String, &'a Expr)], name: &str) -> Result<&'a Expr, String> {
    f.iter().find(|(n, _)| n == name).map(|(_, v)| *v).ok_or_else(|| format!("no field `{}`", name))
}

/// A list's items. A list of one is written `[x,]`: `[x]` is the value
/// itself (2026-09-14), refused here as the oracle refuses it.
fn list(e: &Expr) -> Result<&[Expr], String> {
    match &e.kind {
        ExprKind::List(items) => Ok(items),
        ExprKind::Paren(..) => Err("a list of one is written `[x,]`; `[x]` is the value itself (2026-09-14)".into()),
        _ => Err("expected a list".into()),
    }
}

fn text_of(e: &Expr) -> Result<String, String> {
    match &e.kind {
        ExprKind::Str(s) => Ok(s.clone()),
        ExprKind::Newline => Ok("\n".into()),
        _ => Err("expected a text".into()),
    }
}

fn texts(e: &Expr) -> Result<Vec<String>, String> {
    list(e)?.iter().map(text_of).collect()
}

/// A keyword of the language, as the table lists it.
pub fn is_keyword(w: &str) -> bool {
    !reading() && get().keywords.iter().any(|k| k == w)
}

/// An old spelling the table finds by itself, written where `where_` says
/// (no `where`: anywhere it stands as a token).
pub fn old_word(w: &str, where_: Option<&str>) -> Option<&'static Old> {
    if reading() {
        return None;
    }
    get().old.iter().find(|o| o.written == w && o.where_.as_deref() == where_)
}

/// The sentence for a glyph the language does not have as an operator.
pub fn not_operator(written: &str) -> Option<&'static str> {
    if reading() {
        return None;
    }
    get().not_operators.iter().find(|(w, _)| w == written).map(|(_, s)| s.as_str())
}

/// The hint for a glyph free to declare, when nothing declared it here.
pub fn free_op_hint(op: &str) -> Option<&'static str> {
    if reading() {
        return None;
    }
    get().free_ops.iter().find(|(o, _)| o == op).map(|(_, h)| h.as_str())
}

/// Whether the table lists `word.` as an old spelling of its own (`limb.`).
pub fn old_dotted(word: &str) -> bool {
    !reading() && get().old.iter().any(|o| o.written.strip_suffix('.') == Some(word))
}

/// The calls whose arguments say their roles. None while the table is
/// being read, since reading it runs the parser.
pub fn calls() -> &'static [Call] {
    if reading() {
        return &[];
    }
    &get().calls
}

/// The 2026-10-03 wrapping symbols, old spellings of the words (`x~` is
/// `times.wrap`), found by the parser where an operator stands.
pub fn old_symbol(s: &str) -> Option<&'static Old> {
    if reading() || !matches!(s, "+~" | "-~" | "x~" | "xx~") {
        return None;
    }
    get().old.iter().find(|o| o.written == s && o.where_.as_deref() == Some("special"))
}
