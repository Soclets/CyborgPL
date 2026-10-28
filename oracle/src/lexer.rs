//! Tokens. Everything the design says about how source text is cut up lives here.
//!
//! Token classes, by shape (design/syntax-sketch.md):
//!   "…"   text literal (exactly its characters; `\"` and `\\` only)   '…'   variable name   `…`   one character
//!   bare word                    keyword, function name, or operator word
//!   [ ] { } ( )                  value slot, body, arithmetic grouping
//!   ::  .  :=  =  ;  ,           kit boundary, chain, best-type bind, typed bind, end, separator
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
    /// `7.5%` / `7.5 percent` (owner, 2026-10-16): the fraction, as `Bin`, marked a percentage literal
    PercentLit(f64, String),
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
    Colon,          // :   'x': i32
    Ellipsis,       // ... a variadic parameter
    LeftArrow,      // <-  asm: register into variable
    BothArrow,      // <-> asm: both ways
    Raw(String),    // the verbatim text of an `asm { … }` block
    Arrow,          // ->  fn f [...] -> i32
    Semi,           // ;
    Comma,          // ,
    Op(String),     // + - ++ == !== < > </== >/==
    /// `%` glued to a word: the limb mark (owner, 2026-10-26), checked and taken out by `marks`
    Mark,
    /// `!` glued to a word, what needs attention (owner, 2026-10-26): `!MAIN`, `!no_check`, `!case`, `!end of`; read and
    /// taken out by `redesign::front`
    Attention,
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
            Tok::PercentLit(_, t) => write!(f, "{}%", crate::value::percent_digits(t)),
            Tok::BigInt(s) => write!(f, "{s}"),
            Tok::Bin(b, _) => write!(f, "{b}"),
            Tok::Newline => write!(f, "\\n"),
            Tok::LBracket => write!(f, "["),
            Tok::RBracket => write!(f, "]"),
            // a block's `{` and `}` are its opening `:` and its `end of` line in what the writer wrote (2026-10-26)
            Tok::LBrace => write!(f, "`:`"),
            Tok::RBrace => write!(f, "`end of …`"),
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
            Tok::Mark => write!(f, "%"),
            Tok::Attention => write!(f, "!"),
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

