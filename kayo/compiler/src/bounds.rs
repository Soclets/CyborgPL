//! `coll.bound` (owner, 2026-10-25; `design/collections.md`): named bounded
//! types. `coll.bound quantity { quantity = i32[min: 1, max: 99];
//! quantity.small = i32[min: 1, max: 9]; }` names two types, and a use of a
//! name *is* that type, not a new one, so values move between them freely.
//! Read, as a `use` is, as a rewrite of the file's tokens before parsing: each
//! block is taken out and each member's name written as its type wherever it
//! starts a path, so a type anywhere in the file may name one. Every line
//! writes its full name; only a bounded type, one with a limit after it, is
//! named. A `pub` one is used in its own kit; another kit's use (`kit::q`) is not
//! built, as in the oracle.

use crate::diag::{err, Diagnostic, Result};
use crate::lexer::{Tok, Token};

/// A path's end at `i`: words joined by `.`.
fn path_end(toks: &[Token], mut i: usize) -> usize {
    if !matches!(toks.get(i).map(|t| &t.tok), Some(Tok::Word(_))) {
        return i;
    }
    i += 1;
    while matches!(toks.get(i).map(|t| &t.tok), Some(Tok::Dot)) && matches!(toks.get(i + 1).map(|t| &t.tok), Some(Tok::Word(_))) {
        i += 2;
    }
    i
}

fn spelled(toks: &[Token]) -> String {
    toks.iter().map(|t| match &t.tok {
        Tok::Word(w) => w.clone(),
        Tok::Dot => ".".into(),
        _ => String::new(),
    }).collect()
}

/// Whether a type's tokens put a limit on it: `min:`, `max:`, `max_places:`
/// or `min_places:` inside a bracket.
fn bounded(ty: &[Token]) -> bool {
    let mut depth = 0i32;
    for (k, t) in ty.iter().enumerate() {
        match &t.tok {
            Tok::LBracket => depth += 1,
            Tok::RBracket => depth -= 1,
            Tok::Word(w) if depth > 0 && matches!(w.as_str(), "min" | "max" | "max_places" | "min_places") && matches!(ty.get(k + 1).map(|t| &t.tok), Some(Tok::Colon)) => return true,
            _ => {}
        }
    }
    false
}

