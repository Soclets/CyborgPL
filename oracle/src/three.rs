//! Redesign three (owner, 2026-10-27/28: "done, build it"): the one spelling. A block closes `;kind name`, nothing wears
//! `!`, the operators are symbols (`+ - x ^ == != < > <= >= & && | || ++`, `-5` a negative), `?` / `??` are `otherwise
//! fail` / `otherwise <value>` (`and` / `or` keep their words, owner 2026-10-28), `*` marks a jumping handler where it is used, `change $x += …`, `std::print` /
//! `std::println`, and the number types are `i32` … `%d64` `%bf16`. This pass reads it and hands the rest of the front
//! redesign two's tokens, which it already checks; every redesign-two spelling is refused here by name.

use crate::lexer::{Tok, Token};
use crate::redesign::Fault;

fn word(w: &str, like: &Token) -> Token { Token { tok: Tok::Word(w.to_string()), line: like.line, col: like.col, mtm: None, paren: false, at: like.at, end: like.end } }
fn like(tok: Tok, l: &Token) -> Token { Token { tok, line: l.line, col: l.col, mtm: None, paren: false, at: l.at, end: l.end } }
/// a token the reader of redesign two counts as written `width` characters long (`lt` for `<`, `not_eq` for `!=`)
fn wide(tok: Tok, l: &Token, width: usize) -> Token { let mut t = like(tok, l); if t.at != usize::MAX { t.end = t.at + width; } t }

fn old(t: &Token, written: &str, write: &str) -> Fault {
    (t.line, t.col, format!("`{written}` is an old spelling: write `{write}` (owner, redesign three)"))
}

const OP_WORDS: &[&str] = &["x", "times", "plus", "minus", "pow", "concat", "neg", "and", "or", "not", "xor", "shl", "shr", "div", "mod", "is", "otherwise"];

/// Whether what has been read so far ends with a value, so an operator after it joins two (`-` is `minus`, `x` is
/// `times`): an operator word, or one narrowed (`div.near`), ends none
fn ends_value(out: &[Token]) -> bool {
    let n = out.len();
    if let Some(Tok::Word(w)) = out.last().map(|t| &t.tok) {
        if n >= 2 && out[n - 2].tok == Tok::Dot {
            // a path: a field (`$a.x`) is a value, a narrowed operator (`div.trunc.wrap`) none
            let mut k = n - 1;
            while k >= 2 && out[k - 1].tok == Tok::Dot && matches!(out[k - 2].tok, Tok::Word(_) | Tok::Var(_)) { k -= 2; }
            return !matches!(&out[k].tok, Tok::Word(o) if OP_WORDS.contains(&o.as_str()));
        }
        if OP_WORDS.contains(&w.as_str()) { return false; }
    }
    value_end(out.last())
}

/// A token after which an operator joins two values
fn value_end(t: Option<&Token>) -> bool {
    match t.map(|t| &t.tok) {
        Some(Tok::Var(_) | Tok::Int(_) | Tok::BigInt(_) | Tok::Bin(..) | Tok::Text(..) | Tok::Char(_) | Tok::PercentLit(..) | Tok::RParen | Tok::RBracket | Tok::Newline) => true,
        Some(Tok::Word(w)) => w == "true" || w == "false" || !crate::spellings::is_reserved(w),
        _ => false,
    }
}

/// The number types' short names (owner, 2026-10-27: "yes, make it the exception"): redesign two's long name, and
/// whether a limb adds it (so it wears the mark)
fn number(w: &str) -> Option<(&'static str, &'static str, String, bool)> {
    let (kind, sub, rest) = if let Some(r) = w.strip_prefix("bf") { ("float", "brain", r) }
        else if let Some(r) = w.strip_prefix('i') { ("int", "sign", r) } else if let Some(r) = w.strip_prefix('u') { ("int", "unsign", r) }
        else if let Some(r) = w.strip_prefix('f') { ("float", "bin", r) } else if let Some(r) = w.strip_prefix('d') { ("float", "deci", r) } else { return None };
    if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) { return None; }
    let ok = match (kind, sub) { ("int", _) => matches!(rest, "8" | "16" | "32" | "64" | "128"), ("float", "bin") => matches!(rest, "16" | "32" | "64" | "128"), ("float", "brain") => rest == "16", _ => matches!(rest, "32" | "64" | "128") };
    if !ok { return None; }
    let limb = rest == "128" || sub == "deci" || sub == "brain";
    Some((kind, sub, rest.to_string(), limb))
}

