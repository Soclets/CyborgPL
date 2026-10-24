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
        self.fixes.push(Fix { line, text: text.into() });
        self
    }

    /// The plain-text form. This *is* the message; anything else a terminal
    /// draws is decoration.
    pub fn render(&self, sources: &SourceMap) -> String {
        let mut out = String::new();
        // A concise file's line and column mean nothing (design claude's
        // reading): the decoded line, named and shown.
        let concise = self.span.filter(|s| sources.get(s.file).concise).map(|s| {
            let f = sources.get(s.file);
            let (line, _) = f.line_col(s.start);
            format!(" (concise file; decoded line {}: `{}`)", line, f.line_text_at(s.start).trim())
        });
        let _ = writeln!(out, "{}: {}{}", self.kind.label(), self.sentence, concise.unwrap_or_default());
        // While each message is given its name (owner, 2026-10-22), the
        // tests ask for it here; the full format replaces this line.
        if std::env::var_os("KAYO_NAMES").is_some() || SHOW_NAMES.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = writeln!(out, "name:  {}", self.name.unwrap_or("(none)"));
        }
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

/// A build stops at the first `error`; `check`s and `note`s accumulate.
pub type Result<T> = std::result::Result<T, Box<Diagnostic>>;

/// The tests' way to the names while they are not yet the first line.
static SHOW_NAMES: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Every message after this shows its name (for the tests).
pub fn show_names() {
    SHOW_NAMES.store(true, std::sync::atomic::Ordering::Relaxed);
}

pub fn err<T>(d: Diagnostic) -> Result<T> {
    Err(Box::new(d))
}
