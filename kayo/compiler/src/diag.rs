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
    /// More edits made together with it, each with its own text.
    pub also: Vec<(Span, Edit, String)>,
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
    /// `text` is a new line above the span's, indented as it is.
    InsertLine,
    /// `text` is a new line below the span's, indented as it is.
    InsertLineAfter,
    /// `text` goes in at the span's start.
    Insert,
    /// The whole line (or lines) the span is on goes.
    DeleteLine,
    /// `text` is a new last line of the file.
    Append,
    /// The span's label (`word:` at its start) becomes `text`, or `text`
    /// goes in front when it has none.
    Label,
    /// The span's text becomes another span's, as written.
    TextOf(Span),
    /// `text` goes in front of the word before the span (`pub ` before
    /// `fn f`, the span being the name).
    BeforeWord,
    /// The verb the span starts with (`lend`, `lend.mut`, `give`, `copy`)
    /// becomes `text`, or `text` goes in front when it has none; an empty
    /// `text` takes the verb out.
    Verb,
    /// From the word given (and the space before it) to the span's end goes:
    /// a clause, `otherwise …`.
    CutFrom(String),
}

impl Fix {
    /// The edit as a replacement in its file: the bytes `span` covers
    /// become the text (`--message-format json` carries it, so an editor
    /// can apply it; the tests apply it too). `None` when the source does
    /// not allow it.
    pub fn replacements(&self, sources: &SourceMap) -> Option<Vec<(Span, String)>> {
        let first = self.replacement(sources)?;
        let mut out = vec![first];
        for (span, how, text) in &self.also {
            out.push(Fix { line: None, text: text.clone(), edit: Some((*span, how.clone())), also: Vec::new() }.replacement(sources)?);
        }
        Some(out)
    }

