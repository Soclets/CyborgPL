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
    /// The sentence: what was found, why it matters, what to write instead.
    pub sentence: String,
    pub span: Option<Span>,
    /// The human-readable place: "in `pick`, called from MAIN at line 48".
    pub place: Option<String>,
    /// The counterexample, as a sentence.
    pub when: Option<String>,
    /// The root of the chain, one fact per line.
    pub why: Vec<String>,
    /// A command that reproduces it.
    pub try_: Option<String>,
    /// The edited line(s); alternatives after the first go under `or`.
    pub fixes: Vec<Fix>,
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
        Diagnostic { kind, sentence: sentence.into(), span: None, place: None, when: None, why: Vec::new(), try_: None, fixes: Vec::new() }
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
        self.why.push(why.into());
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
        let _ = writeln!(out, "{}: {}", self.kind.label(), self.sentence);
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
            for (i, w) in self.why.iter().enumerate() {
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
                        let _ = writeln!(out, "{} line {}:", label, l);
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

pub fn err<T>(d: Diagnostic) -> Result<T> {
    Err(Box::new(d))
}