/// The tokens of redesign two's name for a short number type, the limb's mark where it went there
fn number_tokens(kind: &str, sub: &str, width: &str, limb: bool, t: &Token, out: &mut Vec<Token>) {
    out.push(word(kind, t));
    out.push(like(Tok::Dot, t));
    // the mark on the part the limb adds (2026-10-26): `float.%deci.64`, `float.%brain.16`, `int.sign.%128`
    let on_sub = sub == "deci" || sub == "brain";
    if limb && on_sub { out.push(like(Tok::Mark, t)); }
    out.push(word(sub, t));
    out.push(like(Tok::Dot, t));
    if limb && !on_sub { out.push(like(Tok::Mark, t)); }
    out.push(word(width, t));
}

pub fn from_three(toks: Vec<Token>) -> Result<Vec<Token>, Fault> {
    let toks = prints(toks)?;
    let n = toks.len();
    let mut out: Vec<Token> = Vec::with_capacity(n + n / 4);
    let tok = |i: usize| toks.get(i).map(|t| &t.tok).unwrap_or(&Tok::Eof);
    let is_word = |i: usize, w: &str| matches!(toks.get(i).map(|t| &t.tok), Some(Tok::Word(x)) if x == w);
    let mut i = 0;
    while i < n {
        let t = &toks[i];
        let starts_line = i == 0 || toks[i - 1].line < t.line;
        let prev = if i > 0 { Some(&toks[i - 1]) } else { None };
        let after_dot = matches!(prev.map(|p| &p.tok), Some(Tok::Dot)) || (matches!(prev.map(|p| &p.tok), Some(Tok::Mark)) && i >= 2 && toks[i - 2].tok == Tok::Dot);
        let after_kit = matches!(prev.map(|p| &p.tok), Some(Tok::ColonColon));
        // redesign two's marks and closers, refused by name
        if t.tok == Tok::Attention {
            let w = match tok(i + 1) { Tok::Word(w) => w.clone(), _ => String::new() };
            let marked = matches!(w.as_str(), "MAIN" | "no_check" | "NO_FUCKING_CHECKS" | "case" | "start" | "no_skip");
            // `!` before a value is C's and Rust's negation, never a mark: name `not`
            let fix = if w == "end" { "a block closes `;kind name`, `;MAIN`".to_string() } else if marked { format!("`{w}`, with no `!`") } else { format!("`not {}` for a negation", match tok(i + 1) { Tok::Word(w) => w.clone(), Tok::Var(v) => format!("${v}"), _ => "…".to_string() }) };
            return Err((t.line, t.col, format!("`!` is gone (owner, 2026-10-27: \"I'm feeling like removing that `!` meaning\"): write {fix}")));
        }
        if starts_line && is_word(i, "end") && is_word(i + 1, "of") {
            let mut k = i + 2; let mut what = String::new();
            while k < n && toks[k].line == t.line && !matches!(toks[k].tok, Tok::Semi | Tok::Comma | Tok::RParen) { what.push_str(&shown(&toks[k].tok)); if !matches!(toks[k].tok, Tok::Dot | Tok::Mark) && !matches!(toks.get(k + 1).map(|x| &x.tok), Some(Tok::Dot)) { what.push(' '); } k += 1; }
            return Err(old(t, &format!("end of {}", what.trim()), &format!(";{}", what.trim())));
        }
        // a closer, `;fn total` at a line's start: redesign two's `end of fn total`, `!end of MAIN`
        if starts_line && t.tok == Tok::Semi && toks.get(i + 1).is_some_and(|x| x.line == t.line && matches!(x.tok, Tok::Word(_) | Tok::Mark)) {
            let kind = match tok(i + 1) { Tok::Word(w) => w.clone(), _ => String::new() };
            if matches!(kind.as_str(), "MAIN" | "no_check" | "NO_FUCKING_CHECKS") { out.push(like(Tok::Attention, t)); }
            out.push(word("end", t));
            out.push(word("of", t));
            i += 1;
            continue;
        }
        match &t.tok {
            Tok::Word(w) => {
                let w = w.as_str();
                // `MAIN:` and every `no_check` are written plain; redesign two's pipeline reads them marked
                if w == "MAIN" && matches!(tok(i + 1), Tok::Colon | Tok::LBracket) && !matches!(prev.map(|p| &p.tok), Some(Tok::Word(f)) if f == "fn" || f == "proc") && !matches!(out.last().map(|p| &p.tok), Some(Tok::Word(f)) if f == "of") {
                    out.push(like(Tok::Attention, t)); out.push(t.clone()); i += 1; continue;
                }
                if (w == "no_check" || w == "NO_FUCKING_CHECKS") && !matches!(out.last().map(|p| &p.tok), Some(Tok::Word(o)) if o == "of") {
                    out.push(like(Tok::Attention, t)); out.push(t.clone()); i += 1; continue;
                }
                // the number types' short names
                if !after_kit && !(after_dot && i >= 2 && matches!(toks[i - 2].tok, Tok::Var(_))) {
                    if let Some((kind, sub, width, limb)) = number(w) {
                        if matches!(prev.map(|p| &p.tok), Some(Tok::Word(d)) if matches!(d.as_str(), "rec" | "union" | "enum" | "sig" | "set" | "fn" | "proc" | "op" | "module")) {
                            return Err((t.line, t.col, format!("`{w}` is one of the language's number types, so it cannot name a {}", shown(&prev.unwrap().tok))));
                        }
                        let marked = matches!(out.last().map(|x| &x.tok), Some(Tok::Mark));
                        let shown = format!("{w}");
                        if limb && !marked { return Err((t.line, t.col, format!("`{shown}` is a limb's, so it is marked: `%{shown}` (owner, 2026-10-26)"))); }
                        if !limb && marked { return Err((t.line, t.col, format!("`%{shown}` is the base's, so it takes no mark: `{shown}` (owner, 2026-10-26)"))); }
                        if marked { out.pop(); }
                        number_tokens(kind, sub, &width, limb, t, &mut out);
                        i += 1;
                        continue;
                    }
                    // `%not_std.i24`: an integer of any width, `int.sign.%not_std.24` inside
                    if w == "not_std" && tok(i + 1) == &Tok::Dot {
                        if let Tok::Word(x) = tok(i + 2) {
                            if let Some((_, _, _, limb)) = number(x) {
                                return Err((toks[i + 2].line, toks[i + 2].col, format!("`{x}` is standard: write `{}{x}`", if limb { "%" } else { "" })));
                            }
                            if let Some(d) = x.strip_prefix('i').or_else(|| x.strip_prefix('u')).filter(|d| !d.is_empty() && d.chars().all(|c| c.is_ascii_digit())) {
                                let marked = matches!(out.last().map(|x| &x.tok), Some(Tok::Mark));
                                if !marked { return Err((t.line, t.col, format!("an integer of any width is a limb's, so it is marked: `%not_std.{x}` (owner, 2026-10-26)"))); }
                                out.pop();
                                out.push(word("int", t)); out.push(like(Tok::Dot, t)); out.push(word(if x.starts_with('i') { "sign" } else { "unsign" }, t)); out.push(like(Tok::Dot, t));
                                out.push(like(Tok::Mark, t)); out.push(word("not_std", t)); out.push(like(Tok::Dot, t)); out.push(word(d, &toks[i + 2]));
                                i += 3;
                                continue;
                            }
                        }
                    }
                }
                // redesign two's long number names
                if (w == "int" || w == "float") && !after_dot && tok(i + 1) == &Tok::Dot && matches!(tok(i + 2), Tok::Word(s) if matches!(s.as_str(), "sign" | "unsign" | "bin" | "deci" | "brain")) || (w == "float" && tok(i + 1) == &Tok::Dot && tok(i + 2) == &Tok::Mark) {
                    let mut k = i; let mut s = String::new();
                    while k < n && matches!(toks[k].tok, Tok::Word(_) | Tok::Dot | Tok::Mark) { s.push_str(&shown(&toks[k].tok)); k += 1; if k < n && !matches!(toks[k].tok, Tok::Dot) && !matches!(toks[k - 1].tok, Tok::Dot | Tok::Mark) { break; } }
                    return Err(old(t, &s, &short_number(&s)));
                }
                // `x` is multiplication (owner, 2026-10-27: "`x` instead of (inof) `*`"); after a `.` it is a field's name
                if w == "x" && tok(i + 1) == &Tok::Eq && toks[..i].iter().rev().take_while(|p| p.tok != Tok::Semi).any(|p| matches!(&p.tok, Tok::Word(c) if c == "change")) {
                    return Err((t.line, t.col, "`x=` is an old spelling: write `change $x to $x x …;` (owner, 2026-10-28: symbols that break the reading go)".into()));
                }
                if w == "x" && !after_dot && !matches!(tok(i + 1), Tok::Colon | Tok::Eq | Tok::Walrus) && !matches!(prev.map(|p| &p.tok), Some(Tok::Op(a)) if a == "@") {
                    // used between two values, or named as an operator: `op x`, `level 3 [x, dot]`
                    let named = matches!(prev.map(|p| &p.tok), Some(Tok::Word(o)) if o == "op") || matches!(prev.map(|p| &p.tok), Some(Tok::LBracket | Tok::Comma)) && matches!(tok(i + 1), Tok::Comma | Tok::RBracket);
                    if ends_value(&out) || named { out.push(word("times", t)); i += 1; continue; }
                    // something may be named `x` and reached by its path, `my.x(…)`, as `my.true(…)` is (owner, 2026-10-28: "yes")
                    let declares = matches!(prev.map(|p| &p.tok), Some(Tok::Word(d)) if matches!(d.as_str(), "fn" | "proc" | "rec" | "tag" | "union" | "enum" | "sig" | "set" | "module" | "bounded" | "handler"));
                    if declares { out.push(t.clone()); i += 1; continue; }
                    return Err((t.line, t.col, "a bare `x` is multiplication: a function or type named `x` is reached by its path, `my.x(…)` (owner, 2026-10-28)".into()));
                }
                // the base's operators are not std's, so they take no `std::` (owner, 2026-10-28: "`plus` isn't even
                // `std`"); std's own `pow` keeps its path
                if after_kit && i >= 2 && matches!(&toks[i - 2].tok, Tok::Word(k) if k == "std")
                    && matches!(w, "plus" | "minus" | "times" | "div" | "mod" | "and" | "or" | "xor" | "shl" | "shr" | "concat" | "not" | "neg") {
                    let narrowed = tok(i + 1) == &Tok::Dot;
                    let write = match w { "plus" if !narrowed => "+", "minus" if !narrowed => "-", "times" if !narrowed => "x", "neg" if !narrowed => "-5`, `-$x", _ => w };
                    let tail = if narrowed && write == w { format!("`{w}.{}`", match tok(i + 2) { Tok::Word(n) => n.as_str(), _ => "…" }) } else { format!("`{write}`") };
                    return Err((t.line, t.col, format!("`{w}` is the base's, not std's: write {tail}, without `std::` (owner, 2026-10-28: \"`plus` isn't even `std`\")")));
                }
                // redesign two's operator words, refused naming their symbol; a narrowing keeps its word (`times.wrap`)
                if !after_dot && !after_kit {
                    let narrowed = tok(i + 1) == &Tok::Dot;
                    let sym = match w {
                        "plus" if !narrowed => Some("+"), "minus" if !narrowed => Some("-"), "times" if !narrowed => Some("x"),
                        "pow" if !narrowed => Some("^"), "eq" => Some("=="), "not_eq" => Some("!="), "lt" => Some("<"), "gt" => Some(">"),
                        "at_most" => Some("<="), "at_least" => Some(">="),
                        // negation as a value keeps its name, `op neg` (owner, 2026-10-28: "keep it")
                        "neg" if !narrowed && !matches!(prev.map(|p| &p.tok), Some(Tok::Word(o)) if o == "op") => Some("-"),
                        _ => None,
                    };
                    if let Some(s) = sym { return Err(old(t, w, if w == "neg" { "-5`, `-$x" } else { s })); }
                }
                out.push(t.clone());
            }
            Tok::Op(_) if t.mtm.is_some() => out.push(t.clone()),
            Tok::Op(o) => {
                let o = o.as_str();
                let declared = matches!(prev.map(|p| &p.tok), Some(Tok::Word(w)) if w == "op" || w == "operator");
                let glued_after = toks.get(i + 1).is_some_and(|x| x.at == t.end && t.end != usize::MAX);
                let glued_before = prev.is_some_and(|p| p.end == t.at && t.at != usize::MAX);
                match o {
                    "+" => out.push(word("plus", t)),
                    // the line-end `-` joins a line to a block's opening line below it (2026-10-27)
                    "-" if toks.get(i + 1).is_some_and(|x| x.line > t.line) => out.push(t.clone()),
                    // a hyphen inside a name, `aarch64-macos`
                    "-" if glued_after && glued_before && matches!(prev.map(|p| &p.tok), Some(Tok::Word(_))) && matches!(tok(i + 1), Tok::Word(_)) => out.push(t.clone()),
                    "-" => out.push(word(if ends_value(&out) || declared || matches!(tok(i + 1), Tok::Comma | Tok::RBracket) { "minus" } else { "neg" }, t)),
                    "^" => out.push(word("pow", t)),
                    "==" => out.push(wide(Tok::Op("equal_to".into()), t, 2)),
                    "!=" => out.push(wide(Tok::Op("not_equal".into()), t, 6)),
                    // `<T>`, a type's blank, is glued; a comparison is spaced
                    "<" if glued_after && !declared => out.push(t.clone()),
                    ">" if glued_before && !declared => out.push(t.clone()),
                    "<" => out.push(wide(Tok::Op("<".into()), t, 2)),
                    ">" => out.push(wide(Tok::Op(">".into()), t, 2)),
                    "<=" => out.push(like(Tok::Op("at_most".into()), t)),
                    ">=" => out.push(like(Tok::Op("at_least".into()), t)),
                    // `?` / `??` (owner, 2026-10-27): `otherwise fail` / `otherwise <value>`
                    "?" => {
                        // C's `c ? a : b` is not CyborgPL: a `:` after the `?` on the same statement
                        let ternary = toks[i + 1..].iter().take_while(|x| x.line == t.line && !matches!(x.tok, Tok::Semi | Tok::Eof)).any(|x| x.tok == Tok::Colon);
                        if ternary { return Err((t.line, t.col, "`? :` is not CyborgPL: `?` means \"otherwise fail\", after a call that can fail; a choice between values is written with `if` (owner, 2026-10-27)".into())); }
                        out.push(word("otherwise", t)); out.push(word("fail", t));
                    }
                    "??" => out.push(word("otherwise", t)),
                    // `*` before a handler that can jump, where it is used (owner, 2026-10-27: "(b)")
                    "*" if after_dot || matches!(prev.map(|p| &p.tok), Some(Tok::Word(w)) if w == "otherwise") => out.push(like(Tok::Attention, t)),
                    "*" => return Err((t.line, t.col, "`*` is not multiplication: write `x` (owner, 2026-10-27); `*` marks a handler that can jump, where it is used: `otherwise h.*skip(…)`".into())),
                    _ => out.push(t.clone()),
                }
            }
            _ => out.push(t.clone()),
        }
        i += 1;
    }
    Ok(out)
}