thread_local! {
    /// The `#generated_N_lines_below` markers of the file last lexed: (line, N)
    pub static GENERATED: std::cell::RefCell<Vec<(usize, usize)>> = std::cell::RefCell::new(Vec::new());
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tok: Tok,
    pub line: usize,
    pub col: usize,
    /// an old spelling, as written, and the one that replaced it: refused everywhere since MTM, which
    /// kept them, is gone (owner, 2026-10-07: "MTM is gone.")
    pub mtm: Option<(String, &'static crate::spellings::Old)>,
    /// a `[` or `]` that was written `(` or `)`, a call's (owner, 2026-10-22: "`( )` runs something"): the parser turns
    /// a call's parentheses into its brackets, so the rest of it reads one kind
    pub paren: bool,
    /// where the token starts, in characters from the file's start (`oracle --to-paren-calls` rewrites from it)
    pub at: usize,
    /// where it ends, in characters, one past its last (`oracle --redesign` rewrites from it)
    pub end: usize,
}

#[derive(Debug)]
pub struct LexError {
    pub line: usize,
    pub col: usize,
    pub msg: String,
}

thread_local! {
    /// set while text in the spellings before the 2026-10-26/27 redesign is read: `oracle --redesign`'s input, and the
    /// oracle's own built-in sources until they are rewritten
    pub static OLD_SYNTAX: std::cell::Cell<bool> = std::cell::Cell::new(false);
}

/// Text in the spellings before the redesign, read as it always was
pub fn lex_old(src: &str) -> Result<Vec<Token>, LexError> {
    let was = OLD_SYNTAX.with(|o| o.replace(true));
    let r = lex(src);
    OLD_SYNTAX.with(|o| o.set(was));
    r
}

fn old_syntax() -> bool { OLD_SYNTAX.with(|o| o.get()) }

thread_local! {
    /// `oracle --redesign` reads the marks as tokens, to move them where the new names want them
    pub static KEEP_MARKS: std::cell::Cell<bool> = std::cell::Cell::new(false);
    /// `oracle --redesign`'s second step reads each word as written, none read as the token it stands for
    pub static RAW_WORDS: std::cell::Cell<bool> = std::cell::Cell::new(false);
}

fn raw_words() -> bool { RAW_WORDS.with(|r| r.get()) }

/// The operator symbols of redesign three (owner, 2026-10-27), and its other marks: `?` / `??` for `otherwise`, `*` before a
/// jumping handler, the `change` forms `+=` …
pub const THREE_SYMBOLS: &[&str] = &["+", "-", "^", "==", "!=", "<", ">", "<=", ">=", "&", "&&", "|", "||", "++", "?", "??", "*", "+=", "-=", "^=", "++="];

thread_local! {
    /// `oracle --mark-limbs`: the edits that would mark a file's limb words and spell its percentages out, gathered
    /// instead of refused
    pub static FIX_EDITS: std::cell::RefCell<Option<Vec<crate::marks::Edit>>> = std::cell::RefCell::new(None);
}

/// A message's column counts graphemes, what a reader counts (owner, 2026-10-06: "our error messages
/// are gonna be graphemes"; the base's rule). The lexer walks code points, so its columns are mapped
/// here, a line at a time, with the oracle's grapheme approximation.
pub fn lex(src: &str) -> Result<Vec<Token>, LexError> {
    // each line counted once, only the lines a column is asked of
    let lines: Vec<&str> = src.split('\n').collect();
    let mut counted: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    let mut to_graphemes = |line: usize, col: usize| -> usize {
        if col <= 1 { return col; }
        let Some(l) = lines.get(line - 1) else { return col };
        let pre = counted.entry(line).or_insert_with(|| crate::interp::grapheme_prefixes(&l.chars().collect::<Vec<_>>()));
        pre[(col - 1).min(pre.len() - 1)] + 1
    };
    match lex_points(src).and_then(finish) {
        Ok(mut toks) => { for t in &mut toks { t.col = to_graphemes(t.line, t.col); } Ok(toks) }
        Err(mut e) => { e.col = to_graphemes(e.line, e.col); Err(e) }
    }
}

/// A name's characters: English letters, digits and `_`, and, for WSTN to allow (owner, 2026-10-06), any
/// other script or emoji; the base refuses those in the parser, where it knows the cores.
fn name_start(c: char) -> bool { c.is_alphabetic() || c == '_' || (!c.is_ascii() && !c.is_whitespace() && !c.is_control()) }
fn name_char(c: char) -> bool { c.is_alphanumeric() || c == '_' || (!c.is_ascii() && !c.is_whitespace() && !c.is_control()) }

fn lex_points(src: &str) -> Result<Vec<Token>, LexError> {
    let chars: Vec<char> = src.chars().collect();
    let mut toks: Vec<Token> = Vec::new();
    let mut i = 0;
    let mut line = 1;
    let mut col = 1;
    // inside `inline.MSL …` before its code: the bracket depth, the code's `{` being the first at 0
    let mut msl_open: Option<i32> = None;
    // after `inline.ASM` / `inline.%MSL` in the new spellings: the block's `end of` line, its code running up to it from the
    // line after the opening `:` (owner, 2026-10-26: "a block of another language holds raw text up to its exact `end of`")
    let mut raw_pending: Option<String> = None;

    let err = |line, col, msg: String| Err(LexError { line, col, msg });

    // tokens pushed so far whose end is not yet known: it is where the scan stands when the next round begins
    let mut ended = 0;
    while i < chars.len() {
        let from = ended.min(toks.len()); for t in toks[from..].iter_mut() { if t.end == usize::MAX && t.at != usize::MAX { t.end = i; } }
        ended = toks.len();
        let c = chars[i];
        let (tl, tc) = (line, col);
        let ti = i;
        let push = |tok: Tok, toks: &mut Vec<Token>| toks.push(Token { tok, line: tl, col: tc, mtm: None, paren: false, at: ti, end: usize::MAX });

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
                toks.last_mut().unwrap().mtm = mark("\\n");
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

        // 'variable': the old spelling of `$'variable'` since 2026-10-27 (owner: "`''` just does the same thing as `$`, but
        // for crazy names")
        if c == '\'' {
            if !old_syntax() && !crate::spellings::loading() {
                let mut k = i + 1;
                while k < chars.len() && chars[k] != '\'' && chars[k] != '\n' { k += 1; }
                let name: String = chars[i + 1..k.min(chars.len())].iter().collect();
                let plain = name.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_') && name.chars().all(|c| c.is_alphanumeric() || c == '_');
                let shown = if plain { format!("${name}") } else { format!("$'{name}'") };
                return err(tl, tc, format!("a variable's name starts with `$` since 2026-10-27, quoted or not: write `{shown}` (owner)"));
            }
            i += 1; col += 1;
            let s = quoted_name(&chars, &mut i, &mut col, tl, tc)?;
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

        // a segment after `.` that starts with digits, `integer.signed.32_bit` (owner, 2026-10-27: "and not `bits_32`")
        let after_dot = matches!(toks.last().map(|t| &t.tok), Some(Tok::Dot)) || (matches!(toks.last().map(|t| &t.tok), Some(Tok::Mark)) && toks.len() >= 2 && toks[toks.len() - 2].tok == Tok::Dot);
        if c.is_ascii_digit() && after_dot && !old_syntax() {
            let mut j = i;
            while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') { j += 1; }
            let w: String = chars[i..j].iter().collect();
            if (w.ends_with("_bit") && w[..w.len() - 4].chars().all(|d| d.is_ascii_digit())) || (w.chars().all(|d| d.is_ascii_digit()) && matches!(toks.iter().rev().find(|t| t.tok != Tok::Mark && t.tok != Tok::Dot).map(|t| &t.tok), Some(Tok::Word(_)))) {
                push(Tok::Word(w), &mut toks);
                col += j - i; i = j;
                continue;
            }
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
            // a percentage, `80%` or `80 percent` (owner, 2026-10-16: "Yup", "yeah"): the number over 100, a bin as
            // `0.80` is, and exact as `exactly 80%` (an `exactly` form, as every decimal has); any digits
            // (a value right after it, `7%2`, is the old `%` for mod, refused by name as such)
            let glued = i < chars.len() && chars[i] == '%' && !chars.get(i + 1).map_or(false, |c| OP_CHARS.contains(*c) || c.is_ascii_digit() || *c == '\'' || *c == '(');
            let worded = {
                let mut k = i;
                while k < chars.len() && chars[k] == ' ' { k += 1; }
                let w: String = chars[k..].iter().take(8).collect();
                k > i && w.starts_with("percent") && !w[7..].chars().next().map_or(false, |c| c.is_alphanumeric() || c == '_')
            };
            // `80%` is the old spelling since `%` became the limb mark (owner, 2026-10-26: "make it `80 percent`")
            if glued {
                let fixed = FIX_EDITS.with(|f| f.borrow_mut().as_mut().map(|e| e.push(crate::marks::Edit { at: i, remove: 1, insert: " percent".into() })).is_some());
                if !fixed { return err(tl, tc, format!("`{s}%` is an old spelling: write `{s} percent`; `%` marks a limb's word since 2026-10-26 (owner)")); }
            }
            if glued || worded {
                let n = if glued { 1 } else { chars[i..].iter().take_while(|c| **c == ' ').count() + 7 };
                i += n; col += n;
                let (int, frac) = s.split_once('.').unwrap_or((s.as_str(), ""));
                let t = if int.len() <= 2 { format!("0.{}{int}{frac}", "0".repeat(2 - int.len())) } else { format!("{}.{}{frac}", &int[..int.len() - 2], &int[int.len() - 2..]) };
                let v: f64 = t.parse().map_err(|_| LexError { line: tl, col: tc, msg: format!("bad percentage `{s}%`") })?;
                push(Tok::PercentLit(v, t), &mut toks);
                continue;
            }
            if i < chars.len() && (chars[i].is_alphabetic() || chars[i] == '\'') {
                // `0b1010` / `0x1F` / `0o17` are other languages' spelling (owner, 2026-10-19: "(a)", a word in front)
                if s == "0" && matches!(chars[i], 'b' | 'x' | 'o') {
                    let w = match chars[i] { 'b' => "bin", 'x' => "hex", _ => "oct" };
                    let rest: String = chars[i + 1..].iter().take_while(|c| c.is_ascii_alphanumeric() || **c == '_').collect();
                    return err(tl, tc, format!("a number in another base is written with a word in front: `{w} {rest}` (owner, 2026-10-19)"));
                }
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
            // `bin 1010_0101`, `hex FF00_00FF`, `oct 755` (owner, 2026-10-19: "(a)"; shortened 2026-10-20, owner: "(a)"):
            // a whole number in another base, the word in front as `exactly` is; the digits are read here, `_` grouping
            // as in decimals. `binary` and `octal` are old spellings (spellings.cyr)
            // since 2026-10-27 they are `binary`, `hexadecimal`, `octal` (owner: "No abbrevs, bro")
            let radix_word = matches!(s.as_str(), "binary" | "hexadecimal" | "octal" | "bin" | "hex" | "oct");
            if radix_word {
                let mut k = i;
                while k < chars.len() && chars[k] == ' ' { k += 1; }
                let run: String = chars[k..].iter().take_while(|c| c.is_ascii_alphanumeric() || **c == '_').collect();
                let starts = run.chars().next().map_or(false, |c| c.is_ascii_digit() || c == '_' || ((s == "hex" || s == "hexadecimal") && c.is_ascii_hexdigit()));
                if k > i && starts {
                    if !raw_words() && matches!(s.as_str(), "binary" | "hexadecimal" | "octal") {
                        let short = match s.as_str() { "binary" => "bin", "hexadecimal" => "hex", _ => "oct" };
                        return err(tl, tc, format!("`{s} {run}` is an old spelling: write `{short} {run}` (owner, 2026-10-27, redesign two)"));
                    }
                    let radix = match s.as_str() { "bin" | "binary" => 2, "hex" | "hexadecimal" => 16, _ => 8 };
                    let what = match radix { 2 => "a binary number has only the digits 0 and 1", 16 => "a hex number has only 0-9 and A-F", _ => "an octal number has only the digits 0 to 7" };
                    if let Some(bad) = run.chars().find(|c| *c != '_' && !c.is_digit(radix)) { return err(tl, tc, format!("{what}: `{bad}` in `{s} {run}`")); }
                    if run.starts_with('_') || run.ends_with('_') || run.contains("__") { return err(tl, tc, format!("`_` groups digits: one between two of them (`{s} {}`)", run.trim_matches('_').replace("__", "_"))); }
                    let digits: String = run.chars().filter(|c| *c != '_').collect();
                    let v = u128::from_str_radix(&digits, radix).ok().filter(|v| *v <= i128::MAX as u128);
                    let Some(v) = v else { return err(tl, tc, format!("`{s} {run}` is past the oracle's whole numbers (i128)")); };
                    let n = k - i + run.chars().count();
                    i += n; col += n;
                    push(Tok::Int(v as i128), &mut toks);
                    continue;
                }
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
            // a limb's word after `inline.` wears its mark, `inline.%MSL` (owner, 2026-10-26)
            let m = usize::from(toks.last().map_or(false, |t| t.tok == Tok::Mark));
            let after_inline = toks.len() >= 2 + m && toks[toks.len() - 1 - m].tok == Tok::Dot && matches!(&toks[toks.len() - 2 - m].tok, Tok::Word(w) if w == "inline");
            let is_asm = (s == "ASM" || s == "S" || s == "Assembly") && after_inline;
            // `end of inline.ASM` names the block it closes; it opens none
            let closing = toks.len() >= 3 + m && (matches!(&toks[toks.len() - 3 - m].tok, Tok::Word(o) if o == "of") || {
                // `;inline.ASM`: a `;` starting this line, not the last statement's end
                let k = toks.len() - 3 - m;
                toks[k].tok == Tok::Semi && toks[k].line == tl && (k == 0 || toks[k - 1].line < tl)
            });
            // `inline.MSL threads $n { … }` (owner, 2026-10-01): the count comes first, so the first `{` outside any
            // bracket after it opens the Metal code, taken verbatim
            if (s == "MSL" || s == "Metal_Shading_Language") && after_inline {
                if old_syntax() { msl_open = Some(0); } else if !closing { raw_pending = Some(format!("inline.{}{s}", if m == 1 { "%" } else { "" })); }
            }
            // the words that replaced `!` and the comparison symbols (owner, 2026-10-07) are read as the
            // tokens the oracle has always used inside; the old spellings are kept, to be refused by name
            let after_bang = matches!(toks.last().map(|t| &t.tok), Some(Tok::Op(o)) if o == "!");
            // an old spelling, from `spellings.cyr` (owner, 2026-10-13): kept as written, to be refused by name
            let in_path = matches!(toks.last().map(|t| &t.tok), Some(Tok::Dot | Tok::ColonColon));
            let main: Option<&'static crate::spellings::Old> = if after_bang { None } else { crate::spellings::old_at(&s, after_inline, in_path) };
            let inside: Option<Tok> = if raw_words() { None } else { match s.as_str() {
                "no_shadow" => Some(Tok::NoShadow),
                // `eq` / `not_eq` since redesign two (owner, 2026-10-27); `equal_to` / `not_equal` before it
                "equal_to" if old_syntax() || crate::spellings::loading() => Some(Tok::Op("equal_to".into())),
                "eq" if false => Some(Tok::Op("equal_to".into())),
                // the words of `<` and `>` (owner, 2026-10-16: "the long"), read as the symbols inside
                // `lt` / `gt` since redesign two (owner, 2026-10-27: "yes"); the long ones before it
                "less_than" if old_syntax() || crate::spellings::loading() => Some(Tok::Op("<".into())),
                "greater_than" if old_syntax() || crate::spellings::loading() => Some(Tok::Op(">".into())),
                "lt" if false => Some(Tok::Op("<".into())),
                "gt" if false => Some(Tok::Op(">".into())),
                "not_equal" if old_syntax() => Some(Tok::Op("not_equal".into())),
                "not_eq" if false => Some(Tok::Op("not_equal".into())),
                "at_most" if old_syntax() => Some(Tok::Op("at_most".into())),
                "at_least" if old_syntax() => Some(Tok::Op("at_least".into())),
                "newln" => Some(Tok::Newline),
                _ => None,
            } };
            if let Some(t) = inside { push(t, &mut toks); continue; }
            let written = if after_inline && main.is_some() { format!("inline.{s}") } else { s.clone() };
            if is_asm && !old_syntax() && !closing { raw_pending = Some(format!("inline.{}{s}", if m == 1 { "%" } else { "" })); }
            push(Tok::Word(s), &mut toks);
            if let Some(m) = main { toks.last_mut().unwrap().mtm = Some((written, m)); }
            if is_asm && old_syntax() {
                // the word ends here, before the code taken whole after it
                if let Some(t) = toks.last_mut() { t.end = i; }
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
                    toks.push(Token { tok: Tok::Raw(raw), line: bl, col: bc, mtm: None, paren: false, at: usize::MAX, end: usize::MAX });
                    i = k + 1;
                }
            }
            continue;
        }

        // multi-char punctuation, longest first
        let rest: String = chars[i..chars.len().min(i + 3)].iter().collect();
        let two: String = rest.chars().take(2).collect();
        if two == "!$" { push(Tok::NoShadow, &mut toks); toks.last_mut().unwrap().mtm = mark("!$"); i += 2; col += 2; continue; }
        // `!c`, "no cheat", a loop's first modifier (owner, 2026-10-01): kept as its own word, so no
        // name can spell it
        // `!cheat` — "no cheat" (owner, 2026-10-06; `!c` before), one word; `!c` is still read as one,
        // so the parser can refuse it naming `!cheat`
        if chars[i..].iter().take(6).collect::<String>() == "!cheat" && !chars.get(i + 6).map_or(false, |c| c.is_alphanumeric() || *c == '_') {
            push(Tok::Word("!cheat".into()), &mut toks); toks.last_mut().unwrap().mtm = mark("!cheat"); i += 6; col += 6; continue;
        }
        // `!check` is the main spelling of `nocheck` (owner, 2026-10-06): read as the word `nocheck`,
        // which is MTM's spelling when written out
        if chars[i..].iter().take(6).collect::<String>() == "!check" && !chars.get(i + 6).map_or(false, |c| c.is_alphanumeric() || *c == '_') {
            push(Tok::Word("!check".into()), &mut toks); toks.last_mut().unwrap().mtm = mark("!check"); i += 6; col += 6; continue;
        }
        if two == "!c" && !chars.get(i + 2).map_or(false, |c| c.is_alphanumeric() || *c == '_') {
            push(Tok::Word("!c".into()), &mut toks); i += 2; col += 2; continue;
        }
        // `!mut` is the main spelling and `immut` MTM's (owner, 2026-09-25, then 2026-10-06)
        if rest.starts_with("!mu") && chars[i..].iter().take(4).collect::<String>() == "!mut" && !chars.get(i + 4).map_or(false, |c| c.is_alphanumeric() || *c == '_') {
            push(Tok::Word("immut".into()), &mut toks); toks.last_mut().unwrap().mtm = mark("!mut"); i += 4; col += 4; continue;
        }
        // `$total` is a variable, the same one as `'total'` (owner, 2026-10-22: "Why `$`? Even though, it requires a
        // shift, it's easier to write"): letters, digits and `_`; a name with spaces or symbols keeps its quotes
        if c == '$' {
            // `$'total due'` (owner, 2026-10-27): the quotes only let a name hold spaces or symbols
            if !old_syntax() && chars.get(i + 1) == Some(&'\'') {
                i += 2; col += 2;
                let s = quoted_name(&chars, &mut i, &mut col, tl, tc)?;
                push(Tok::Var(s), &mut toks);
                continue;
            }
            let start = i + 1;
            let mut j = start;
            if j < chars.len() && (chars[j].is_alphabetic() || chars[j] == '_') {
                while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') { j += 1; }
                push(Tok::Var(chars[start..j].iter().collect()), &mut toks);
                col += j - i; i = j;
                continue;
            }
            return err(tl, tc, "`$` starts a variable's name: `$total`; a name with spaces or symbols is quoted, `'total due'`".into());
        }
        if two == "::" { push(Tok::ColonColon, &mut toks); i += 2; col += 2; continue; }
        if two == ":=" { push(Tok::Walrus, &mut toks); i += 2; col += 2; continue; }
        if c == ':' {
            push(Tok::Colon, &mut toks); i += 1; col += 1;
            // the opening line of an inline block ends here: its code is the lines up to its `end of` line, kept whole
            let rest_of_line: String = chars[i..].iter().take_while(|c| **c != '\n').collect();
            let line_ends = rest_of_line.trim().is_empty() || rest_of_line.trim_start().starts_with('#');
            if let (Some(kind), true) = (raw_pending.clone(), line_ends) {
                raw_pending = None;
                let close = if raw_words() { format!("end of {kind}") } else { format!(";{kind}") };
                let mut k = i;
                while k < chars.len() && chars[k] != '\n' { k += 1; }
                let body_start = (k + 1).min(chars.len());
                let mut p = body_start;
                let mut found = None;
                while p < chars.len() {
                    let mut e = p;
                    while e < chars.len() && chars[e] != '\n' { e += 1; }
                    let text: String = chars[p..e].iter().collect();
                    if text.trim() == close { found = Some(p); break; }
                    p = e + 1;
                }
                let Some(end_at) = found else { return err(tl, tc, format!("this `{kind}` block has no `{close}` line")); };
                let raw: String = chars[body_start..end_at].iter().collect();
                // the code's lines are counted, and the scan goes on at the `end of` line
                for d in chars[i..end_at].iter() { if *d == '\n' { line += 1; col = 1; } else { col += 1; } }
                toks.push(Token { tok: Tok::Raw(raw), line: tl, col: tc, mtm: None, paren: false, at: usize::MAX, end: usize::MAX });
                i = end_at;
                // the column of the `end of` line's first character
                while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t') { i += 1; col += 1; }
            }
            continue;
        }
        if rest.starts_with("...") { push(Tok::Ellipsis, &mut toks); i += 3; col += 3; continue; }

        // Operator characters (decided 2026-09-20): a maximal run is one operator —
        // the language's own, or a custom `op`; a space separates two. `$`, `:`, `.`
        // and the brackets are grammar, not operator characters.
        const OP_CHARS: &str = "!%&*+-/<>=?@^|~";
        // `%` glued to a word marks a limb's word (owner, 2026-10-26)
        if c == '%' && chars.get(i + 1).map_or(false, |n| n.is_alphabetic() || *n == '_' || (n.is_ascii_digit() && after_dot && !old_syntax())) {
            push(Tok::Mark, &mut toks); i += 1; col += 1; continue;
        }
        if OP_CHARS.contains(c) {
            let mut run = String::new();
            let mut j = i;
            while j < chars.len() && OP_CHARS.contains(chars[j]) { run.push(chars[j]); j += 1; }
            let n = run.len();
            // the value's `not` is a spaced `!` (owner, 2026-10-06: "Why not with a space, `!` without
            // space reads as a rule"); a glued one is refused
            // `!std_int24` reads `!` then the word, as it always has
            let std_word = matches!(chars[j..].iter().take(4).collect::<String>().as_str(), "std_" | "std.");
            // `!` glued to a word is what needs attention (owner, 2026-10-26): `!MAIN`, `!no_check`, `!case`, `!end of`
            if run == "!" && !std_word && !old_syntax() && !crate::spellings::loading() && chars.get(j).map_or(false, |c| c.is_alphabetic() || *c == '_') {
                push(Tok::Attention, &mut toks); i = j; col += 1; continue;
            }
            if run == "!" && !std_word {
                if chars.get(j).map_or(true, |c| c.is_whitespace()) { push(Tok::Word("not".into()), &mut toks); toks.last_mut().unwrap().mtm = mark("!"); i = j; col += 1; continue; }
                return err(tl, tc, "`!` is not an operator since 2026-10-07: a value's not is `not $ready`".into());
            }
            if run == "!" && std_word {
                push(Tok::Op("!".into()), &mut toks); toks.last_mut().unwrap().mtm = mark("!std."); i = j; col += 1; continue;
            }
            // the arrows are short, `->`, `<-`, `<->` (owner, 2026-10-07: "Short, the neck & eye movement
            // is less"); the comparisons are words (`equal_to`, `not_equal`, `at_most`, `at_least`); every
            // other spelling is refused by name, MTM being gone
            // redesign three's symbols (owner, 2026-10-27): read as written; the front turns them into the words inside
            // `*` only before a jumping handler (after `.` or `otherwise`), `&` only between two values: elsewhere each is the
            // old spelling it was, named as before (`*const`, `&mut`, `&$x`)
            let three_place = match run.as_str() {
                "*" => matches!(toks.last().map(|t| &t.tok), Some(Tok::Dot)) || matches!(toks.last().map(|t| &t.tok), Some(Tok::Word(o)) if o == "otherwise"),
                "&" => matches!(toks.last().map(|t| &t.tok), Some(Tok::Var(_) | Tok::Int(_) | Tok::BigInt(_) | Tok::Bin(..) | Tok::Text(..) | Tok::Char(_) | Tok::RParen | Tok::RBracket | Tok::Word(_))) && !chars[j..].iter().skip_while(|c| **c == ' ').collect::<String>().starts_with("mut"),
                _ => true,
            };
            if !old_syntax() && !crate::spellings::loading() && three_place && THREE_SYMBOLS.contains(&run.as_str()) {
                push(Tok::Op(run), &mut toks); i = j; col += n; continue;
            }
            let mut main: Option<&'static crate::spellings::Old> = crate::spellings::old_at(&run, false, false);
            // a symbol spells only an operator that reads correctly (owner, 2026-10-16): `*` and `&` are old in
            // every place, each place naming its own word
            if !crate::spellings::loading() && (run == "*" || run == "&" || run == "&@") {
                let after_value = matches!(toks.last().map(|t| &t.tok), Some(Tok::Var(_) | Tok::Int(_) | Tok::BigInt(_) | Tok::Bin(..) | Tok::Text(..) | Tok::Char(_) | Tok::RParen | Tok::RBracket))
                    || matches!(toks.last().map(|t| &t.tok), Some(Tok::Word(b)) if b == "true" || b == "false");
                let rest: String = chars[j..].iter().skip_while(|c| **c == ' ').take(6).collect();
                let starts_word = |w: &str| rest.starts_with(w) && !rest[w.len()..].chars().next().map_or(false, |c| c.is_ascii_alphanumeric() || c == '_');
                let row = if run == "*" {
                    if after_value { "*" }
                    else if starts_word("const") { "*const" }
                    else if starts_word("mut") { "*mut" }
                    else if rest.starts_with('\'') || rest.starts_with('$') { if matches!(toks.last().map(|t| &t.tok), Some(Tok::Word(c)) if c == "change") { "change *'p' =" } else { "*'p'" } }
                    else if rest.starts_with(|c: char| c.is_ascii_digit()) { "*19.99" }
                    else { "*" }
                } else if run == "&" && after_value { "'a' & 'b'" }
                else if run == "&" && (starts_word("mut") || rest.starts_with("mut@")) { "&mut" }
                else { "&" };
                main = Some(crate::spellings::old(row));
            }
            if !crate::spellings::loading() {
                if let Some(m) = crate::spellings::not_operator(&run) { return err(tl, tc, m); }
            }
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
                _ => Tok::Op(run),   // built-in or custom; the parser tells them apart
            };
            push(tok, &mut toks);
            if let Some(m) = main { toks.last_mut().unwrap().mtm = Some((written, m)); }
            i = j; col += n; continue;
        }

        if let Some(d) = msl_open {
            match c {
                '(' | '[' => msl_open = Some(d + 1),
                ')' | ']' => msl_open = Some(d - 1),
                ';' => msl_open = None,
                '{' if d == 0 => {
                    msl_open = None;
                    // the token before the code ends where it ended, not after the code
                    if let Some(t) = toks.last_mut() { if t.end == usize::MAX { t.end = i; } }
                    let (bl, bc) = (line, col);
                    let mut k = i + 1;
                    let mut depth = 1;
                    let mut raw = String::new();
                    while k < chars.len() {
                        let e = chars[k];
                        if e == '{' { depth += 1; } else if e == '}' { depth -= 1; if depth == 0 { break; } }
                        raw.push(e);
                        k += 1;
                    }
                    if k >= chars.len() { return err(bl, bc, "an `inline.MSL {` is never closed".into()); }
                    for e in chars[i..=k].iter() { if *e == '\n' { line += 1; col = 1; } else { col += 1; } }
                    toks.push(Token { tok: Tok::Raw(raw), line: bl, col: bc, mtm: None, paren: false, at: usize::MAX, end: usize::MAX });
                    i = k + 1;
                    continue;
                }
                _ => {}
            }
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
                while j < chars.len() && (chars[j].is_ascii_alphanumeric() || chars[j] == '_') { word.push(chars[j]); j += 1; }
                let skip_lines: usize;   // full lines to skip after this one
                // `#generated_21_lines_below` (owner, 2026-10-13): the next 21 lines are what `kayo generate`
                // wrote from the template above; they are ordinary code, the marker a comment noted for the check
                if let Some(n) = word.strip_prefix("generated_").and_then(|r| r.strip_suffix("_lines_below")).and_then(|d| d.parse::<usize>().ok()) {
                    GENERATED.with(|g| g.borrow_mut().push((tl, n)));
                    while i < chars.len() && chars[i] != '\n' { i += 1; }
                    continue;
                }
                if word.starts_with("generated") {
                    return err(tl, tc, "a generated part is marked `#generated_21_lines_below`, the number being the lines under the marker (owner, 2026-10-13)".into());
                }
                if word == "strict" {
                    // `#strict` (owner, 2026-09-27): every check site's proof written in its own
                    // function. A compile-time rule about proofs, which the oracle has none of, so
                    // here it is recognised and changes nothing.
                    toks.push(Token { tok: Tok::Strict, line: tl, col: tc, mtm: None, paren: false, at: ti, end: usize::MAX });
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
    let from = ended.min(toks.len()); for t in toks[from..].iter_mut() { if t.end == usize::MAX && t.at != usize::MAX { t.end = i; } }
    toks.push(Token { tok: Tok::Eof, line, col, mtm: None, paren: false, at: chars.len(), end: chars.len() });
    Ok(toks)
}

/// The limb marks checked and taken out, after the new spellings are read into the parser's tokens
fn finish(mut toks: Vec<Token>) -> Result<Vec<Token>, LexError> {
    if KEEP_MARKS.with(|k| k.get()) { return Ok(toks); }
    if !old_syntax() && !crate::spellings::loading() {
        toks = crate::redesign::front(toks).map_err(|(line, col, msg)| LexError { line, col, msg })?;
    }
    let mut fix = FIX_EDITS.with(|f| f.borrow_mut().take());
    let r = crate::marks::check(&mut toks, fix.as_mut());
    FIX_EDITS.with(|f| *f.borrow_mut() = fix);
    r.map_err(|(line, col, msg)| LexError { line, col, msg })?;
    Ok(toks)
}

/// A quoted name's characters, `i` just past its opening `'`: a quoted name holds any printable character (owner,
/// 2026-10-16: "Yeah, allow it", then "Allow 1, 2, and 4"): `'80%'`, `'v1.2'`, `'total due'`, and `'it\'s'`, a `'`
/// written `\'`; design claude's readings: `\\` is a `\`, one space at a time and none at the ends, no tab or control
/// character
fn quoted_name(chars: &[char], i: &mut usize, col: &mut usize, tl: usize, tc: usize) -> Result<String, LexError> {
    let err = |msg: &str| Err(LexError { line: tl, col: tc, msg: msg.to_string() });
    let mut s = String::new();
    loop {
        if *i >= chars.len() { return err("unterminated variable name"); }
        let d = chars[*i];
        if d == '\'' { *i += 1; *col += 1; break; }
        if d == '\n' { return err("a variable name cannot span lines"); }
        if d == '\\' {
            match chars.get(*i + 1) {
                Some('\'') => { s.push('\''); *i += 2; *col += 2; continue; }
                Some('\\') => { s.push('\\'); *i += 2; *col += 2; continue; }
                _ => return err("in a quoted name, `\\` comes before `'` or `\\`: `$'it\\'s'`"),
            }
        }
        if d == ' ' && (s.is_empty() || s.ends_with(' ')) { return err("a quoted name's spaces come one at a time, between words: `$'total due'`"); }
        if d.is_whitespace() && d != ' ' { return err("a quoted name holds spaces, not tabs or other gaps"); }
        if d.is_control() { return err("a quoted name cannot hold a control character"); }
        s.push(d);
        *i += 1; *col += 1;
    }
    if s.is_empty() { return err("empty variable name ''"); }
    if s.ends_with(' ') { return err("a quoted name's spaces come one at a time, between words: `$'total due'`"); }
    Ok(s)
}

/// The old spelling `written` as the lexer marks it, unless the table itself is being read
fn mark(written: &str) -> Option<(String, &'static crate::spellings::Old)> {
    if crate::spellings::loading() { None } else { Some((written.to_string(), crate::spellings::old(written))) }
}

/// Text the oracle writes itself (a C header's lines, 2026-10-10): lexed with its limb words marked, as a writer marks
/// them (owner, 2026-10-26), since the generator writes the words bare.
pub fn lex_marking(src: &str) -> Result<Vec<Token>, LexError> {
    // the generator writes the spellings before the redesign (until it is rewritten)
    let was = OLD_SYNTAX.with(|o| o.replace(true));
    let r = lex_marking_old(src);
    OLD_SYNTAX.with(|o| o.set(was));
    r
}

fn lex_marking_old(src: &str) -> Result<Vec<Token>, LexError> {
    FIX_EDITS.with(|f| *f.borrow_mut() = Some(Vec::new()));
    let first = lex(src);
    let mut edits = FIX_EDITS.with(|f| f.borrow_mut().take()).unwrap_or_default();
    if edits.is_empty() { return first; }
    let mut chars: Vec<char> = src.chars().collect();
    edits.sort_by(|a, b| b.at.cmp(&a.at));
    for e in edits { chars.splice(e.at..e.at + e.remove, e.insert.chars()); }
    lex(&chars.into_iter().collect::<String>())
}
