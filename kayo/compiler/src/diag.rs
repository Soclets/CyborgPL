//! Diagnostics in the shape `design/diagnostics.md` gives: labelled plain
//! text — `error` / `check` / `note`, then `where`, `code`, `when`, `why`,
//! `try`, `fix` — one fact per line, nothing depending on alignment.

use crate::source::{SourceMap, Span};
use std::fmt::Write as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Will not build.
    Error,
    /// A check site: unproven, with its counterexample.
    Check,
    /// The report: a fact worth saying.
    Note,
}

impl Kind {
    fn label(self) -> &'static str {
        match self {
            Kind::Error => "error",
            Kind::Check => "check",
            Kind::Note => "note",
        }
    }
}

/// One suggested edit: the whole line as it should read.
#[derive(Debug, Clone)]
pub struct Fix {
    pub line: Option<u32>,
    pub text: String,
    /// Made from the source: the line holding the span, edited (owner,
    /// 2026-10-22: "the fix as edited code", the whole line).
    pub edit: Option<(Span, Edit)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    /// The span's text becomes `text`.
    Replace,
    /// `text` goes in front of the statement the span is in.
    Before,
    /// The word `text` (and the space after it) comes out of the line.
    Remove,
    /// The span's text goes inside `text(…)`.
    Wrap,
    /// The first of these words in the line becomes `text`; no fix when
    /// none is there.
    Swap(Vec<String>),
}

impl Fix {
    /// The edited line and its number, for an edit made from the source.
    fn edited(&self, sources: &SourceMap) -> Option<(u32, String)> {
        let (span, how) = self.edit.clone()?;
        let file = sources.get(span.file);
        let (line, _) = file.line_col(span.start);
        let text = file.line_text(line);
        let start = span.start as usize - (file.text[..span.start as usize].rfind('\n').map_or(0, |i| i + 1));
        let edited = match how {
            Edit::Replace => {
                let end = (start + (span.end - span.start) as usize).min(text.len());
                format!("{}{}{}", &text[..start], self.text, &text[end..])
            }
            Edit::Wrap => {
                let end = (start + (span.end - span.start) as usize).min(text.len());
                format!("{}{}({}){}", &text[..start], self.text, &text[start..end], &text[end..])
            }
            Edit::Swap(from) => {
                // A whole word: `give $s` is not in `give $sx`.
                let whole = |at: usize, len: usize| !text[at + len..].starts_with(|c: char| c.is_alphanumeric() || c == '_');
                let (at, len) = from.iter().find_map(|w| text.match_indices(w.as_str()).map(|(at, _)| at).find(|&at| whole(at, w.len())).map(|at| (at, w.len())))?;
                format!("{}{}{}", &text[..at], self.text, &text[at + len..])
            }
            Edit::Before => {
                let indent = text.len() - text.trim_start().len();
                format!("{}{}{}", &text[..indent], self.text, &text[indent..])
            }
            Edit::Remove => {
                let at = text.find(&self.text)?;
                let mut end = at + self.text.len();
                if text[end..].starts_with(' ') {
                    end += 1;
                }
                format!("{}{}", &text[..at], &text[end..])
            }
        };
        Some((line, edited.trim().to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub kind: Kind,
    /// Its name, the title of the message (owner, 2026-10-22: one fixed name
    /// per kind of error, from `errors.cyr`, which the oracle reads too).
    pub name: Option<&'static str>,
    /// The sentence: what was found, why it matters, what to write instead.
    /// The start of the error's docs (owner: "learning could be in docs").
    pub sentence: String,
    pub span: Option<Span>,
    /// The human-readable place: "in `pick`, called from MAIN at line 48".
    pub place: Option<String>,
    /// The counterexample, as a sentence.
    pub when: Option<String>,
    /// The root of the chain, one fact per line, each with a place when it has one.
    pub why: Vec<(String, Option<Span>)>,
    /// A command that reproduces it.
    pub try_: Option<String>,
    /// The edited line(s); alternatives after the first go under `or`.
    pub fixes: Vec<Fix>,
}

/// A message's code, between backticks, shows a plain quoted name as
/// `$name` (owner, 2026-10-22: good practice writes `$`, quoting only a
/// name that needs it); the prose around it keeps the quotes. A message
/// about the quotes themselves is left as written.
fn dollar_names(sentence: String) -> String {
    if sentence.contains("quote") || !sentence.contains('`') {
        return sentence;
    }
    let mut out = String::with_capacity(sentence.len());
    for (k, part) in sentence.split('`').enumerate() {
        if k > 0 {
            out.push('`');
        }
        if k % 2 == 0 {
            out.push_str(part);
            continue;
        }
        let mut rest = part;
        while let Some(open) = rest.find('\'') {
            let after = &rest[open + 1..];
            let name_len = after.find(|c: char| !(c.is_alphanumeric() || c == '_')).unwrap_or(after.len());
            // `<'T'>` and `.'x'` are about the quotes themselves.
            let before = rest[..open].chars().last();
            if name_len > 0 && after[name_len..].starts_with('\'') && !matches!(before, Some('<' | '.')) {
                out.push_str(&rest[..open]);
                out.push('$');
                out.push_str(&after[..name_len]);
                rest = &after[name_len + 1..];
            } else {
                out.push_str(&rest[..open + 1]);
                rest = after;
            }
        }
        out.push_str(rest);
    }
    out
}

impl Diagnostic {
    pub fn error(sentence: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Kind::Error, sentence)
    }
    pub fn check(sentence: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Kind::Check, sentence)
    }
    pub fn note(sentence: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Kind::Note, sentence)
    }
    pub fn new(kind: Kind, sentence: impl Into<String>) -> Diagnostic {
        Diagnostic { kind, name: None, sentence: dollar_names(sentence.into()), span: None, place: None, when: None, why: Vec::new(), try_: None, fixes: Vec::new() }
    }
    /// Its name from `errors.cyr`.
    pub fn named(mut self, name: &'static str) -> Diagnostic {
        self.name = Some(name);
        self
    }
    pub fn at(mut self, span: Span) -> Diagnostic {
        self.span = Some(span);
        self
    }
    pub fn place(mut self, place: impl Into<String>) -> Diagnostic {
        self.place = Some(place.into());
        self
    }
    pub fn when(mut self, when: impl Into<String>) -> Diagnostic {
        self.when = Some(when.into());
        self
    }
    pub fn why(mut self, why: impl Into<String>) -> Diagnostic {
        self.why.push((why.into(), None));
        self
    }
    /// A fact with a place: rendered with "— line N" after it.
    pub fn why_at(mut self, span: Span, why: impl Into<String>) -> Diagnostic {
        self.why.push((why.into(), Some(span)));
        self
    }
    pub fn try_(mut self, cmd: impl Into<String>) -> Diagnostic {
        self.try_ = Some(cmd.into());
        self
    }
    pub fn fix(mut self, line: Option<u32>, text: impl Into<String>) -> Diagnostic {
        self.fixes.push(Fix { line, text: text.into(), edit: None });
        self
    }
    /// A fix made from the source: the line holding `span`, edited.
    pub fn fix_edit(mut self, span: Span, how: Edit, text: impl Into<String>) -> Diagnostic {
        self.fixes.push(Fix { line: None, text: text.into(), edit: Some((span, how)) });
        self
    }

