//! The limb mark (owner, 2026-10-26; `design/cores.md`, "The limb mark"):
//! every piece a limb adds is marked `%`, the base is bare, a kit is its
//! path. The base places every mark by one rule: it goes on a limb's first
//! word in a dotted path, so `%extern.C`, `to.%d64`, `inline.%MSL`, `40
//! %graphemes`. Names are never marked, nor anything after a kit's `::`,
//! nor a word used as a label or a field. Checked on a file's tokens as
//! written, before anything else reads them; a marked word then reads as
//! the bare one always did.

use crate::diag::{err, Diagnostic, Edit, Result};
use crate::lexer::{Tok, Token};

/// The limb a word is the first word of, where it stands: `before`, the word
/// before it in the path (`to` in `to.d64`), if any; `after`, the next one;
/// `prev`, the token before the path.
fn limb_of(word: &str, before: Option<&str>, after: Option<&str>, prev: Option<&Tok>) -> Option<&'static str> {
    Some(match (before, word) {
        (None, "extern" | "layout") if after.is_some() => "CNC",
        (None, "C") if after.is_some() => "CNC",
        // A limb's number type is marked wherever it stands (owner, 2026-10-28):
        // `to.stored.%d64` too.
        (_, "d32" | "d64" | "d128") => "Money",
        (_, "i128" | "u128" | "f128") => "WSMW",
        (None, "not_std") if after.is_some() => "WSMW",
        (_, "brain16" | "bf16") => "AI",
        (None, "concur" | "channel" | "whichever") => "MCPS",
        (None, "para") if matches!(prev, Some(Tok::Word(w)) if w == "for") => "MCPS",
        (Some("incl"), "C") => "CNC",
        (Some("inline"), "C") => "CNC",
        (Some("inline"), "MSL" | "Metal_Shading_Language") => "Metal",
        (Some("char"), "point" | "grapheme") => "unicode",
        (Some("str"), "utf16" | "utf32") => "unicode",
        // A text's unit after a number: `40 %graphemes`.
        (None, "graphemes" | "grapheme" | "points" | "point") if matches!(prev, Some(Tok::Num { .. })) => "unicode",
        _ => return None,
    })
}

/// A limb's number type, marked wherever it stands in a path.
fn number_type(w: &str) -> bool {
    matches!(w, "d32" | "d64" | "d128" | "i128" | "u128" | "f128" | "brain16" | "bf16")
}

fn word(t: &Token) -> Option<&str> {
    match &t.tok {
        Tok::Word(w) => Some(w.as_str()),
        _ => None,
    }
}