    pub fn replacement(&self, sources: &SourceMap) -> Option<(Span, String)> {
        let (span, how) = self.edit.clone()?;
        let file = sources.get(span.file);
        let line_start = file.text[..span.start as usize].rfind('\n').map_or(0, |i| i + 1);
        let (line, _) = file.line_col(span.start);
        let text = file.line_text(line);
        let at = |from: usize, to: usize| Span::new(span.file, line_start + from, line_start + to);
        let indent = text.len() - text.trim_start().len();
        Some(match how {
            Edit::Replace => (span, self.text.clone()),
            Edit::Wrap => (span, format!("{}({})", self.text, file.slice(span))),
            Edit::Swap(from) => {
                // A whole word: `give $s` is not in `give $sx`.
                let whole = |at: usize, len: usize| !text[at + len..].starts_with(|c: char| c.is_alphanumeric() || c == '_');
                let (pos, len) = from.iter().find_map(|w| text.match_indices(w.as_str()).map(|(at, _)| at).find(|&at| whole(at, w.len())).map(|at| (at, w.len())))?;
                (at(pos, pos + len), self.text.clone())
            }
            Edit::Before => (at(indent, indent), self.text.clone()),
            Edit::Insert => (Span::new(span.file, span.start as usize, span.start as usize), self.text.clone()),
            Edit::TextOf(from) => (span, sources.get(from.file).slice(from).to_string()),
            Edit::Verb => {
                let here = &file.text[span.start as usize..span.end as usize];
                let verb = ["lend.mut", "lend", "give", "copy"].into_iter().find(|v| here.starts_with(v) && here[v.len()..].starts_with([' ', '$', '\'']));
                match verb {
                    Some(v) => {
                        let space = usize::from(here[v.len()..].starts_with(' '));
                        let with = if self.text.is_empty() { String::new() } else { format!("{} ", self.text) };
                        (Span::new(span.file, span.start as usize, span.start as usize + v.len() + space), with)
                    }
                    None if self.text.is_empty() => return None,
                    None => (Span::new(span.file, span.start as usize, span.start as usize), format!("{} ", self.text)),
                }
            }
            Edit::CutFrom(word) => {
                let here = &file.text[span.start as usize..span.end as usize];
                let at = here.find(&format!(" {}", word)).or_else(|| here.find(word.as_str()))?;
                (Span::new(span.file, span.start as usize + at, span.end as usize), String::new())
            }
            Edit::BeforeWord => {
                let before = file.text[..span.start as usize].trim_end_matches([' ', '\t']);
                let word = before.trim_end_matches(|c: char| c.is_alphanumeric() || c == '_' || c == '.');
                (Span::new(span.file, word.len(), word.len()), self.text.clone())
            }
            Edit::Label => {
                let here = &file.text[span.start as usize..];
                let word = here.chars().take_while(|c| c.is_alphanumeric() || *c == '_').map(char::len_utf8).sum::<usize>();
                let after = &here[word..];
                if word > 0 && after.starts_with(':') {
                    let space = after[1..].len() - after[1..].trim_start_matches(' ').len();
                    (Span::new(span.file, span.start as usize, span.start as usize + word + 1 + space), self.text.clone())
                } else {
                    (Span::new(span.file, span.start as usize, span.start as usize), self.text.clone())
                }
            }
            Edit::Append => {
                let end = file.text.len();
                let lead = if file.text.ends_with('\n') || file.text.is_empty() { "" } else { "\n" };
                (Span::new(span.file, end, end), format!("{}{}\n", lead, self.text))
            }
            Edit::DeleteLine => {
                let end = file.text[span.end as usize..].find('\n').map_or(file.text.len(), |i| span.end as usize + i + 1);
                (Span::new(span.file, line_start, end), String::new())
            }
            Edit::Remove => {
                let pos = text.find(&self.text)?;
                let mut end = pos + self.text.len();
                if text[end..].starts_with(' ') {
                    end += 1;
                }
                (at(pos, end), String::new())
            }
            Edit::InsertLineAfter => {
                let end = file.text[span.start as usize..].find('\n').map_or(file.text.len(), |i| span.start as usize + i + 1);
                let lead = if end == file.text.len() && !file.text.ends_with('\n') { "\n" } else { "" };
                (Span::new(span.file, end, end), format!("{}{}{}\n", lead, &text[..indent], self.text))
            }
            Edit::InsertLine => {
                // Above a block's `}`, the block's own indentation: the
                // line before it.
                let indent = if text.trim_start().starts_with('}') && line > 1 {
                    let above = file.line_text(line - 1);
                    above[..above.len() - above.trim_start().len()].to_string()
                } else {
                    text[..indent].to_string()
                };
                (at(0, 0), format!("{}{}\n", indent, self.text))
            }
        })
    }

    /// The edited line, its number and its file, for an edit made from the
    /// source: the replacement made, and the line it starts on read back.
    fn edited(&self, sources: &SourceMap) -> Option<(crate::source::FileId, u32, String)> {
        let mut all = self.replacements(sources)?;
        let (span, _) = all[0].clone();
        let file = sources.get(span.file);
        let mut text = file.text.clone();
        // Last first, so the earlier offsets hold.
        all.sort_by(|a, b| b.0.start.cmp(&a.0.start));
        for (sp, with) in all.iter().filter(|(sp, _)| sp.file == span.file) {
            text.replace_range(sp.start as usize..sp.end as usize, with);
        }
        // The line of the earliest edit: nothing before it moved; text put
        // in after a line's end is shown from where it starts.
        let span = all.iter().filter(|(sp, _)| sp.file == span.file).map(|(sp, _)| *sp).min_by_key(|sp| sp.start).unwrap_or(span);
        let first = all.iter().find(|(sp, _)| *sp == span).map(|(_, t)| t.as_str()).unwrap_or("");
        let at = span.start as usize + if first.starts_with('\n') { 1 } else { 0 };
        let start = text[..at].rfind('\n').map_or(0, |i| i + 1);
        let end = text[start..].find('\n').map_or(text.len(), |i| start + i);
        let line = text[..start].matches('\n').count() as u32 + 1;
        Some((span.file, line, text[start..end].trim().to_string()))
    }

