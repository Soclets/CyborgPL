//! Every argument after the first says its role (owner, 2026-10-14:
//! `list.append[lend.mut 'xs', value 2]`, `map.set[lend.mut 'm', key "ann",
//! value 5551]`). The first argument, the thing worked on, stays bare.
//!
//! The calls and their words are `spellings.zv`'s `calls`. Read on the
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
        if matches!(toks.get(k).map(|t| &t.tok), Some(Tok::LBracket)) && best.map_or(true, |(b, _)| b.name.len() < c.name.len()) {
            best = Some((c, k));
        }
    }
    best
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
            let says = matches!(&toks[at].tok, Tok::Word(w) if w == word) && at + 1 < close && !matches!(toks[at + 1].tok, Tok::Comma);
            if !says {
                return err(Diagnostic::error(format!("each argument after the first says what it is: `{}` (owner, 2026-10-14)", call.example)).at(toks[at].span));
            }
            drop[at] = true;
        }
    }
    Ok(toks.iter().zip(drop).filter(|(_, d)| !d).map(|(t, _)| t.clone()).collect())
}
