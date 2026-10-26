//! The limb mark (owner, 2026-10-26): every piece a limb adds is marked `%`, the base bare, a kit by its path. The base
//! places every mark by one rule, a limb's first word in a path ("Seems good", then "yes"), and each limb marks its own,
//! so one limb extending another's shows two, `%layout.%Cpp` ("yes"). The official limbs' words are the table below.
//! This pass checks every word against it and takes the marks out, so the parser reads what it always read.

use crate::lexer::{Tok, Token};

/// The limb a word is, from the path segments before it and the tokens around it; `None` for the base's.
fn limb_of(w: &str, before: &[String], prev: &Tok, next: &Tok) -> Option<&'static str> {
    let dot_next = *next == Tok::Dot;
    match before.last().map(|s| s.as_str()) {
        None => match w {
            "d32" | "d64" | "d128" => Some("Money"),
            "i128" | "u128" | "f128" | "not_std" => Some("WSMW"),
            "brain16" => Some("AI"),
            "concur" | "channel" | "whichever" => Some("MCPS"),
            "para" if matches!(prev, Tok::Word(f) if f == "for") => Some("MCPS"),
            "extern" | "layout" | "C" if dot_next => Some("CNC"),
            "points" | "graphemes" if matches!(prev, Tok::Int(_)) => Some("unicode"),
            _ => None,
        },
        // a limb's word after the base's: `to.%d64`, `incl.%C`, `inline.%MSL`, `char.%point`, `str.%utf16`
        Some(last) => match (last, w) {
            ("to", "d32" | "d64" | "d128") => Some("Money"),
            ("to", "i128" | "u128" | "f128") => Some("WSMW"),
            ("to", "brain16") => Some("AI"),
            ("incl" | "inline", "C") => Some("CNC"),
            ("inline", "MSL") => Some("Metal"),
            ("char", "point" | "grapheme") => Some("unicode"),
            ("str", "utf16" | "utf32") => Some("unicode"),
            _ => None,
        },
    }
}

/// An edit for `oracle --mark-limbs`: at this character, take out `remove` characters and put `insert`.
pub struct Edit { pub at: usize, pub remove: usize, pub insert: String }

/// Checks the marks and takes them out; with `fix`, gathers the edits that would make the file right instead of
/// refusing it. Returns (line, column, message) for the first fault when not fixing.
pub fn check(toks: &mut Vec<Token>, fix: Option<&mut Vec<Edit>>) -> Result<(), (usize, usize, String)> {
    let mut edits = fix;
    // `%[d32]` was the percent type's symbol (2026-10-17); `%` is the limb mark now (owner, 2026-10-26)
    for i in 0..toks.len().saturating_sub(1) {
        if matches!(&toks[i].tok, Tok::Op(o) if o == "%") && toks[i + 1].tok == Tok::LBracket && toks[i + 1].at == toks[i].at + 1 {
            match edits.as_deref_mut() {
                Some(e) => e.push(Edit { at: toks[i].at, remove: 1, insert: "percent".into() }),
                None => return Err((toks[i].line, toks[i].col, "`%[…]` is an old spelling: a percent type is `percent[d32]`; `%` marks a limb's word since 2026-10-26 (owner)".into())),
            }
        }
    }
    // the marks, as a flag on the word after each
    let mut out: Vec<Token> = Vec::with_capacity(toks.len());
    let mut marked: Vec<Option<usize>> = Vec::with_capacity(toks.len());
    let mut pending: Option<usize> = None;
    for t in toks.drain(..) {
        if t.tok == Tok::Mark {
            if pending.is_some() { return Err((t.line, t.col, "two `%` in a row: a word takes one mark".into())); }
            pending = Some(t.at);
            continue;
        }
        if let Some(at) = pending.take() {
            if !matches!(t.tok, Tok::Word(_)) {
                match edits.as_deref_mut() {
                    Some(e) => e.push(Edit { at, remove: 1, insert: String::new() }),
                    None => return Err((t.line, t.col, format!("a `%` marks a limb's word, as `%d64` or `inline.%MSL`; found {}", t.tok))),
                }
                marked.push(None);
                out.push(t);
                continue;
            }
            marked.push(Some(at));
        } else {
            marked.push(None);
        }
        out.push(t);
    }
    *toks = out;
    // each path: its words, who owns each, and whether each wears a mark
    let mut i = 0;
    while i < toks.len() {
        let Tok::Word(_) = &toks[i].tok else { i += 1; continue };
        // a path starts at a word that does not follow a `.`
        if i > 0 && toks[i - 1].tok == Tok::Dot { i += 1; continue; }
        // after a kit's `::` the path is the kit's, never marked
        let kit = i > 0 && toks[i - 1].tok == Tok::ColonColon;
        let mut segs: Vec<usize> = vec![i];
        let mut j = i;
        while j + 2 < toks.len() && toks[j + 1].tok == Tok::Dot && matches!(toks[j + 2].tok, Tok::Word(_)) { j += 2; segs.push(j); }
        // a label or a field set, `channel: …`, `C = …`, names nothing of a limb's
        let opens = i == 0 || matches!(toks[i - 1].tok, Tok::LBracket | Tok::LParen | Tok::LBrace | Tok::Comma | Tok::Semi);
        let named = segs.len() == 1 && opens && matches!(toks.get(i + 1).map(|t| &t.tok), Some(Tok::Colon | Tok::Eq));
        let all: Vec<String> = segs.iter().map(|&at| match &toks[at].tok { Tok::Word(w) => w.clone(), _ => String::new() }).collect();
        let mut words: Vec<String> = Vec::new();
        let mut owner_before: Option<&'static str> = None;
        for (k, &at) in segs.iter().enumerate() {
            let Tok::Word(w) = toks[at].tok.clone() else { unreachable!() };
            let prev = if k == 0 && at > 0 { toks[at - 1].tok.clone() } else { Tok::Dot };
            let next = toks.get(at + 1).map(|t| t.tok.clone()).unwrap_or(Tok::Eof);
            let own = if kit || named { None } else { limb_of(&w, &words, &prev, &next) };
            // a word inside a limb's path is that limb's, unless it is another limb's own (`%layout.%Cpp`)
            let owner = own.or(owner_before);
            let needs = own.is_some() && own != owner_before;
            let has = marked[at];
            // the whole path as it should read, this word marked or not
            let shown = |_: &[String], _: &str, mark: bool| {
                all.iter().enumerate().map(|(n, x)| if n == k && mark { format!("%{x}") } else { x.clone() }).collect::<Vec<_>>().join(".")
            };
            match (needs, has) {
                (true, None) => match edits.as_deref_mut() {
                    Some(e) => e.push(Edit { at: toks[at].at, remove: 0, insert: "%".into() }),
                    None => return Err((toks[at].line, toks[at].col, format!("`{w}` is the {} limb's, so it is marked: `{}` (owner, 2026-10-26)", own.unwrap(), shown(&words, &w, true)))),
                },
                (false, Some(m)) => match edits.as_deref_mut() {
                    Some(e) => e.push(Edit { at: m, remove: 1, insert: String::new() }),
                    None => {
                        let why = if kit { "a kit's path is never marked".to_string() } else if let Some(l) = owner_before { format!("it is part of the {l} limb's `{}`, marked once", words.join(".")) } else { format!("`{w}` is the base's, or no limb adds it") };
                        return Err((toks[at].line, toks[at].col, format!("`%{w}`: {why}; a mark goes on a limb's own first word (owner, 2026-10-26)")));
                    }
                },
                _ => {}
            }
            owner_before = owner;
            words.push(w);
        }
        i = j + 1;
    }
    Ok(())
}