    /// The plain-text form. This *is* the message; anything else a terminal
    /// draws is decoration.
    pub fn render(&self, sources: &SourceMap) -> String {
        // Errors and checks in the full form (owner, 2026-10-22); a note,
        // which only the build report shows, keeps its sentence.
        if self.kind != Kind::Note {
            return self.render_full(sources);
        }
        let mut out = String::new();
        // A concise file's line and column mean nothing (design claude's
        // reading): the decoded line, named and shown.
        let concise = self.span.filter(|s| sources.get(s.file).concise).map(|s| {
            let f = sources.get(s.file);
            let (line, _) = f.line_col(s.start);
            format!(" (concise file; decoded line {}: `{}`)", line, f.line_text_at(s.start).trim())
        });
        let _ = writeln!(out, "{}: {}{}", self.kind.label(), self.sentence, concise.unwrap_or_default());
        if let Some(span) = self.span {
            let file = sources.get(span.file);
            let (line, col) = file.line_col(span.start);
            let mut where_ = format!("where: {}, line {}, column {}", file.display_name(), line, col);
            if let Some(place) = &self.place {
                let _ = write!(where_, " — {}", place);
            }
            let _ = writeln!(out, "{}", where_);
            let code = file.slice(span).trim();
            let code = if code.contains('\n') { code.lines().next().unwrap_or("").trim_end().to_string() + " …" } else { code.to_string() };
            if !code.is_empty() {
                let _ = writeln!(out, "code:  {}", code);
            }
        } else if let Some(place) = &self.place {
            let _ = writeln!(out, "where: {}", place);
        }
        if let Some(when) = &self.when {
            let _ = writeln!(out, "when:  {}", when);
        }
        if !self.why.is_empty() {
            out.push('\n');
            for (i, (w, at)) in self.why.iter().enumerate() {
                let w = match at {
                    Some(sp) => {
                        format!("{} — {}", w, sources.get(sp.file).line_of(sp.start))
                    }
                    None => w.clone(),
                };
                let w = &w;
                if i == 0 {
                    let _ = writeln!(out, "why:   {}", w);
                } else {
                    let _ = writeln!(out, "       {}", w);
                }
            }
        }
        if let Some(t) = &self.try_ {
            out.push('\n');
            let _ = writeln!(out, "try:   {}", t);
        }
        if !self.fixes.is_empty() {
            out.push('\n');
            for (i, f) in self.fixes.iter().enumerate() {
                let label = if i == 0 { "fix:  " } else { "   or " };
                match f.line {
                    Some(l) => {
                        let file = self.span.map(|s| format!(" of {}", sources.get(s.file).short_name())).unwrap_or_default();
                        let _ = writeln!(out, "{} line {}{}:", label, l, file);
                        let _ = writeln!(out, "       {}", f.text);
                    }
                    None => {
                        let _ = writeln!(out, "{} {}", label, f.text);
                    }
                }
            }
        }
        out
    }
}