    /// The other lines the fix edits, once it is made: a guard's closing
    /// `}` under its `if`. Each by its number and its text.
    fn edited_more(&self, sources: &SourceMap) -> Vec<(u32, String)> {
        let Some(all) = self.replacements(sources) else { return Vec::new() };
        let Some((first, _)) = self.edited(sources).map(|(f, l, _)| (f, l)) else { return Vec::new() };
        let file = sources.get(first);
        let mut text = file.text.clone();
        let mut sorted: Vec<&(Span, String)> = all.iter().filter(|(sp, _)| sp.file == first).collect();
        sorted.sort_by(|a, b| b.0.start.cmp(&a.0.start));
        for (sp, with) in &sorted {
            text.replace_range(sp.start as usize..sp.end as usize, with);
        }
        let (_, first_line, _) = self.edited(sources).unwrap();
        let mut out: Vec<(u32, String)> = Vec::new();
        for (sp, with) in all.iter().filter(|(sp, _)| sp.file == first) {
            // Where it starts once every edit before it is made.
            let moved: i64 = all.iter().filter(|(o, _)| o.file == first && o.start < sp.start).map(|(o, t)| t.len() as i64 - (o.end - o.start) as i64).sum();
            let at = (sp.start as i64 + moved) as usize + usize::from(with.starts_with('\n'));
            let start = text[..at.min(text.len())].rfind('\n').map_or(0, |i| i + 1);
            let end = text[start..].find('\n').map_or(text.len(), |i| start + i);
            let line = text[..start].matches('\n').count() as u32 + 1;
            if line != first_line && !out.iter().any(|(l, _)| *l == line) {
                out.push((line, text[start..end].trim().to_string()));
            }
        }
        out.sort();
        out
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
    /// The fix lies outside the program's own code (owner, 2026-10-24):
    /// `fix: none suggested`.
    pub outside: bool,
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
        Diagnostic { kind, name: None, sentence: dollar_names(sentence.into()), span: None, place: None, when: None, why: Vec::new(), try_: None, fixes: Vec::new(), outside: false }
    }
    /// Its name from `errors.cyr`.
    pub fn named(mut self, name: &'static str) -> Diagnostic {
        self.name = Some(name);
        // Its fix is kayo's to build or the owner's to rule on, never in the
        // writer's code (design claude, 2026-10-24).
        if matches!(name, "not_built_yet" | "not_decided") {
            self.outside = true;
        }
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
        self.fixes.push(Fix { line, text: text.into(), edit: None, also: Vec::new() });
        self
    }
    /// The close name, if there is one, in place of the one written at `at`.
    pub fn fix_close<'a>(self, at: Span, written: &str, candidates: impl IntoIterator<Item = &'a str>) -> Diagnostic {
        match closest(written, candidates) {
            Some(c) => {
                let c = c.to_string();
                self.fix_edit(at, Edit::Replace, c)
            }
            None => self,
        }
    }
    /// Another edit made with the last fix (both are one fix).
    pub fn fix_also(mut self, span: Span, how: Edit, text: impl Into<String>) -> Diagnostic {
        if let Some(f) = self.fixes.last_mut() {
            f.also.push((span, how, text.into()));
        }
        self
    }
    /// The fix is outside the program's own code: two kits with one
    /// prefix, a program past the target's limits (owner, 2026-10-24).
    pub fn outside_program(mut self) -> Diagnostic {
        self.outside = true;
        self
    }
    /// A fix made from the source: the line holding `span`, edited.
    pub fn fix_edit(mut self, span: Span, how: Edit, text: impl Into<String>) -> Diagnostic {
        let text = text.into();
        // The same edit twice is one fix.
        if !self.fixes.iter().any(|f| f.also.is_empty() && f.text == text && f.edit.as_ref() == Some(&(span, how.clone()))) {
            self.fixes.push(Fix { line: None, text, edit: Some((span, how)), also: Vec::new() });
        }
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

/// Each message as a JSON line instead (`--message-format json`).
static JSON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn json_messages() {
    JSON.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// A message as the tests see it: its name, where, and each fix's
/// replacements, path and byte range in that file's text.
#[derive(Debug, Clone)]
pub struct Shown {
    pub name: Option<&'static str>,
    /// Its file, line, column and byte offset.
    pub at: Option<(std::path::PathBuf, u32, u32, u32)>,
    pub fixes: Vec<Vec<(std::path::PathBuf, u32, u32, String)>>,
}

thread_local! {
    /// Filled by every render on this thread while a test collects.
    static SHOWN: std::cell::RefCell<Option<Vec<Shown>>> = const { std::cell::RefCell::new(None) };
}

/// Start keeping what this thread renders.
pub fn collect() {
    SHOWN.with(|s| *s.borrow_mut() = Some(Vec::new()));
}

/// What this thread rendered since `collect`, and stop keeping it.
pub fn collected() -> Vec<Shown> {
    SHOWN.with(|s| s.borrow_mut().take().unwrap_or_default())
}

fn json_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

impl Diagnostic {
    /// The fixes kayo can write here, each as its replacements.
    fn replacements(&self, sources: &SourceMap) -> Vec<Vec<(Span, String)>> {
        self.fixes.iter().filter_map(|f| f.replacements(sources)).collect()
    }

    /// One JSON line, rustc's and LSP's shapes where they fit: the
    /// severity, the name, the place (line and a UTF-16 column, as the
    /// Language Server Protocol counts, owner 2026-10-06), and each fix
    /// as replacements, byte ranges with their lines and columns.
    fn render_json(&self, sources: &SourceMap) -> String {
        let place = |sp: Span| {
            let f = sources.get(sp.file);
            let (l1, c1) = f.line_col_utf16(sp.start);
            let (l2, c2) = f.line_col_utf16(sp.end);
            format!("\"file\":{},\"byte_start\":{},\"byte_end\":{},\"line_start\":{},\"column_start\":{},\"line_end\":{},\"column_end\":{}", json_text(&f.display_name()), sp.start, sp.end, l1, c1, l2, c2)
        };
        let mut out = format!("{{\"severity\":{},\"name\":{}", json_text(self.kind.label()), self.name.map(json_text).unwrap_or_else(|| "null".into()));
        if let Some(sp) = self.span {
            let _ = write!(out, ",\"span\":{{{}}},\"code\":{}", place(sp), json_text(sources.get(sp.file).slice(sp).trim()));
        }
        if let Some(w) = &self.when {
            let _ = write!(out, ",\"when\":{}", json_text(w));
        }
        let fixes: Vec<String> = self.replacements(sources).iter().map(|rs| {
            let parts: Vec<String> = rs.iter().map(|(sp, text)| format!("{{{},\"new_text\":{}}}", place(*sp), json_text(text))).collect();
            format!("{{\"replacements\":[{}]}}", parts.join(","))
        }).collect();
        let _ = write!(out, ",\"fixes\":[{}]", fixes.join(","));
        if fixes.is_empty() && self.outside {
            out.push_str(",\"none_suggested\":true");
        }
        if fixes.is_empty() && SENTENCE_LINE && !self.outside {
            let _ = write!(out, ",\"about\":{}", json_text(&self.sentence));
        }
        out.push_str("}\n");
        out
    }

    /// The full form (owner, 2026-10-22; `design/diagnostics.md`): the name
    /// and the place for programs, `where:` for people, the code, a check's
    /// `when:`, and every fix under `or`.
    fn render_full(&self, sources: &SourceMap) -> String {
        SHOWN.with(|s| {
            if let Some(v) = s.borrow_mut().as_mut() {
                let path = |sp: Span| sources.get(sp.file).path.clone();
                v.push(Shown {
                    name: self.name,
                    at: self.span.map(|sp| {
                        let (l, c) = sources.get(sp.file).line_col(sp.start);
                        (path(sp), l, c, sp.start)
                    }),
                    fixes: self.replacements(sources).into_iter().map(|rs| rs.into_iter().map(|(sp, t)| (path(sp), sp.start, sp.end, t)).collect()).collect(),
                });
            }
        });
        if JSON.load(std::sync::atomic::Ordering::Relaxed) {
            return self.render_json(sources);
        }
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
            // A line that goes is shown as it was, named as going.
            if let (Some((span, Edit::DeleteLine)), true) = (&f.edit, f.also.is_empty()) {
                let file = sources.get(span.file);
                let (line, _) = file.line_col(span.start);
                let _ = writeln!(out, "{} line {} goes:", label, line);
                let _ = writeln!(out, "       {}", file.line_text(line).trim());
                written += 1;
                continue;
            }
            if f.edit.is_some() {
                let Some((file, line, text)) = f.edited(sources) else { continue };
                let inserted = matches!(f.edit, Some((_, Edit::InsertLine)));
                if Some(line) == here && Some(file) == self.span.map(|s| s.file) && !inserted {
                    let _ = writeln!(out, "{} {}", label, text);
                } else if Some(file) == self.span.map(|s| s.file) {
                    let _ = writeln!(out, "{} line {}:", label, line);
                    let _ = writeln!(out, "       {}", text);
                    for (more, t) in f.edited_more(sources) {
                        let _ = writeln!(out, "       line {}:", more);
                        let _ = writeln!(out, "       {}", t);
                    }
                } else {
                    let _ = writeln!(out, "{} line {} of {}:", label, line, sources.get(file).short_name());
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
        if written == 0 && self.outside {
            let _ = writeln!(out, "fix:   none suggested");
        }
        // The sentence stays, last, until the message's fix is written
        // (owner, 2026-10-24: (c)); it is in the docs under the name too.
        if SENTENCE_LINE && written == 0 && !self.outside {
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

/// The name among these that the one written most likely meant: a few
/// letters off, case aside (owner, 2026-10-24: "the close name if there is
/// one"), the nearest, and alphabetically first among the nearest.
pub fn closest<'a>(name: &str, candidates: impl IntoIterator<Item = &'a str>) -> Option<&'a str> {
    // Letters added, taken out, changed, or two swapped, each one step.
    fn distance(a: &[char], b: &[char]) -> usize {
        let mut d = vec![vec![0usize; b.len() + 1]; a.len() + 1];
        for (i, row) in d.iter_mut().enumerate() {
            row[0] = i;
        }
        for j in 0..=b.len() {
            d[0][j] = j;
        }
        for i in 1..=a.len() {
            for j in 1..=b.len() {
                let cost = usize::from(a[i - 1] != b[j - 1]);
                d[i][j] = (d[i - 1][j] + 1).min(d[i][j - 1] + 1).min(d[i - 1][j - 1] + cost);
                if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                    d[i][j] = d[i][j].min(d[i - 2][j - 2] + 1);
                }
            }
        }
        d[a.len()][b.len()]
    }
    let lower: Vec<char> = name.to_lowercase().chars().collect();
    let limit = (lower.len() / 3).max(1);
    let mut best: Option<(usize, &str)> = None;
    for c in candidates {
        if c == name || c.is_empty() {
            continue;
        }
        let d = distance(&lower, &c.to_lowercase().chars().collect::<Vec<_>>());
        if d <= limit && best.is_none_or(|(bd, bc)| d < bd || (d == bd && c < bc)) {
            best = Some((d, c));
        }
    }
    best.map(|b| b.1)
}

/// A variable's name as written: `$x`, quoted when it needs quotes.
pub fn name_written(n: &str) -> String {
    if !n.is_empty() && n.chars().all(|c| c.is_alphanumeric() || c == '_') { format!("${}", n) } else { format!("'{}'", n) }
}

/// A fix on a refusal on its way out: `return self.fail_n(…).with_fix(…)`.
pub trait WithFix {
    fn with_fix(self, span: Span, how: Edit, text: impl Into<String>) -> Self;
    fn outside_program(self) -> Self;
}

impl<T> WithFix for Result<T> {
    fn with_fix(self, span: Span, how: Edit, text: impl Into<String>) -> Self {
        self.map_err(|d| Box::new(d.fix_edit(span, how, text)))
    }
    fn outside_program(self) -> Self {
        self.map_err(|d| Box::new(d.outside_program()))
    }
}

pub fn err<T>(d: Diagnostic) -> Result<T> {
    Err(Box::new(d))
}