pub fn check(toks: &[Token]) -> Result<()> {
    let mut i = 0;
    while i < toks.len() {
        if word(&toks[i]).is_none() {
            if toks[i].marked {
                return err(Diagnostic::error("a `%` marks a limb's word, glued to it: `%d64`").named("limb_mark_misplaced").at(toks[i].span));
            }
            i += 1;
            continue;
        }
        // A path: words joined by `.`, and whether it follows a kit's `::`
        // or a value's `.` (a field, never marked).
        let prev = i.checked_sub(1).map(|p| &toks[p].tok);
        let kit = matches!(prev, Some(Tok::ColonColon));
        let field_of_value = matches!(prev, Some(Tok::Dot));
        let mut end = i + 1;
        while matches!(toks.get(end).map(|t| &t.tok), Some(Tok::Dot)) && toks.get(end + 1).is_some_and(|t| word(t).is_some()) {
            end += 2;
        }
        let path: Vec<&Token> = toks[i..end].iter().step_by(2).collect();
        // A label or a field, `name: …` / `name = …` right after `[ ( { , ;`.
        let after = toks.get(end).map(|t| &t.tok);
        let labelled = path.len() == 1
            && matches!(prev, Some(Tok::LBracket | Tok::LParen | Tok::LBrace | Tok::Comma | Tok::Semi))
            && (matches!(after, Some(Tok::Colon)) || matches!(after, Some(Tok::Op(o)) if o == "="));
        let spelled = |marked_at: Option<usize>| -> String {
            path.iter().enumerate().map(|(n, t)| format!("{}{}", if Some(n) == marked_at { "%" } else { "" }, word(t).unwrap_or(""))).collect::<Vec<_>>().join(".")
        };
        // Which word of the path, if any, is a limb's first, and which limb.
        let mut first: Option<(usize, &'static str)> = None;
        if !kit && !field_of_value && !labelled {
            for n in 0..path.len() {
                let before = n.checked_sub(1).and_then(|b| word(path[b]));
                let next = path.get(n + 1).and_then(|t| word(t));
                // Only a base word before a limb's own first one: `to.`,
                // `incl.`, `inline.`, `char.`, `str.`, or none.
                if n > 0 && !matches!(before, Some("to" | "incl" | "inline" | "char" | "str")) && !number_type(word(path[n]).unwrap_or("")) {
                    continue;
                }
                if let Some(limb) = limb_of(word(path[n]).unwrap_or(""), before, next, if n == 0 { prev } else { None }) {
                    first = Some((n, limb));
                    break;
                }
            }
        }
        for (n, t) in path.iter().enumerate() {
            let w = word(t).unwrap_or("");
            match (first, t.marked) {
                (Some((f, limb)), false) if f == n => {
                    let at = Diagnostic::error(format!("`{}` is the {} limb's, so it is marked: `{}` (owner, 2026-10-26)", w, limb, spelled(Some(n)))).named("limb_mark_missing").at(t.span);
                    return err(at.fix_edit(t.span, Edit::Insert, "%"));
                }
                (Some((f, limb)), true) if n > f => {
                    let head = word(path[f]).unwrap_or("");
                    return err(Diagnostic::error(format!("`{}` is part of the {} limb's `{}`, marked once: `{}`", w, limb, head, spelled(Some(f)))).named("limb_mark_misplaced").at(t.span).fix_edit(t.span, Edit::Replace, w));
                }
                (_, true) if kit => {
                    return err(Diagnostic::error(format!("`%{}`: a kit's path is never marked; a mark goes on a limb's own first word", w)).named("limb_mark_misplaced").at(t.span).fix_edit(t.span, Edit::Replace, w));
                }
                (None, true) | (Some(_), true) if first.map(|(f, _)| f) != Some(n) => {
                    let why = if labelled || field_of_value { "a name or a label is never marked" } else { "it is the base's, or no limb adds it; a mark goes on a limb's own first word" };
                    return err(Diagnostic::error(format!("`%{}`: `{}` {}", w, w, why)).named("limb_mark_misplaced").at(t.span).fix_edit(t.span, Edit::Replace, w));
                }
                _ => {}
            }
        }
        i = end;
    }
    Ok(())
}

/// The code in a message, marked as a program must be written (owner,
/// 2026-10-26: the messages show what they suggest with its marks): each
/// `` `…` `` in `sentence`, its limb words marked by the table above where
/// they are not yet. A message's own text outside the backticks is left.
pub fn in_message(sentence: &str) -> String {
    if !sentence.contains('`') {
        return sentence.to_string();
    }
    sentence.split('`').enumerate().map(|(n, part)| if n % 2 == 1 { mark_code(part) } else { part.to_string() }).collect::<Vec<_>>().join("`")
}

/// One piece of code with its limb words marked.
fn mark_code(code: &str) -> String {
    #[derive(PartialEq)]
    enum K {
        Word,
        Num,
        Other,
    }
    // Words, numbers and everything else, each with where it starts.
    let b = code.as_bytes();
    let mut pieces: Vec<(K, usize, usize)> = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        let start = i;
        if c.is_ascii_alphabetic() || c == b'_' {
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            pieces.push((K::Word, start, i));
        } else if c.is_ascii_digit() {
            while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'_' || b[i] == b'.') {
                i += 1;
            }
            pieces.push((K::Num, start, i));
        } else {
            i += code[i..].chars().next().map_or(1, |c| c.len_utf8());
            pieces.push((K::Other, start, i));
        }
    }
    let text = |p: &(K, usize, usize)| &code[p.1..p.2];
    let mut marks: Vec<usize> = Vec::new();
    let mut k = 0;
    while k < pieces.len() {
        if pieces[k].0 != K::Word {
            k += 1;
            continue;
        }
        // The path from here: words joined by `.`.
        let mut path = vec![k];
        let mut j = k + 1;
        while j + 1 < pieces.len() && text(&pieces[j]) == "." && pieces[j + 1].0 == K::Word {
            path.push(j + 1);
            j += 2;
        }
        let before_path = k.checked_sub(1).map(|p| text(&pieces[p]));
        let marked_already = |w: usize| pieces[w].1 > 0 && &code[pieces[w].1 - 1..pieces[w].1] == "%";
        let after_kit = k >= 2 && text(&pieces[k - 1]) == ":" && text(&pieces[k - 2]) == ":";
        let field = before_path == Some(".") || before_path == Some("$") || before_path == Some("'");
        // The token before the path, skipping spaces: a number, or `for`.
        let mut p = k;
        while p > 0 && text(&pieces[p - 1]).trim().is_empty() {
            p -= 1;
        }
        let prev = p.checked_sub(1).map(|q| match pieces[q].0 {
            K::Num => Tok::Num { text: text(&pieces[q]).to_string(), bin: false, percent: false },
            K::Word => Tok::Word(text(&pieces[q]).to_string()),
            K::Other => Tok::Op(text(&pieces[q]).to_string()),
        });
        if !after_kit && !field {
            for n in 0..path.len() {
                let w = text(&pieces[path[n]]);
                let before = n.checked_sub(1).map(|m| text(&pieces[path[m]]));
                let next = path.get(n + 1).map(|&m| text(&pieces[m]));
                if n > 0 && !matches!(before, Some("to" | "incl" | "inline" | "char" | "str")) && !number_type(w) {
                    continue;
                }
                if limb_of(w, before, next, if n == 0 { prev.as_ref() } else { None }).is_some() {
                    if !marked_already(path[n]) {
                        marks.push(pieces[path[n]].1);
                    }
                    break;
                }
            }
        }
        k = j;
    }
    let mut out = String::with_capacity(code.len() + marks.len());
    let mut last = 0;
    for m in marks {
        out.push_str(&code[last..m]);
        out.push('%');
        last = m;
    }
    out.push_str(&code[last..]);
    out
}

