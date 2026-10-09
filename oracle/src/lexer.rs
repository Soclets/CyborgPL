//! Tokens. Everything the design says about how source text is cut up lives here.
//!
//! Token classes, by shape (design/syntax-sketch.md):
//!   "…"   text literal (exactly its characters; `\"` and `\\` only)   '…'   variable name   `…`   one character
//!   bare word                    keyword, function name, or operator word
//!   [ ] { } ( )                  value slot, body, arithmetic grouping
//!   ::  .  :=  =  ;  ,           Neuron boundary, chain, best-type bind, typed bind, end, separator
//!   :  ->                        name-to-type, function return type (decided 2026-09-19)
//!   newln                        a newline piece (owner, 2026-10-07; `\n` before)
//!   ++                           joins text (decided 2026-09-19)
//!   numbers                      `_` groups digits; `,` never does

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Word(String),   // bare identifier: keyword / function / operator word / chain segment
    Var(String),    // 'name'
    Text(String, String),   // "…": the stored text, and the text as typed (`print.format` reads that)
    Char(String),   // `…`
    Int(i128),
    /// a whole number past i128, for the wide `!std_` integers (owner, 2026-10-03): its digits
    BigInt(String),
    Bin(f64, String),   // the value, and its digits as written (to round once, at a narrower width)
    Newline,        // bare \n
    LBracket, RBracket,
    LBrace, RBrace,
    LParen, RParen,
    ColonColon,     // ::
    Dot,            // .
    Walrus,         // :=
    Eq,             // =
    ShadowWalrus,   // $:=
    ShadowEq,       // $=
    Strict,         // #strict — the file's reasons are written out (owner, 2026-09-27)
    NoShadow,       // !$ — before a declaration: the name cannot be shadowed (owner, 2026-09-25)
    Colon,          // :   'x': int32
    Ellipsis,       // ... a variadic parameter
    LeftArrow,      // <-  asm: register into variable
    BothArrow,      // <-> asm: both ways
    Raw(String),    // the verbatim text of an `asm { … }` block
    Arrow,          // ->  fn f [...] -> int32
    Semi,           // ;
    Comma,          // ,
    Op(String),     // + - ++ == !== < > </== >/==
    Eof,
}

impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tok::Word(w) => write!(f, "`{w}`"),
            Tok::Var(v) => write!(f, "'{v}'"),
            Tok::Text(t, _) => write!(f, "\"{t}\""),
            Tok::Char(c) => write!(f, "`{c}`"),
            Tok::Int(i) => write!(f, "{i}"),
            Tok::BigInt(s) => write!(f, "{s}"),
            Tok::Bin(b, _) => write!(f, "{b}"),
            Tok::Newline => write!(f, "\\n"),
            Tok::LBracket => write!(f, "["),
            Tok::RBracket => write!(f, "]"),
            Tok::LBrace => write!(f, "{{"),
            Tok::RBrace => write!(f, "}}"),
            Tok::LParen => write!(f, "("),
            Tok::RParen => write!(f, ")"),
            Tok::ColonColon => write!(f, "::"),
            Tok::Dot => write!(f, "."),
            Tok::Walrus => write!(f, ":="),
            Tok::Eq => write!(f, "="),
            Tok::ShadowWalrus => write!(f, "$:="),
            Tok::ShadowEq => write!(f, "$="),
            Tok::NoShadow => write!(f, "!$"),
            Tok::Strict => write!(f, "#strict"),
            Tok::Colon => write!(f, ":"),
            Tok::Ellipsis => write!(f, "..."),
            Tok::LeftArrow => write!(f, "<-"),
            Tok::BothArrow => write!(f, "<-->"),
            Tok::Raw(_) => write!(f, "assembly"),
            Tok::Arrow => write!(f, "->"),
            Tok::Semi => write!(f, ";"),
            Tok::Comma => write!(f, ","),
            Tok::Op(o) => write!(f, "{o}"),
            Tok::Eof => write!(f, "end of file"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tok: Tok,
    pub line: usize,
    pub col: usize,
    /// an old spelling, as written, and the one that replaced it: refused everywhere since MTM, which
    /// kept them, is gone (owner, 2026-10-07: "MTM is gone.")
    pub mtm: Option<(String, &'static str)>,
}

#[derive(Debug)]
pub struct LexError {
    pub line: usize,
    pub col: usize,
    pub msg: String,
}

/// A message's column counts graphemes, what a reader counts (owner, 2026-10-06: "our error messages
/// are gonna be graphemes"; Brainstem's rule). The lexer walks code points, so its columns are mapped
/// here, a line at a time, with the oracle's grapheme approximation.
pub fn lex(src: &str) -> Result<Vec<Token>, LexError> {
    let lines: Vec<Vec<char>> = src.split('\n').map(|l| l.chars().collect()).collect();
    let to_graphemes = |line: usize, col: usize| -> usize {
        if col <= 1 { return col; }
        let Some(l) = lines.get(line - 1) else { return col };
        let before: String = l[..(col - 1).min(l.len())].iter().collect();
        crate::interp::grapheme_count(&before) + 1
    };
    match lex_points(src) {
        Ok(mut toks) => { for t in &mut toks { t.col = to_graphemes(t.line, t.col); } Ok(toks) }
        Err(mut e) => { e.col = to_graphemes(e.line, e.col); Err(e) }
    }
}

/// A name's characters: English letters, digits and `_`, and, for WSTN to allow (owner, 2026-10-06), any
/// other script or emoji; Brainstem refuses those in the parser, where it knows the cores.
fn name_start(c: char) -> bool { c.is_alphabetic() || c == '_' || (!c.is_ascii() && !c.is_whitespace() && !c.is_control()) }
fn name_char(c: char) -> bool { c.is_alphanumeric() || c == '_' || (!c.is_ascii() && !c.is_whitespace() && !c.is_control()) }

fn lex_points(src: &str) -> Result<Vec<Token>, LexError> {
    let chars: Vec<char> = src.chars().collect();
    let mut toks = Vec::new();
    let mut i = 0;
    let mut line = 1;
    let mut col = 1;

    let err = |line, col, msg: String| Err(LexError { line, col, msg });

    while i < chars.len() {
        let c = chars[i];
        let (tl, tc) = (line, col);
        let push = |tok: Tok, toks: &mut Vec<Token>| toks.push(Token { tok, line: tl, col: tc, mtm: None });

        // whitespace
        if c == '\n' {
            i += 1; line += 1; col = 1; continue;
        }
        if c.is_whitespace() {
            i += 1; col += 1; continue;
        }

        // `\n` outside a string was the newline piece; it is `newln` since 2026-10-07 (owner: "If `\`
        // means escape, `\n` could read as escape the letter `n`")
        if c == '\\' {
            if i + 1 < chars.len() && chars[i + 1] == 'n' {
                push(Tok::Newline, &mut toks);
                toks.last_mut().unwrap().mtm = Some(("\\n".into(), "newln"));
                i += 2; col += 2; continue;
            }
            return err(line, col, "a bare backslash: a backslash is only an escape inside text; the newline piece is `newln`".into());
        }

        // "text"
        if c == '"' {
            let mut s = String::new();
            let mut raw = String::new();
            i += 1; col += 1;
            loop {
                if i >= chars.len() { return err(tl, tc, "unterminated text literal".into()); }
                let d = chars[i];
                if d == '"' { i += 1; col += 1; break; }
                // Decided 2026-09-19: a string is exactly its characters. `\` escapes only
                // what cannot otherwise be written inside one — `\"` and `\\`. Anything
                // else after a backslash is literal (`"\n"` is a backslash and an n; the
                // newline is the bare `\n` outside quotes).
                if d == '\\' && i + 1 < chars.len() && (chars[i + 1] == '"' || chars[i + 1] == '\\') {
                    s.push(chars[i + 1]);
                    raw.push('\\'); raw.push(chars[i + 1]);
                    i += 2; col += 2; continue;
                }
                if d == '\n' { line += 1; col = 0; }
                s.push(d);
                raw.push(d);
                i += 1; col += 1;
            }
            push(Tok::Text(s, raw), &mut toks);
            continue;
        }

        // 'variable'
        if c == '\'' {
            let mut s = String::new();
            i += 1; col += 1;
            loop {
                if i >= chars.len() { return err(tl, tc, "unterminated variable name".into()); }
                let d = chars[i];
                if d == '\'' { i += 1; col += 1; break; }
                if d == '\n' { return err(tl, tc, "a variable name cannot span lines".into()); }
                s.push(d);
                i += 1; col += 1;
            }
            if s.is_empty() { return err(tl, tc, "empty variable name ''".into()); }
            push(Tok::Var(s), &mut toks);
            continue;
        }

        // `character`
        if c == '`' {
            let mut s = String::new();
            i += 1; col += 1;
            loop {
                if i >= chars.len() { return err(tl, tc, "unterminated character literal".into()); }
                let d = chars[i];
                if d == '`' { i += 1; col += 1; break; }
                if d == '\n' { return err(tl, tc, "a character literal cannot span lines".into()); }
                s.push(d);
                i += 1; col += 1;
            }
            if s.is_empty() { return err(tl, tc, "empty character literal ``".into()); }
            push(Tok::Char(s), &mut toks);
            continue;
        }

        // numbers: digits, `_` grouping, optional fraction; no leading zeros
        if c.is_ascii_digit() {
            if c == '0' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                return err(tl, tc, "a number does not start with 0 (leading zeros are not allowed)".into());
            }
            let mut s = String::new();
            let mut is_bin = false;
            while i < chars.len() {
                let d = chars[i];
                if d.is_ascii_digit() { s.push(d); }
                else if d == '_' { /* grouping, cosmetic */ }
                else if d == '.' && !is_bin && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() { is_bin = true; s.push('.'); }
                else { break; }
                i += 1; col += 1;
            }
            if i < chars.len() && chars[i] == ',' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                return err(tl, tc, "`,` is not a digit separator — use `_` (1_000_000); a comma separates values".into());
            }
            if i < chars.len() && (chars[i].is_alphabetic() || chars[i] == '\'') {
                return err(tl, tc, format!("a number runs straight into `{}`", chars[i]));
            }
            if is_bin {
                let v: f64 = s.parse().map_err(|_| LexError { line: tl, col: tc, msg: format!("bad number `{s}`") })?;
                push(Tok::Bin(v, s.clone()), &mut toks);
            } else {
                // past i128 it is a wide integer's literal; LLVM's widest integer is 2^23 bits
                match s.parse::<i128>() {
                    Ok(v) => push(Tok::Int(v), &mut toks),
                    Err(_) => {
                        if s.len() > 2_600_000 { return err(tl, tc, format!("an integer literal of {} digits is past the widest integer there is (2^23 bits)", s.len())); }
                        push(Tok::BigInt(s.clone()), &mut toks)
                    }
                }
            }
            continue;
        }

        // bare words: keywords, function names, operator words, chain segments
        if name_start(c) {
            let mut s = String::new();
            while i < chars.len() && name_char(chars[i]) {
                s.push(chars[i]);
                i += 1; col += 1;
            }
            // `inline.ASM { … }` — the braces hold raw assembly, taken verbatim up to
            // the matching `}` (design/ffi.md, 2026-09-20)
            // `x~` / `xx~` — wrapping multiply / power (decided 2026-09-20): the word and the
            // tilde are one operator
            if (s == "x" || s == "xx") && i < chars.len() && chars[i] == '~' {
                push(Tok::Op(format!("{s}~")), &mut toks);
                i += 1; col += 1;
                continue;
            }
            let after_inline = toks.len() >= 2 && toks[toks.len() - 1].tok == Tok::Dot && matches!(&toks[toks.len() - 2].tok, Tok::Word(w) if w == "inline");
            let is_asm = (s == "ASM" || s == "S" || s == "Assembly") && after_inline;
            // the words that replaced `!` and the comparison symbols (owner, 2026-10-07) are read as the
            // tokens the oracle has always used inside; the old spellings are kept, to be refused by name
            let after_bang = matches!(toks.last().map(|t| &t.tok), Some(Tok::Op(o)) if o == "!");
            let main: Option<&'static str> = match s.as_str() {
                "nocheck" => Some("no_check"),
                // the inline languages' main names are `inline.ASM`, `inline.LL`, `inline.MSL` (owner, 2026-10-07)
                "Assembly" | "S" if after_inline => Some("inline.ASM"),
                "LLVM" | "LLVM_Intermediate_Representation" if after_inline => Some("inline.LL"),
                "Metal_Shading_Language" if after_inline => Some("inline.MSL"),
                // the standard library is `cord` (owner, 2026-10-09: "lowercase syntax"); `stdkit` (2026-10-07) and `std` before it
                "std" | "stdkit" if !after_bang => Some("cord"),
                // a library is a Neuron (owner, 2026-10-09); `pub.limb` is the old spelling of `pub.neuron`
                "limb" => Some("neuron"),
                "auto_newline" => Some("auto_newln"),
                "giveS" | "gives" => Some("is"),
                _ => None,
            };
            let inside: Option<Tok> = match s.as_str() {
                "no_check" => Some(Tok::Word("nocheck".into())),
                "no_cheat" => Some(Tok::Word("!cheat".into())),
                "no_shadow" => Some(Tok::NoShadow),
                "equal_to" => Some(Tok::Op("==".into())),
                "not_equal" => Some(Tok::Op("!==".into())),
                "at_most" => Some(Tok::Op("</==".into())),
                "at_least" => Some(Tok::Op(">/==".into())),
                "newln" => Some(Tok::Newline),
                "cord" => Some(Tok::Word("std".into())),
                "neuron" => Some(Tok::Word("limb".into())),
                "auto_newln" => Some(Tok::Word("auto_newline".into())),
                // a case's value says `is` (owner, 2026-10-07), held inside as `#strict`'s `giveS`
                "is" => Some(Tok::Word("giveS".into())),
                _ => None,
            };
            if s == "not_std" {
                // `not_std.int24` (owner, 2026-10-07), held inside as `!std_…`
                push(Tok::Op("!".into()), &mut toks);
                push(Tok::Word("std".into()), &mut toks);
                continue;
            }
            if let Some(t) = inside { push(t, &mut toks); continue; }
            let written = if after_inline && main.is_some() { format!("inline.{s}") } else { s.clone() };
            push(Tok::Word(s), &mut toks);
            if let Some(m) = main { toks.last_mut().unwrap().mtm = Some((written, m)); }
            if is_asm {
                let mut j = i;
                while j < chars.len() && chars[j].is_whitespace() && chars[j] != '\n' { j += 1; }
                if j < chars.len() && chars[j] == '{' {
                    let (bl, bc) = (line, col + (j - i));
                    let mut k = j + 1;
                    let mut depth = 1;
                    let mut raw = String::new();
                    while k < chars.len() {
                        let d = chars[k];
                        if d == '{' { depth += 1; } else if d == '}' { depth -= 1; if depth == 0 { break; } }
                        raw.push(d);
                        k += 1;
                    }
                    if k >= chars.len() { return err(bl, bc, "unclosed `asm {`".into()); }
                    for d in chars[i..=k].iter() { if *d == '\n' { line += 1; col = 1; } else { col += 1; } }
                    toks.push(Token { tok: Tok::Raw(raw), line: bl, col: bc, mtm: None });
                    i = k + 1;
                }
            }
            continue;
        }

        // multi-char punctuation, longest first
        let rest: String = chars[i..chars.len().min(i + 3)].iter().collect();
        let two: String = rest.chars().take(2).collect();
        if two == "!$" { push(Tok::NoShadow, &mut toks); toks.last_mut().unwrap().mtm = Some(("!$".into(), "no_shadow")); i += 2; col += 2; continue; }
        // `!c`, "no cheat", a loop's first modifier (owner, 2026-10-01): kept as its own word, so no
        // name can spell it
        // `!cheat` — "no cheat" (owner, 2026-10-06; `!c` before), one word; `!c` is still read as one,
        // so the parser can refuse it naming `!cheat`
        if chars[i..].iter().take(6).collect::<String>() == "!cheat" && !chars.get(i + 6).map_or(false, |c| c.is_alphanumeric() || *c == '_') {
            push(Tok::Word("!cheat".into()), &mut toks); toks.last_mut().unwrap().mtm = Some(("!cheat".into(), "no_cheat")); i += 6; col += 6; continue;
        }
        // `!check` is the main spelling of `nocheck` (owner, 2026-10-06): read as the word `nocheck`,
        // which is MTM's spelling when written out
        if chars[i..].iter().take(6).collect::<String>() == "!check" && !chars.get(i + 6).map_or(false, |c| c.is_alphanumeric() || *c == '_') {
            push(Tok::Word("nocheck".into()), &mut toks); toks.last_mut().unwrap().mtm = Some(("!check".into(), "no_check")); i += 6; col += 6; continue;
        }
        if two == "!c" && !chars.get(i + 2).map_or(false, |c| c.is_alphanumeric() || *c == '_') {
            push(Tok::Word("!c".into()), &mut toks); i += 2; col += 2; continue;
        }
        // `!mut` is the main spelling and `immut` MTM's (owner, 2026-09-25, then 2026-10-06)
        if rest.starts_with("!mu") && chars[i..].iter().take(4).collect::<String>() == "!mut" && !chars.get(i + 4).map_or(false, |c| c.is_alphanumeric() || *c == '_') {
            push(Tok::Word("immut".into()), &mut toks); toks.last_mut().unwrap().mtm = Some(("!mut".into(), "immut")); i += 4; col += 4; continue;
        }
        // shadowing is the word `shadow` in `let`'s place since 2026-10-08 (owner: "yea"): `$:=`, `$=` are old
        if rest.starts_with("$:=") { push(Tok::ShadowWalrus, &mut toks); toks.last_mut().unwrap().mtm = Some(("$:=".into(), "shadow 'x' := …")); i += 3; col += 3; continue; }
        if two == "$=" { push(Tok::ShadowEq, &mut toks); toks.last_mut().unwrap().mtm = Some(("$=".into(), "shadow 'x': T = …")); i += 2; col += 2; continue; }
        if c == '$' { return err(tl, tc, "`$` is not part of the language: a shadowing declaration is `shadow 'x' := …;` (owner, 2026-10-08)".into()); }
        if two == "::" { push(Tok::ColonColon, &mut toks); i += 2; col += 2; continue; }
        if two == ":=" { push(Tok::Walrus, &mut toks); i += 2; col += 2; continue; }
        if c == ':' { push(Tok::Colon, &mut toks); i += 1; col += 1; continue; }
        if rest.starts_with("...") { push(Tok::Ellipsis, &mut toks); i += 3; col += 3; continue; }

        // Operator characters (decided 2026-09-20): a maximal run is one operator —
        // the language's own, or a custom `op`; a space separates two. `$`, `:`, `.`
        // and the brackets are grammar, not operator characters.
        const OP_CHARS: &str = "!%&*+-/<>=?@^|~";
        if OP_CHARS.contains(c) {
            let mut run = String::new();
            let mut j = i;
            while j < chars.len() && OP_CHARS.contains(chars[j]) { run.push(chars[j]); j += 1; }
            let n = run.len();
            // the value's `not` is a spaced `!` (owner, 2026-10-06: "Why not with a space, `!` without
            // space reads as a rule"); a glued one is refused
            // `!std_int24` reads `!` then the word, as it always has
            let std_word = matches!(chars[j..].iter().take(4).collect::<String>().as_str(), "std_" | "std.");
            if run == "!" && !std_word {
                if chars.get(j).map_or(true, |c| c.is_whitespace()) { push(Tok::Word("not".into()), &mut toks); toks.last_mut().unwrap().mtm = Some(("!".into(), "not")); i = j; col += 1; continue; }
                return err(tl, tc, "`!` is not an operator since 2026-10-07: a value's not is `not 'ready'`".into());
            }
            if run == "!" && std_word {
                push(Tok::Op("!".into()), &mut toks); toks.last_mut().unwrap().mtm = Some(("!std.".into(), "not_std.")); i = j; col += 1; continue;
            }
            // the arrows are short, `->`, `<-`, `<->` (owner, 2026-10-07: "Short, the neck & eye movement
            // is less"); the comparisons are words (`equal_to`, `not_equal`, `at_most`, `at_least`); every
            // other spelling is refused by name, MTM being gone
            let main: Option<&'static str> = match run.as_str() {
                "-->" | ">>" | ">>>" => Some("->"),
                "<--" | "<<" | "<<<" => Some("<-"),
                "<-->" | "<<>>" | "<<<>>>" | "<---->" | "<--|-->" | "<-|->" | "<<|>>" | "<<<|>>>" => Some("<->"),
                "&" | "&@" => Some("lend` / `lent`; a bitwise and is `and.bit"),
                "==" => Some("equal_to"),
                "!==" => Some("not_equal"),
                "</==" => Some("at_most"),
                ">/==" => Some("at_least"),
                _ => None,
            };
            let written = run.clone();
            let tok = match run.as_str() {
                // `-->` is another spelling of `->`, everywhere (owner, 2026-10-03)
                // and `>>`, `>>>` too, as `<<`, `<<<` are of `<-` (owner, 2026-10-06: "yea")
                "->" | "-->" | ">>" | ">>>" => Tok::Arrow,
                // `<--` is another spelling of `<-` (owner, 2026-10-03)
                "<-" | "<--" | "<<" | "<<<" => Tok::LeftArrow,
                // both ways: `<-->` is the main spelling, as the other arrows are long (owner, 2026-10-06:
                // "yea"); `<->`, `<<>>`, `<<<>>>` and `<---->`, "Two long arrows", are the others, and the
                // walled ones, `<--|-->`, `<-|->`, `<<|>>`, `<<<|>>>` (owner: "`b`")
                "<-->" | "<->" | "<<>>" | "<<<>>>" | "<---->" | "<--|-->" | "<-|->" | "<<|>>" | "<<<|>>>" => Tok::BothArrow,
                "=" => Tok::Eq,
                "!=" => return err(tl, tc, "`!=` is not an operator; not-equal is `not_equal`".into()),
                "<=" | "<==" => return err(tl, tc, format!("`{run}` is not an operator; less-or-equal is `at_most` (owner, 2026-10-07)")),
                ">=" | ">==" => return err(tl, tc, format!("`{run}` is not an operator; greater-or-equal is `at_least` (owner, 2026-10-07)")),
                "/" => return err(tl, tc, "`/` is not an operator; division is `div`: `'a' div 'b'` (owner, 2026-09-29)".into()),
                "--" | "+=" | "-=" => return err(tl, tc, format!("`{run}` does not exist; write the change in full: change 'i' = 'i' plus 1;")),
                _ => Tok::Op(run),   // built-in or custom; the parser tells them apart
            };
            push(tok, &mut toks);
            if let Some(m) = main { toks.last_mut().unwrap().mtm = Some((written, m)); }
            i = j; col += n; continue;
        }

        let single = match c {
            '[' => Tok::LBracket, ']' => Tok::RBracket,
            '{' => Tok::LBrace, '}' => Tok::RBrace,
            '(' => Tok::LParen, ')' => Tok::RParen,
            '.' => Tok::Dot, ';' => Tok::Semi, ',' => Tok::Comma,
            '#' => {
                // Comments (decided 2026-09-17):
                //   #      from here to the end of this line
                //   #wl    the whole line, including code before the #
                //   #Nl    N lines, counting this one
                //   #Nld   this line and N more down (N+1 lines)
                let mut j = i + 1;
                let mut word = String::new();
                while j < chars.len() && chars[j].is_ascii_alphanumeric() { word.push(chars[j]); j += 1; }
                let skip_lines: usize;   // full lines to skip after this one
                if word == "strict" {
                    // `#strict` (owner, 2026-09-27): every check site's proof written in its own
                    // function. A compile-time rule about proofs, which the oracle has none of, so
                    // here it is recognised and changes nothing.
                    toks.push(Token { tok: Tok::Strict, line: tl, col: tc, mtm: None });
                    col += j - i; i = j; continue;
                }
                if word == "wl" {
                    // drop everything already lexed on this line
                    while toks.last().map(|t| t.line == tl).unwrap_or(false) { toks.pop(); }
                    skip_lines = 0;
                } else if let Some(n) = word.strip_suffix("ld").and_then(|d| d.parse::<usize>().ok()) {
                    skip_lines = n;
                } else if let Some(n) = word.strip_suffix('l').and_then(|d| d.parse::<usize>().ok()) {
                    if n == 0 { return err(tl, tc, "`#0l` comments nothing".into()); }
                    skip_lines = n - 1;
                } else if word.is_empty() {
                    skip_lines = 0; // plain `#`, followed by a space or the end of the line
                } else {
                    return err(tl, tc, format!("`#{word}` is not a comment form; use `# text`, `#wl`, `#Nl`, or `#Nld`"));
                }
                // rest of this line
                while i < chars.len() && chars[i] != '\n' { i += 1; }
                // and N more whole lines
                let mut left = skip_lines;
                while left > 0 && i < chars.len() {
                    i += 1; line += 1; col = 1; // consume the newline
                    while i < chars.len() && chars[i] != '\n' { i += 1; }
                    left -= 1;
                }
                continue;
            }
            other => return err(tl, tc, format!("unexpected character `{other}`")),
        };
        push(single, &mut toks);
        i += 1; col += 1;
    }
    toks.push(Token { tok: Tok::Eof, line, col, mtm: None });
    Ok(toks)
}
