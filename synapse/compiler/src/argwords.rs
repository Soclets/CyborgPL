//! Every argument after the first says its role (owner, 2026-10-14:
//! `list.append[lend.mut 'xs', value 2]`, `map.set[lend.mut 'm', key "ann",
//! value 5551]`). The first argument, the thing worked on, stays bare.
//!
//! Read on the tokens before the parser sees them, as the oracle reads them:
//! each listed call's words are checked and taken out, so the call reaches
//! the parser in the form it always had. A word that was the call's own
//! syntax before (`pos` on a SIMD position, `op` on fold's operator) stays,
//! and is not asked about here.

use crate::diag::{err, Diagnostic, Result};
use crate::lexer::{Tok, Token};

/// One listed call: its path, the word each argument after the first says
/// (and whether the word is taken out here), and the sentence's example.
struct Call {
    path: &'static [&'static str],
    words: &'static [(&'static str, bool)],
    example: &'static str,
}

const CALLS: &[Call] = &[
    Call { path: &["list", "append"], words: &[("value", true)], example: "list.append[lend.mut 'xs', value 2]" },
    Call { path: &["map", "get"], words: &[("key", true)], example: "map.get[lend 'm', key \"ann\"]" },
    Call { path: &["map", "has"], words: &[("key", true)], example: "map.has[lend 'm', key \"ann\"]" },
    Call { path: &["map", "remove"], words: &[("key", true)], example: "map.remove[lend.mut 'm', key \"ann\"]" },
    Call { path: &["map", "set"], words: &[("key", true), ("value", true)], example: "map.set[lend.mut 'm', key \"ann\", value 5551]" },
    Call { path: &["str", "contains"], words: &[("part", true)], example: "str.contains[lend 's', part \"ab\"]" },
    Call { path: &["str", "starts_with"], words: &[("part", true)], example: "str.starts_with[lend 's', part \"ab\"]" },
    Call { path: &["str", "ends_with"], words: &[("part", true)], example: "str.ends_with[lend 's', part \"ab\"]" },
    Call { path: &["str", "find", "byte"], words: &[("part", true)], example: "str.find.byte[lend 's', part \"x\"]" },
    Call { path: &["str", "find", "point"], words: &[("part", true)], example: "str.find.point[lend 's', part \"x\"]" },
    Call { path: &["str", "find", "grapheme"], words: &[("part", true)], example: "str.find.grapheme[lend 's', part \"x\"]" },
    Call { path: &["str", "split"], words: &[("at", true)], example: "str.split[lend 's', at \",\"]" },
    Call { path: &["str", "join"], words: &[("between", true)], example: "str.join[lend 'parts', between \", \"]" },
    Call { path: &["str", "replace"], words: &[("from", true), ("to", true)], example: "str.replace[lend 's', from \"a\", to \"b\"]" },
    Call { path: &["channel", "send"], words: &[("value", true)], example: "channel.send[lend 'ch', value 5]" },
    Call { path: &["ptr", "at"], words: &[("by", true)], example: "ptr.at['p', by 3]" },
    Call { path: &["ptr", "write"], words: &[("value", true)], example: "ptr.write['p', value 65]" },
    Call { path: &["simd", "load", "some"], words: &[("pos", false), ("count", true)], example: "simd.load.some[lend 'xs', pos 1, count 3]" },
    Call { path: &["simd", "store"], words: &[("pos", false), ("value", true)], example: "simd.store[lend.mut 'xs', pos 1, value 'v']" },
    Call { path: &["simd", "store", "some"], words: &[("pos", false), ("count", true), ("value", true)], example: "simd.store.some[lend.mut 'xs', pos 1, count 3, value 'v']" },
    Call { path: &["fold"], words: &[("from", true), ("op", false)], example: "fold[lend 'xs', from 0, op plus]" },
];

/// The listed call whose path starts at token `i`, and the index of its `[`.
/// A call after a `.` or a Neuron's `::` is someone's own (`my.fold[…]`),
/// not the language's or cord's. The longest path wins: `simd.store.some` over `simd.store`.
fn call_at(toks: &[Token], i: usize) -> Option<(&'static Call, usize)> {
    // After `fn`, it is a declaration's parameters (`fn fold [lent 'xs': …]`).
    // `cord::str.split[…]` is cord's (the lexer reads `cord` as `std`).
    let cord = i >= 2 && matches!(toks[i - 1].tok, Tok::ColonColon) && matches!(&toks[i - 2].tok, Tok::Word(w) if w == "std");
    if i > 0 && !cord && (matches!(toks[i - 1].tok, Tok::Dot | Tok::ColonColon) || matches!(&toks[i - 1].tok, Tok::Word(w) if w == "fn")) {
        return None;
    }
    let word = |k: usize| match toks.get(k).map(|t| &t.tok) {
        Some(Tok::Word(w)) => Some(w.as_str()),
        _ => None,
    };
    let mut best: Option<(&'static Call, usize)> = None;
    'calls: for c in CALLS {
        let mut k = i;
        for (n, seg) in c.path.iter().enumerate() {
            if n > 0 {
                if !matches!(toks.get(k).map(|t| &t.tok), Some(Tok::Dot)) {
                    continue 'calls;
                }
                k += 1;
            }
            if word(k) != Some(*seg) {
                continue 'calls;
            }
            k += 1;
        }
        if matches!(toks.get(k).map(|t| &t.tok), Some(Tok::LBracket)) && best.map_or(true, |(b, _)| b.path.len() < c.path.len()) {
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
        for (n, &(word, take)) in call.words.iter().enumerate() {
            let Some(&at) = starts.get(n + 1) else { break };
            if at >= close {
                break;
            }
            if !take {
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