pub fn rewrite(toks: &[Token]) -> Result<Vec<Token>> {
    let is_word = |i: usize, w: &str| matches!(toks.get(i).map(|t| &t.tok), Some(Tok::Word(x)) if x == w);
    if !(0..toks.len()).any(|i| is_word(i, "coll") && matches!(toks.get(i + 1).map(|t| &t.tok), Some(Tok::Dot)) && is_word(i + 2, "bound")) {
        return Ok(toks.to_vec());
    }
    // Each block: its members, each a name's tokens and its type's.
    let mut members: Vec<(Vec<Token>, Vec<Token>)> = Vec::new();
    let mut kept: Vec<Token> = Vec::with_capacity(toks.len());
    let mut i = 0;
    while i < toks.len() {
        let starts = is_word(i, "coll") && matches!(toks.get(i + 1).map(|t| &t.tok), Some(Tok::Dot)) && is_word(i + 2, "bound");
        if !starts {
            kept.push(toks[i].clone());
            i += 1;
            continue;
        }
        let at = toks[i].span;
        // `pub` is read and used within its own kit; another kit's use,
        // `kit::q`, is not built, as in the oracle (design claude, 2026-10-26).
        if kept.last().is_some_and(|t| matches!(&t.tok, Tok::Word(w) if w == "pub")) {
            kept.pop();
        }
        let Some(Tok::Word(name)) = toks.get(i + 3).map(|t| t.tok.clone()) else {
            return err(Diagnostic::error("a `coll.bound` says its name, then its members in `{ }`: `coll.bound quantity { quantity = i32[min: 1, max: 99]; }`").named("collection_form").at(at));
        };
        if !matches!(toks.get(i + 4).map(|t| &t.tok), Some(Tok::LBrace)) {
            return err(Diagnostic::error(format!("`coll.bound {}` holds its members in `{{ }}`", name)).named("collection_form").at(toks[i + 3].span));
        }
        let mut k = i + 5;
        loop {
            match toks.get(k).map(|t| &t.tok) {
                Some(Tok::RBrace) => {
                    k += 1;
                    break;
                }
                None | Some(Tok::Eof) => return err(Diagnostic::error(format!("`coll.bound {}`'s `{{` is never closed", name)).named("unclosed_block").at(at)),
                _ => {}
            }
            let n1 = path_end(toks, k);
            let member = toks[k..n1].to_vec();
            if member.is_empty() || !matches!(toks.get(n1).map(|t| &t.tok), Some(Tok::Op(o)) if o == "=") {
                return err(Diagnostic::error(format!("a member of `coll.bound {}` is its full name, `=`, and a bounded type, then `;`: `{}.small = i32[min: 1, max: 9];`", name, name)).named("collection_form").at(toks.get(k).map_or(at, |t| t.span)));
            }
            let full = spelled(&member);
            if !(full == name || full.starts_with(&format!("{}.", name))) {
                return err(Diagnostic::error(format!("a member writes the collection's name in front: `{}.{} = …;`", name, full)).named("collection_member_unprefixed").at(member[0].span).fix_edit(member[0].span, crate::diag::Edit::Insert, format!("{}.", name)));
            }
            if members.iter().any(|(m, _)| spelled(m) == full) {
                return err(Diagnostic::error(format!("`{}` is named twice: a `coll.bound` names each type once", full)).named("defined_twice").at(member[0].span));
            }
            // The type, to its `;` outside every bracket.
            let mut t1 = n1 + 1;
            let mut depth = 0i32;
            while let Some(t) = toks.get(t1) {
                match &t.tok {
                    Tok::LBracket | Tok::LParen | Tok::LBrace => depth += 1,
                    Tok::RBracket | Tok::RParen | Tok::RBrace if depth > 0 => depth -= 1,
                    Tok::Semi if depth == 0 => break,
                    Tok::RBrace | Tok::Eof => return err(Diagnostic::error(format!("`{} = …` ends with `;`", full)).named("semicolon_missing").at(t.span)),
                    _ => {}
                }
                t1 += 1;
            }
            let ty = toks[n1 + 1..t1].to_vec();
            if !bounded(&ty) {
                return err(Diagnostic::error(format!("`{}` limits nothing: a `coll.bound` names a bounded type, a type with a limit after it (`i32[min: 1, max: 99]`, `str.utf8[max: 40 graphemes]`, `std::list[i32][max: 3]`)", full)).named("bound_type_unbounded").at(member[0].span));
            }
            members.push((member, ty));
            k = t1 + 1;
        }
        i = k;
    }
    // Longest names first: `quantity.small` before `quantity`.
    members.sort_by_key(|(m, _)| std::cmp::Reverse(m.len()));
    let mut out: Vec<Token> = Vec::with_capacity(kept.len());
    let mut k = 0;
    while k < kept.len() {
        let t = &kept[k];
        let head = k == 0 || !matches!(kept[k - 1].tok, Tok::Dot | Tok::ColonColon);
        // After `:` or `->` a name is a type, whatever follows it.
        let typed = k > 0 && (matches!(kept[k - 1].tok, Tok::Colon) || matches!(&kept[k - 1].tok, Tok::Op(o) if o == "->"));
        let hit = head.then(|| members.iter().find(|(m, _)| {
            let end = k + m.len();
            kept.get(k..end).is_some_and(|w| w.iter().zip(m).all(|(a, b)| a.tok == b.tok))
                // A longer path is not the type; nor, unless it follows `:`,
                // a field of that name, `quantity = 3` in a struct, a label.
                && !matches!(kept.get(end).map(|t| &t.tok), Some(Tok::Dot | Tok::ColonColon))
                && (typed || !(matches!(kept.get(end).map(|t| &t.tok), Some(Tok::Colon)) || matches!(kept.get(end).map(|t| &t.tok), Some(Tok::Op(o)) if o == "=")))
        })).flatten();
        if let Some((m, ty)) = hit {
            for p in ty {
                let mut p = p.clone();
                p.span = t.span;
                p.line_first = false;
                out.push(p);
            }
            if let Some(first) = out.len().checked_sub(ty.len()) {
                out[first].line_first = t.line_first;
            }
            k += m.len();
            continue;
        }
        out.push(t.clone());
        k += 1;
    }
    Ok(out)
}