/// `std::print(…)` is `std::print.shortest(…)`, and `std::println…` ends the line (owner, 2026-10-27): read as redesign
/// two's `std::print.shortest(…, newln)`, a format's line ended inside its text
fn prints(toks: Vec<Token>) -> Result<Vec<Token>, Fault> {
    let n = toks.len();
    let mut out = Vec::with_capacity(n);
    let mut i = 0;
    let is_word = |i: usize, w: &str| matches!(toks.get(i).map(|t| &t.tok), Some(Tok::Word(x)) if x == w);
    while i < n {
        let t = &toks[i];
        if (is_word(i, "print") || is_word(i, "println")) && i >= 2 && toks[i - 1].tok == Tok::ColonColon && is_word(i - 2, "std") {
            let ln = is_word(i, "println");
            // the form: bare, or `.stored`, `.format`, …
            let mut j = i + 1;
            let mut form: Vec<Token> = Vec::new();
            while toks.get(j).is_some_and(|x| x.tok == Tok::Dot) && matches!(toks.get(j + 1).map(|x| &x.tok), Some(Tok::Word(_))) { form.push(toks[j].clone()); form.push(toks[j + 1].clone()); j += 2; }
            let form_words: Vec<String> = form.iter().filter_map(|x| if let Tok::Word(w) = &x.tok { Some(w.clone()) } else { None }).collect();
            if form_words.iter().any(|w| w == "auto_newline" || w == "auto_newln") { return Err(old(t, &format!("print.{}", form_words.join(".")), "std::print(…, newln)")); }
            // `println` was dropped (owner, 2026-10-28: "drop"): a line ends with `newln`, in every form
            if ln { return Err(old(t, &format!("std::println{}", form_words.iter().map(|w| format!(".{w}")).collect::<String>()), &format!("std::print{}(…, newln)", form_words.iter().map(|w| format!(".{w}")).collect::<String>()))); }
            let call = toks.get(j).is_some_and(|x| x.tok == Tok::LParen);
            if !call || (!ln && !form.is_empty()) {
                // a named form of `print` (`print.stored(…)`, `print.flush`, `print.writer…`) is as it was
                out.push(t.clone()); i += 1; continue;
            }
            out.push(word("print", t));
            if form.is_empty() { out.push(like(Tok::Dot, t)); out.push(word("shortest", t)); } else { out.extend(form.iter().cloned()); }
            let format = form_words.first().is_some_and(|w| w == "format");
            // the arguments, to the matching `)`
            out.push(toks[j].clone());
            let mut d = 0i32; let mut k = j;
            let mut close = None;
            while k < n { match toks[k].tok { Tok::LParen => d += 1, Tok::RParen => { d -= 1; if d == 0 { close = Some(k); break; } } _ => {} } k += 1; }
            let Some(close) = close else { out.push(t.clone()); i += 1; continue; };
            let args = &toks[j + 1..close];
            if ln && format {
                // the format's text ends its line
                let mut first = true;
                for a in args {
                    if first { if let Tok::Text(s, raw) = &a.tok { out.push(like(Tok::Text(format!("{s}{{newln}}"), format!("{raw}{{newln}}")), a)); first = false; continue; } }
                    first = false;
                    out.push(a.clone());
                }
            } else {
                out.extend(args.iter().cloned());
                if ln { if !args.is_empty() { out.push(like(Tok::Comma, &toks[close])); } out.push(like(Tok::Newline, &toks[close])); }
            }
            out.push(toks[close].clone());
            i = close + 1;
            continue;
        }
        out.push(t.clone());
        i += 1;
    }
    Ok(out)
}