/// What `where:` names for a place in a file (owner, 2026-10-22: "`pick()`"):
/// the top-level item open there, read off the file's tokens — so a message
/// made before or without the parsed items still says it. `pick()`, `MAIN`,
/// `struct Item`; `None` between items.
pub fn enclosing_item(file: &crate::source::SourceFile, offset: u32) -> Option<String> {
    use crate::lexer::Tok;
    // A file the lexer stops in (a quote never closed) is read up to the
    // line the message is on.
    let toks = crate::lexer::lex(file.id, &file.text).or_else(|_| {
        let line_start = file.text[..(offset as usize).min(file.text.len())].rfind('\n').map_or(0, |i| i + 1);
        crate::lexer::lex(file.id, &file.text[..line_start])
    }).ok()?;
    let mut depth = 0usize;
    // The depths a `module m {` opened: items inside it are top-level too.
    let mut bases: Vec<usize> = vec![0];
    // The item being read, and whether its `{` has opened.
    let mut cur: Option<(String, bool)> = None;
    let mut found = None;
    // Each open `{`: the line of the loop it opened, if one did; a `for` /
    // `loop` read and not yet opened.
    let mut braces: Vec<Option<u32>> = Vec::new();
    let mut pending_loop: Option<u32> = None;
    let mut i = 0;
    while i < toks.len() {
        let t = &toks[i];
        if t.span.start > offset {
            break;
        }
        let base = *bases.last().unwrap();
        match &t.tok {
            Tok::LBrace => {
                braces.push(pending_loop.take());
                depth += 1;
                if let Some((_, opened)) = &mut cur {
                    if depth == base + 1 {
                        *opened = true;
                    }
                }
            }
            Tok::RBrace => {
                braces.pop();
                depth = depth.saturating_sub(1);
                if bases.len() > 1 && depth + 1 == base {
                    bases.pop();
                } else if depth == base && cur.as_ref().is_some_and(|c| c.1) {
                    cur = None;
                }
            }
            Tok::Semi if depth == base && cur.as_ref().is_some_and(|c| !c.1) => cur = None,
            Tok::Word(w) if depth > base && matches!(w.as_str(), "for" | "loop") => pending_loop = Some(file.line_col(t.span.start).0),
            Tok::Word(w) if depth == base => {
                let name_after = |k: usize| -> Option<String> {
                    let mut parts = Vec::new();
                    let mut k = k;
                    while let Some(Tok::Word(n)) = toks.get(k).map(|t| &t.tok) {
                        parts.push(n.clone());
                        if !matches!(toks.get(k + 1).map(|t| &t.tok), Some(Tok::Dot)) {
                            break;
                        }
                        k += 2;
                    }
                    (!parts.is_empty()).then(|| parts.join("."))
                };
                let head = match w.as_str() {
                    "MAIN" => Some("MAIN".to_string()),
                    "fn" => name_after(i + 1).map(|n| format!("{}()", n)),
                    "struct" | "enum" | "interface" | "const" => name_after(i + 1).map(|n| format!("{} {}", w, n)),
                    "module" => {
                        if let Some(Tok::Word(_)) = toks.get(i + 1).map(|t| &t.tok) {
                            if matches!(toks.get(i + 2).map(|t| &t.tok), Some(Tok::LBrace)) {
                                depth += 1;
                                braces.push(None);
                                bases.push(depth);
                                i += 3;
                                continue;
                            }
                        }
                        None
                    }
                    _ => None,
                };
                if let Some(h) = head {
                    cur = Some((h, false));
                }
            }
            _ => {}
        }
        found = cur.as_ref().map(|c| match braces.iter().rev().find_map(|b| *b) {
            Some(line) if c.1 => format!("{}, the loop starting at line {}", c.0, line),
            _ => c.0.clone(),
        });
        i += 1;
    }
    found
}

