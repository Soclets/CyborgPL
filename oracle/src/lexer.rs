//! Tokens. Everything the design says about how source text is cut up lives here.
//!
//! Token classes, by shape (design/syntax-sketch.md):
//!   "…"   text literal          '…'   variable name          `…`   one character
//!   bare word                    keyword, function name, or operator word
//!   [ ] { } ( )                  value slot, body, arithmetic grouping
//!   ::  .  :=  =  ;  ,           Limb boundary, chain, best-type bind, typed bind, end, separator
//!   \n                           a newline piece (bare, outside any string)
//!   numbers                      `_` groups digits; `,` never does

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Word(String),   // bare identifier: keyword / function / operator word / chain segment
    Var(String),    // 'name'
    Text(String),   // "…"
    Char(String),   // `…`
    Int(i128),
    Bin(f64),
    Newline,        // bare \n
    LBracket, RBracket,
    LBrace, RBrace,
    LParen, RParen,
    ColonColon,     // ::
    Dot,            // .
    Walrus,         // :=
    Eq,             // =
    Semi,           // ;
    Comma,          // ,
    Op(String),     // + - / == !== < > <== >==
    Eof,
}

impl fmt::Display for Tok {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tok::Word(w) => write!(f, "`{w}`"),
            Tok::Var(v) => write!(f, "'{v}'"),
            Tok::Text(t) => write!(f, "\"{t}\""),
            Tok::Char(c) => write!(f, "`{c}`"),
            Tok::Int(i) => write!(f, "{i}"),
            Tok::Bin(b) => write!(f, "{b}"),
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
}

#[derive(Debug)]
pub struct LexError {
    pub line: usize,
    pub col: usize,
    pub msg: String,
}

pub fn lex(src: &str) -> Result<Vec<Token>, LexError> {
    let chars: Vec<char> = src.chars().collect();
    let mut toks = Vec::new();
    let mut i = 0;
    let mut line = 1;
    let mut col = 1;

    let err = |line, col, msg: String| Err(LexError { line, col, msg });

    while i < chars.len() {
        let c = chars[i];
        let (tl, tc) = (line, col);
        let push = |tok: Tok, toks: &mut Vec<Token>| toks.push(Token { tok, line: tl, col: tc });

        // whitespace
        if c == '\n' {
            i += 1; line += 1; col = 1; continue;
        }
        if c.is_whitespace() {
            i += 1; col += 1; continue;
        }

        // bare \n : a newline piece
        if c == '\\' {
            if i + 1 < chars.len() && chars[i + 1] == 'n' {
                push(Tok::Newline, &mut toks);
                i += 2; col += 2; continue;
            }
            return err(line, col, "a bare backslash: only `\\n` is a piece outside a string".into());
        }

        // "text"
        if c == '"' {
            let mut s = String::new();
            i += 1; col += 1;
            loop {
                if i >= chars.len() { return err(tl, tc, "unterminated text literal".into()); }
                let d = chars[i];
                if d == '"' { i += 1; col += 1; break; }
                if d == '\\' {
                    if i + 1 >= chars.len() { return err(line, col, "unterminated escape in text".into()); }
                    let e = chars[i + 1];
                    s.push(match e {
                        'n' => '\n',
                        't' => '\t',
                        '"' => '"',
                        '\\' => '\\',
                        other => return err(line, col, format!("unknown escape `\\{other}` in text")),
                    });
                    i += 2; col += 2; continue;
                }
                if d == '\n' { line += 1; col = 0; }
                s.push(d);
                i += 1; col += 1;
            }
            push(Tok::Text(s), &mut toks);
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

        // numbers: digits, `_` grouping, optional fraction
        if c.is_ascii_digit() {
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
                push(Tok::Bin(v), &mut toks);
            } else {
                let v: i128 = s.parse().map_err(|_| LexError { line: tl, col: tc, msg: format!("integer literal `{s}` does not fit even an int128") })?;
                push(Tok::Int(v), &mut toks);
            }
            continue;
        }

        // bare words: keywords, function names, operator words, chain segments
        if c.is_alphabetic() || c == '_' {
            let mut s = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                s.push(chars[i]);
                i += 1; col += 1;
            }
            push(Tok::Word(s), &mut toks);
            continue;
        }

        // multi-char punctuation, longest first
        let rest: String = chars[i..chars.len().min(i + 3)].iter().collect();
        let two: String = rest.chars().take(2).collect();
        if rest.starts_with("!==") { push(Tok::Op("!==".into()), &mut toks); i += 3; col += 3; continue; }
        if rest.starts_with("<==") { push(Tok::Op("<==".into()), &mut toks); i += 3; col += 3; continue; }
        if rest.starts_with(">==") { push(Tok::Op(">==".into()), &mut toks); i += 3; col += 3; continue; }
        if two == "::" { push(Tok::ColonColon, &mut toks); i += 2; col += 2; continue; }
        if two == ":=" { push(Tok::Walrus, &mut toks); i += 2; col += 2; continue; }
        if two == "==" { push(Tok::Op("==".into()), &mut toks); i += 2; col += 2; continue; }
        if two == "!=" { return err(tl, tc, "`!=` is not an operator; not-equal is `!==`".into()); }
        if two == "<=" { return err(tl, tc, "`<=` is not an operator; less-or-equal is `<==`".into()); }
        if two == ">=" { return err(tl, tc, "`>=` is not an operator; greater-or-equal is `>==`".into()); }
        if two == "++" || two == "--" || two == "+=" || two == "-=" {
            return err(tl, tc, format!("`{two}` does not exist; write the assignment in full: 'i' = ['i' + 1];"));
        }

        let single = match c {
            '[' => Tok::LBracket, ']' => Tok::RBracket,
            '{' => Tok::LBrace, '}' => Tok::RBrace,
            '(' => Tok::LParen, ')' => Tok::RParen,
            '.' => Tok::Dot, '=' => Tok::Eq, ';' => Tok::Semi, ',' => Tok::Comma,
            '+' | '-' | '/' | '<' | '>' => Tok::Op(c.to_string()),
            '*' => return err(tl, tc, "`*` is not an operator; multiply is `x`, power is `xx`".into()),
            '^' => return err(tl, tc, "`^` is not an operator; power is `xx`".into()),
            '%' => return err(tl, tc, "`%` is not an operator; remainder is `mod`".into()),
            '&' | '|' | '!' => return err(tl, tc, format!("`{c}` is not an operator; use `and`, `or`, `not`")),
            '#' => return err(tl, tc, "comments are not designed yet; the oracle accepts none".into()),
            other => return err(tl, tc, format!("unexpected character `{other}`")),
        };
        push(single, &mut toks);
        i += 1; col += 1;
    }
    toks.push(Token { tok: Tok::Eof, line, col });
    Ok(toks)
}