fn shown(t: &Tok) -> String {
    match t { Tok::Word(w) => w.clone(), Tok::Var(v) => format!("${v}"), Tok::Dot => ".".into(), Tok::Mark => "%".into(), Tok::Op(o) => o.clone(), other => format!("{other}").trim_matches('`').to_string() }
}

/// Redesign two's name for a number type, `int.sign.32`, as redesign three writes it, `i32`
fn short_number(s: &str) -> String {
    let t = s.replace('%', "");
    let parts: Vec<&str> = t.split('.').collect();
    let (k, w) = match parts.as_slice() {
        ["int", "sign", w] => ("i", *w), ["int", "unsign", w] => ("u", *w), ["float", "bin", w] => ("f", *w),
        ["float", "deci", w] => ("d", *w), ["float", "brain", w] => ("bf", *w),
        ["int", "sign", "not_std", w] => return format!("%not_std.i{w}"), ["int", "unsign", "not_std", w] => return format!("%not_std.u{w}"),
        _ => return s.to_string(),
    };
    let limb = w == "128" || k == "d" || k == "bf";
    format!("{}{k}{w}", if limb { "%" } else { "" })
}

/// A message's code in redesign three's spellings: `end of if` → `;if`, `!MAIN` → `MAIN`, `eq` → `==`, `int.sign.32` →
/// `i32`, inside its backticks
pub fn threes(msg: &str) -> String {
    let mut out = String::new();
    let mut rest = msg;
    while let Some(a) = rest.find('`') {
        // a quotation stays as it was said: (owner, 2026-10-28: "`plus` isn't even `std`")
        if let Some(q) = rest[..a].find('"') {
            out.push_str(&rest[..q + 1]);
            let Some(e) = rest[q + 1..].find('"') else { out.push_str(&rest[q + 1..]); return out; };
            out.push_str(&rest[q + 1..q + 2 + e]);
            rest = &rest[q + 2 + e..];
            continue;
        }
        out.push_str(&rest[..a]);
        let after = &rest[a + 1..];
        let Some(b) = after.find('`') else { out.push_str(&rest[a..]); return out; };
        out.push('`');
        // the spelling a message refuses stays as written: "`and` is an old spelling: write `&`"
        let refused = [" is an old spelling", " is gone", " is a limb's", " is the base's", " is standard"].iter().any(|r| after[b + 1..].starts_with(r));
        out.push_str(&if refused { after[..b].to_string() } else { three_code(&after[..b]) });
        out.push('`');
        rest = &after[b + 1..];
    }
    out.push_str(rest);
    out
}

