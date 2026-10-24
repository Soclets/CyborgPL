//! `use <another kit's path> as <name>;` (owner, 2026-10-22; Kotlin's shape):
//! one per line at the top of a file, `as` always written. The name stands for
//! the path in that file, read as a rewrite of the tokens before parsing, as
//! the oracle reads it: wherever the name starts a path, the path is written
//! instead, every rename in one pass over the written text, so a rename's
//! output is never renamed again (design claude's readings, not asked: no
//! wildcard; `use my.…` is refused; nothing in the file may be declared under
//! a `use`'s name). Every `use` gets a note in the build report.

use crate::diag::{err, Diagnostic, Result};
use crate::lexer::{Tok, Token};

thread_local! {
    /// Every `use` read in this build: where, the name, and the path, for
    /// the report's notes.
    static SEEN: std::cell::RefCell<Vec<(crate::source::Span, String, String)>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// The `use` lines read so far: where each is, its name, its path.
pub fn seen() -> Vec<(crate::source::Span, String, String)> {
    SEEN.with(|s| s.borrow().clone())
}

fn spelled(toks: &[Token]) -> String {
    toks.iter().map(|t| match &t.tok {
        Tok::Word(w) => w.clone(),
        Tok::Dot => ".".into(),
        Tok::ColonColon => "::".into(),
        other => other.describe(),
    }).collect()
}

/// A path's tokens at `i`: words joined by `.` and `::`. Its end.
fn path_end(toks: &[Token], mut i: usize) -> usize {
    if !matches!(toks.get(i).map(|t| &t.tok), Some(Tok::Word(_))) {
        return i;
    }
    i += 1;
    while matches!(toks.get(i).map(|t| &t.tok), Some(Tok::Dot | Tok::ColonColon)) && matches!(toks.get(i + 1).map(|t| &t.tok), Some(Tok::Word(_))) {
        i += 2;
    }
    i
}

pub fn rewrite(toks: &[Token]) -> Result<Vec<Token>> {
    let mut i = 0;
    let mut uses: Vec<(Vec<Token>, Vec<Token>)> = Vec::new();
    while matches!(&toks.get(i).map(|t| &t.tok), Some(Tok::Word(w)) if w == "use") {
        let at = toks[i].span;
        let p0 = i + 1;
        let p1 = path_end(toks, p0);
        let path = toks[p0..p1].to_vec();
        if path.is_empty() {
            return err(Diagnostic::error("`use` names another kit's path, then `as` and the name: `use std::list as list;`").named("use_form").at(at));
        }
        if !path.iter().any(|t| t.tok == Tok::ColonColon) || matches!(&path[0].tok, Tok::Word(w) if w == "my") {
            return err(Diagnostic::error(format!("`use` names another kit's path (`std::…`, `greet::…`); `{}` is this kit's own, which is written as it is (owner, 2026-10-22)", spelled(&path))).named("use_form").at(at));
        }
        if !matches!(&toks.get(p1).map(|t| &t.tok), Some(Tok::Word(w)) if w == "as") {
            return err(Diagnostic::error(format!("`use` says the name after `as`: `use {} as name;` (owner, 2026-10-22)", spelled(&path))).named("use_form").at(at));
        }
        let n0 = p1 + 1;
        let n1 = path_end(toks, n0);
        let name = toks[n0..n1].to_vec();
        if name.is_empty() || !matches!(toks.get(n1).map(|t| &t.tok), Some(Tok::Semi)) {
            return err(Diagnostic::error(format!("`use` says the name after `as`, then `;`: `use {} as name;` (owner, 2026-10-22)", spelled(&path))).named("use_form").at(at));
        }
        SEEN.with(|s| s.borrow_mut().push((at, spelled(&name), spelled(&path))));
        uses.push((name, path));
        i = n1 + 1;
    }
    if uses.is_empty() && !toks.iter().any(|t| t.line_first && matches!(&t.tok, Tok::Word(w) if w == "use")) {
        return Ok(toks.to_vec());
    }
    let mut out: Vec<Token> = Vec::with_capacity(toks.len());
    let mut k = i;
    while k < toks.len() {
        let t = &toks[k];
        if t.line_first && matches!(&t.tok, Tok::Word(w) if w == "use") {
            return err(Diagnostic::error("a `use` line goes at the top of the file, before anything else (owner, 2026-10-22)").named("use_form").at(t.span));
        }
        let head = k == 0 || !matches!(toks[k - 1].tok, Tok::Dot | Tok::ColonColon);
        let hit = head.then(|| uses.iter().find(|(name, _)| toks.get(k..k + name.len()).is_some_and(|w| w.iter().zip(name).all(|(a, b)| a.tok == b.tok)) && !matches!(toks.get(k + name.len()).map(|t| &t.tok), Some(Tok::ColonColon)))).flatten();
        if let Some((name, path)) = hit {
            // Declared under a `use`'s name: refused.
            if k > 0 && matches!(&toks[k - 1].tok, Tok::Word(w) if matches!(w.as_str(), "fn" | "struct" | "enum" | "interface" | "module" | "const")) {
                return err(Diagnostic::error(format!("`{}` is this file's name for `{}` (its `use`): declare this under another name", spelled(name), spelled(path))).named("use_name_taken").at(t.span));
            }
            for p in path {
                let mut p = p.clone();
                p.span = t.span;
                p.line_first = false;
                out.push(p);
            }
            if let Some(first) = out.len().checked_sub(path.len()) {
                out[first].line_first = t.line_first;
            }
            k += name.len();
            continue;
        }
        out.push(t.clone());
        k += 1;
    }
    Ok(out)
}
