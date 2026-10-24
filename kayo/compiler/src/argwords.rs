//! Every argument but the thing worked on says its role (owner, 2026-10-14:
//! `list.append[lend.mut 'xs', value: 2]`, `map.set[lend.mut 'm', key:
//! "ann", value: 5551]`; the `:` since 2026-10-20). The thing worked on
//! stays bare; any other says its word, the only one too (2026-10-23:
//! `std::exit(code: 3)`).
//!
//! The calls and their words are `spellings.cyr`'s `calls`. Read on the
//! tokens before the parser sees them, as the oracle reads them: each listed
//! call's words are checked and taken out, so the call reaches the parser in
//! the form it always had. A position the table leaves "" (the first, or
//! one with syntax of its own, as SIMD's `pos`) is not asked about here.

use crate::diag::{err, Diagnostic, Result};
use crate::lexer::{Tok, Token};
use crate::spellings::{calls, Call};

/// The listed call whose path starts at token `i`, and the index of its `[`.
/// A call after a `.` or a Neuron's `::` is someone's own (`my.fold[…]`),
/// not the language's or cord's. The longest path wins: `simd.store.some` over `simd.store`.
fn call_at(toks: &[Token], i: usize) -> Option<(&'static Call, usize)> {
    // `cord::str.split[…]` is cord's (the lexer reads `cord` as `std`).
    let cord = i >= 2 && matches!(toks[i - 1].tok, Tok::ColonColon) && matches!(&toks[i - 2].tok, Tok::Word(w) if w == "std");
    // After `fn`, it is a declaration's parameters (`fn str.split [lent 's':
    // …]` in cord).
    if i > 0 && !cord && (matches!(toks[i - 1].tok, Tok::Dot | Tok::ColonColon) || matches!(&toks[i - 1].tok, Tok::Word(w) if w == "fn")) {
        return None;
    }
    let word = |k: usize| match toks.get(k).map(|t| &t.tok) {
        Some(Tok::Word(w)) => Some(w.as_str()),
        _ => None,
    };
    let mut best: Option<(&'static Call, usize)> = None;
    'calls: for c in calls() {
        let mut k = i;
        for (n, seg) in c.name.split('.').enumerate() {
            if n > 0 {
                if !matches!(toks.get(k).map(|t| &t.tok), Some(Tok::Dot)) {
                    continue 'calls;
                }
                k += 1;
            }
            if word(k) != Some(seg) {
                continue 'calls;
            }
            k += 1;
        }
        if matches!(toks.get(k).map(|t| &t.tok), Some(Tok::LBracket | Tok::LParen)) && best.map_or(true, |(b, _)| b.name.len() < c.name.len()) {
            best = Some((c, k));
        }
    }
    best
}

/// A word inside `[ ]` takes `:` before its value (owner, 2026-10-20):
/// `value: 2`, `pos: 3`, `from: pos 2`, `i32[min: 0, max: 100]`, `for 'i'
/// [from: 1, to: 10, by: 1]`. Each such word is marked `labelled` and its
/// `:` taken out, so the parser reads the words as it always has and asks
/// only whether each was labelled. Only where the innermost open bracket is
/// `[`: a struct's `{ x: i32 }` is a field's type.
pub fn labels(toks: &[Token]) -> Vec<Token> {
    let mut out: Vec<Token> = Vec::with_capacity(toks.len());
    let mut open: Vec<Tok> = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        let t = &toks[i];
        match t.tok {
            Tok::LBracket | Tok::LParen | Tok::LBrace => open.push(t.tok.clone()),
            Tok::RBracket | Tok::RParen | Tok::RBrace => {
                open.pop();
            }
            _ => {}
        }
        // A label starts an argument, right after `[` or `,`: `<T: any>`
        // is a generic's bound, `5: u8` a typed literal.
        // A call's `( )` holds labels as `[ ]` did (owner, 2026-10-22:
        // `std::exit(code: 3)`).
        let label = matches!(open.last(), Some(Tok::LBracket | Tok::LParen))
            && i > 0
            && matches!(toks[i - 1].tok, Tok::LBracket | Tok::LParen | Tok::Comma)
            && matches!(&t.tok, Tok::Word(w) if w != "true" && w != "false")
            && matches!(toks.get(i + 1).map(|n| &n.tok), Some(Tok::Colon));
        let mut t = t.clone();
        if label {
            t.labelled = true;
            out.push(t);
            i += 2;
            continue;
        }
        out.push(t);
        i += 1;
    }
    out
}

/// What a bare word inside `[ ]` is told (owner, 2026-10-20).
pub fn needs_colon(word: &str) -> String {
    format!("inside `[ ]` a word takes `:` before its value: `{}: …` (owner, 2026-10-20)", word)
}

/// The tokens with each listed call's words checked and taken out.
pub fn strip(toks: &[Token]) -> Result<Vec<Token>> {
    let mut drop = vec![false; toks.len()];
    for i in 0..toks.len() {
        let Some((call, open)) = call_at(toks, i) else { continue };
        // The arguments' first tokens, split at the commas of this bracket.
        let mut starts = vec![open + 1];
        let mut depth = 0usize;
        let mut k = open + 1;
        while k < toks.len() {
            match toks[k].tok {
                Tok::LBracket | Tok::LParen | Tok::LBrace => depth += 1,
                Tok::RBracket | Tok::RParen | Tok::RBrace if depth == 0 => break,
                Tok::RBracket | Tok::RParen | Tok::RBrace => depth -= 1,
                Tok::Comma if depth == 0 => starts.push(k + 1),
                _ => {}
            }
            k += 1;
        }
        let close = k;
        for (n, word) in call.words.iter().enumerate() {
            let Some(&at) = starts.get(n) else { break };
            if at >= close {
                break;
            }
            if word.is_empty() {
                continue;
            }
            let named = matches!(&toks[at].tok, Tok::Word(w) if w == word) && at + 1 < close && !matches!(toks[at + 1].tok, Tok::Comma);
            // `round.near['x', decimals 2]` (owner, 2026-10-20: `places:`).
            if *word == "places" && matches!(&toks[at].tok, Tok::Word(w) if w == "decimals") {
                let said = crate::spellings::old_word("decimals", Some("special")).map(|o| o.dated()).unwrap_or_else(|| "`decimals` is an old spelling: write `places:` (owner, 2026-10-20)".into());
                return err(Diagnostic::error(said).named("old_spelling").at(toks[at].span));
            }
            if named && !toks[at].labelled {
                return err(Diagnostic::error(needs_colon(word)).named("label_needs_colon").at(toks[at].span));
            }
            if !named {
                // The only argument says its word too when it is not the
                // thing worked on (owner, 2026-10-23: "`(3)` fucking
                // doesn't").
                if n == 0 {
                    return err(Diagnostic::error(format!("only the thing worked on goes bare; this argument says what it is: `{}` (owner, 2026-10-23)", call.example)).named("argument_label_missing").at(toks[at].span));
                }
                return err(Diagnostic::error(format!("each argument after the first says what it is: `{}` (owner, 2026-10-14)", call.example)).named("argument_label_missing").at(toks[at].span));
            }
            drop[at] = true;
        }
    }
    Ok(toks.iter().zip(drop).filter(|(_, d)| !d).map(|(t, _)| t.clone()).collect())
}