fn three_code(c: &str) -> String {
    let mut s = c.to_string();
    for (a, b) in [("to.useful.", "to.shortest."), ("print.useful", "print.shortest"), ("otherwise fail;", "?"), ("otherwise fail", "?"), ("!end of ", ";"), ("end of ", ";"), ("!MAIN", "MAIN"), ("!no_check", "no_check"), ("!case ", "case "), (".!", ".*")] { s = s.replace(a, b); }
    for m in ["MAIN", "no_check", "NO_FUCKING_CHECKS", "case "] { if let Some(r) = s.strip_prefix('!') { if r.starts_with(m) { s = r.to_string(); } } }
    let words: Vec<String> = s.split(' ').map(|w| match w {
        "plus" => "+".into(), "minus" => "-".into(), "times" => "x".into(), "pow" => "^".into(), "eq" => "==".into(), "not_eq" => "!=".into(),
        "lt" => "<".into(), "gt" => ">".into(), "at_most" => "<=".into(), "at_least" => ">=".into(),
        other => other.to_string(),
    }).collect();
    let mut s = words.join(" ");
    for p in ["int.sign.", "int.unsign.", "float.bin.", "float.%deci.", "float.deci.", "float.%brain.", "int.sign.%", "int.unsign.%", "float.bin.%"] {
        while let Some(at) = s.find(p) {
            let end = s[at..].find(|c: char| !(c.is_alphanumeric() || c == '.' || c == '%' || c == '_')).map_or(s.len(), |e| at + e);
            let long = s[at..end].to_string();
            let short = short_number(&long);
            if short == long { break; }
            s.replace_range(at..end, &short);
        }
    }
    if let Some(r) = s.strip_prefix("neg ") { s = format!("-{r}"); }
    // a limb's number type wears its mark in front (`d64` → `%d64`), as the oracle's own names do not
    let mut out = String::new();
    let cs: Vec<char> = s.chars().collect();
    let mut k = 0;
    while k < cs.len() {
        let starts = k == 0 || !(cs[k - 1].is_alphanumeric() || cs[k - 1] == '_' || cs[k - 1] == '%');
        if starts {
            let mut e = k; while e < cs.len() && (cs[e].is_alphanumeric() || cs[e] == '_') { e += 1; }
            let w: String = cs[k..e].iter().collect();
            if number(&w).is_some_and(|(_, _, _, limb)| limb) { out.push('%'); }
            if w == "brain16" { out.push_str("%bf16"); k = e; continue; }
            out.push_str(&w);
            if e > k { k = e; continue; }
        }
        out.push(cs[k]);
        k += 1;
    }
    out
}