/// A `CyJet.toml` place: the section it is under (design claude's
/// reading, 2026-10-24: `where: in CyJet.toml, my kits:`).
fn manifest_section(file: &crate::source::SourceFile, offset: u32) -> Option<String> {
    let before = &file.text[..(offset as usize).min(file.text.len())];
    let start = before.rfind('\n').map_or(0, |i| i + 1);
    before[..start].lines().rev().map(|l| l.split('#').next().unwrap_or("").trim()).find(|l| l.ends_with(':') && !l.contains('=')).map(|l| format!("in {}, {}", file.short_name(), l))
}

impl Diagnostic {
    /// The full form (owner, 2026-10-22; `design/diagnostics.md`): the name
    /// and the place for programs, `where:` for people, the code, a check's
    /// `when:`, and every fix under `or`.
    fn render_full(&self, sources: &SourceMap) -> String {
        let mut out = String::new();
        let title = match self.name {
            Some(n) => format!("{}[{}]", self.kind.label(), n),
            None => self.kind.label().to_string(),
        };
        // A place given as words: "in `f`" is a function, anything else a
        // file (a manifest's path).
        let given = self.place.as_ref().map(|p| match p.strip_prefix("in `").and_then(|r| r.strip_suffix('`')) {
            Some(f) => (true, format!("in {}()", f)),
            None => (false, p.clone()),
        });
        match self.span {
            Some(span) => {
                let file = sources.get(span.file);
                let (line, col) = file.line_col(span.start);
                let _ = writeln!(out, "{} at {}:{}:{}", title, file.display_name(), line, col);
                let toml = file.path.extension().is_some_and(|e| e == "toml");
                let item = if toml { None } else { enclosing_item(file, span.start) };
                let mut where_ = match (&given, item) {
                    _ if toml => manifest_section(file, span.start).unwrap_or_else(|| format!("at the top of {}", file.short_name())),
                    (Some((true, f)), _) => f.clone(),
                    (_, Some(item)) => format!("in {}", item),
                    _ => format!("at the top of {}", file.short_name()),
                };
                if file.concise {
                    let _ = write!(where_, ", decoded line {} of {} (concise)", line, file.short_name());
                }
                let _ = writeln!(out, "where: {}", where_);
                let code = file.slice(span).trim();
                let code = if code.contains('\n') { code.lines().next().unwrap_or("").trim_end().to_string() + " …" } else { code.to_string() };
                if !code.is_empty() {
                    let _ = writeln!(out, "code:  {}", code);
                }
            }
            None => match &given {
                Some((false, p)) => {
                    let _ = writeln!(out, "{} at {}", title, p);
                }
                Some((true, f)) => {
                    let _ = writeln!(out, "{}", title);
                    let _ = writeln!(out, "where: {}", f);
                }
                None => {
                    let _ = writeln!(out, "{}", title);
                }
            },
        }
        if let Some(when) = &self.when {
            let _ = writeln!(out, "when:  {}", when);
        }
        let here = self.span.map(|sp| sources.get(sp.file).line_col(sp.start).0);
        let mut written = 0;
        for f in &self.fixes {
            let label = if written == 0 { "fix:  " } else { "or:   " };
            // On the message's own line, the edited line itself; another
            // line is named above it. An edit the source does not allow
            // (no `give` on that line) is no fix.
            if f.edit.is_some() {
                let Some((line, text)) = f.edited(sources) else { continue };
                if Some(line) == here {
                    let _ = writeln!(out, "{} {}", label, text);
                } else {
                    let _ = writeln!(out, "{} line {}:", label, line);
                    let _ = writeln!(out, "       {}", text);
                }
                written += 1;
                continue;
            }
            written += 1;
            match f.line {
                Some(l) => {
                    let _ = writeln!(out, "{} line {}:", label, l);
                    let _ = writeln!(out, "       {}", f.text);
                }
                None => {
                    let _ = writeln!(out, "{} {}", label, f.text);
                }
            }
        }
        // The sentence stays, last, until the message's fix is written
        // (owner, 2026-10-24: (c)); it is in the docs under the name too.
        if SENTENCE_LINE && written == 0 {
            let _ = writeln!(out, "about: {}", self.sentence);
        }
        out
    }
}

/// The `about:` line, for a message whose fix is not written yet (owner,
/// 2026-10-24: (c)).
pub const SENTENCE_LINE: bool = true;

/// A build stops at the first `error`; `check`s and `note`s accumulate.
pub type Result<T> = std::result::Result<T, Box<Diagnostic>>;

pub fn err<T>(d: Diagnostic) -> Result<T> {
    Err(Box::new(d))
}
