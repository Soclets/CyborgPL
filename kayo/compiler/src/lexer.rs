//! The lexer. Three quote styles, three token classes: `"…"` is text,
//! `'…'` is a name, `` `…` `` is one character. Bare words are keywords,
//! function, type and module names. A run of operator glyphs is one
//! operator (`operators.md`). Comments: `#`, `#wl`, `#Nl`, `#Nld`.

use crate::diag::{err, Diagnostic, Result};
use crate::source::{FileId, Span};

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    /// `'name'` — a user-named value.
    Name(String),
    /// `"text"` — decoded (`\"` and `\\` only).
    Str(String),
    /// `` `c` `` — the characters between the backticks.
    Char(String),
    /// A number literal: the digits with `_` removed; `bin` if it had a `.`.
    /// `percent`: written with `%` or `percent` after it (owner,
    /// 2026-10-17), its text already over 100.
    Num { text: String, bin: bool, percent: bool },
    /// The bare `\n` outside quotes: a newline text.
    Newline,
    /// A bare word: keyword, function, type, module, `MAIN`.
    Word(String),
    /// A run of `! % & * + - / < > = ? @ ^ | ~`.
    Op(String),
    /// `{ raw }` after `inline.ASM`: the verbatim text between the braces.
    Raw(String),
    /// `#strict` (owner, 2026-09-27): the file writes every reason out.
    Strict,
    /// `#generated_N_lines_below` (owner, 2026-10-13): under a template,
    /// the N lines `kayo generate` wrote.
    Generated(u32),
    Walrus,      // :=
    ColonColon,  // ::
    Colon,       // :
    Dollar,      // $
    Dot,         // .
    Ellipsis,    // ...
    Comma,
    Semi,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    LParen,
    RParen,
    Eof,
}

impl Tok {
    pub fn describe(&self) -> String {
        match self {
            Tok::Name(n) => format!("'{}'", n.replace('\\', "\\\\").replace('\'', "\\'")),
            Tok::Str(_) => "a text".into(),
            Tok::Char(_) => "a character".into(),
            Tok::Num { text, .. } => text.clone(),
            Tok::Newline => "`newln`".into(),
            Tok::Word(w) => format!("`{}`", w),
            // The comparisons by their words (owner, 2026-10-07).
            Tok::Op(o) if o == "==" => "`equal_to`".to_string(),
            Tok::Op(o) if o == "!==" => "`not_equal`".to_string(),
            Tok::Op(o) if o == "</==" => "`at_most`".to_string(),
            Tok::Op(o) if o == ">/==" => "`at_least`".to_string(),
            Tok::Op(o) => format!("`{}`", o),
            Tok::Raw(_) => "a raw block".into(),
            Tok::Strict => "`#strict`".into(),
            Tok::Generated(n) => format!("`#generated_{}_lines_below`", n),
            Tok::Walrus => "`:=`".into(),
            Tok::ColonColon => "`::`".into(),
            Tok::Colon => "`:`".into(),
            Tok::Dollar => "`$`".into(),
            Tok::Dot => "`.`".into(),
            Tok::Ellipsis => "`...`".into(),
            Tok::Comma => "`,`".into(),
            Tok::Semi => "`;`".into(),
            Tok::LBracket => "`[`".into(),
            Tok::RBracket => "`]`".into(),
            Tok::LBrace => "`{`".into(),
            Tok::RBrace => "`}`".into(),
            Tok::LParen => "`(`".into(),
            Tok::RParen => "`)`".into(),
            Tok::Eof => "the end of the file".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
    /// The first token on its line: an interface's types go one per line
    /// (owner, 2026-10-02).
    pub line_first: bool,
    /// Written in an old spelling (the 2026-10-07 revision; MTM's before
    /// it, owner: "MTM is gone."): what was written and the spelling that
    /// replaces it, which every Kit is refused, naming it.
    pub alt: Option<(String, &'static str)>,
    /// A word written with `:` after it inside `[ ]` (owner, 2026-10-20:
    /// `value: 2`, `from: pos 2`, `i32[min: 0]`): `argwords::labels` takes
    /// the `:` out and marks the word.
    pub labelled: bool,
    /// A `[` that was a call's `(` (owner, 2026-10-22: "`( )` runs
    /// something; `[ ]` describes or holds something"): the parser writes a
    /// call's `( … )` as `[ … ]`, marked, so the rest reads one kind.
    pub call: bool,
}

pub const OP_CHARS: &str = "!%&*+-/<>=?@^|~";

/// A number's digits with the point two places left: `80` is `0.80`,
/// `0.0001` is `0.000001`, `12345` is `123.45`.
fn over_100(text: &str) -> String {
    let (whole, frac) = text.split_once('.').unwrap_or((text, ""));
    let digits = format!("{}{}", whole, frac);
    let point = whole.len() as isize - 2;
    let (int, rest) = if point > 0 {
        (digits[..point as usize].trim_start_matches('0').to_string(), digits[point as usize..].to_string())
    } else {
        (String::new(), format!("{}{}", "0".repeat((-point) as usize), digits))
    };
    format!("{}.{}", if int.is_empty() { "0" } else { &int }, rest)
}

pub fn is_op_char(c: char) -> bool {
    OP_CHARS.contains(c)
}

/// Any character past ASCII that is not a space is a name's (owner,
/// 2026-10-06): every script and emoji, which WSTN lets a Kit use; without
/// it the parser refuses the name (Signatar's are English).
pub fn is_word_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || (!c.is_ascii() && !c.is_whitespace())
}

pub fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || (!c.is_ascii() && !c.is_whitespace())
}

struct Lexer<'a> {
    file: FileId,
    src: &'a str,
    pos: usize,
    toks: Vec<Token>,
    /// Byte offset where the current line began (for `#wl`).
    line_start: usize,
}

pub fn lex(file: FileId, src: &str) -> Result<Vec<Token>> {
    let mut lx = Lexer { file, src, pos: 0, toks: Vec::new(), line_start: 0 };
    lx.run()?;
    Ok(lx.toks)
}

impl<'a> Lexer<'a> {
    fn peek(&self) -> Option<char> {
        self.src[self.pos..].chars().next()
    }
    fn peek_at(&self, n: usize) -> Option<char> {
        self.src[self.pos..].chars().nth(n)
    }
    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }
    fn span(&self, start: usize) -> Span {
        Span::new(self.file, start, self.pos)
    }
    fn push(&mut self, tok: Tok, start: usize) {
        let span = self.span(start);
        let line_first = self.toks.last().map_or(true, |t| self.src.get(t.span.end as usize..start).is_some_and(|gap| gap.contains('\n')));
        self.toks.push(Token { tok, span, line_first, alt: None, labelled: false, call: false });
    }
    fn push_alt(&mut self, tok: Tok, start: usize, alt: Option<(String, &'static str)>) {
        self.push(tok, start);
        self.toks.last_mut().unwrap().alt = alt;
    }
    fn fail<T>(&self, start: usize, name: &'static str, msg: impl Into<String>) -> Result<T> {
        err(Diagnostic::error(msg).at(self.span(start)).named(name))
    }

    /// Skip to the end of the current line (not past the newline).
    fn skip_line(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' {
                break;
            }
            self.bump();
        }
    }

    /// Skip `n` whole lines, including their newlines.
    fn skip_lines(&mut self, n: usize) {
        for _ in 0..n {
            self.skip_line();
            if self.peek() == Some('\n') {
                self.bump();
                self.line_start = self.pos;
            }
        }
    }

    fn run(&mut self) -> Result<()> {
        while let Some(c) = self.peek() {
            let start = self.pos;
            match c {
                '\n' => {
                    self.bump();
                    self.line_start = self.pos;
                }
                c if c.is_whitespace() => {
                    self.bump();
                }
                '#' => self.comment(start)?,
                '\'' => self.quoted(start)?,
                '"' => self.text(start)?,
                '`' => self.char_lit(start)?,
                '\\' => {
                    self.bump();
                    if self.peek() == Some('n') {
                        self.bump();
                        // `newln` since 2026-10-07 (owner).
                        self.push_alt(Tok::Newline, start, Some(("\\n".into(), "newln")));
                    } else {
                        return self.fail(start, "unexpected_text", "a bare backslash: only `\\n` is a value outside a text; inside a text only `\\\"` and `\\\\` escape");
                    }
                }
                c if c.is_ascii_digit() => self.number(start)?,
                c if is_word_start(c) => self.word(start)?,
                c if is_op_char(c) => {
                    while self.peek().map(is_op_char).unwrap_or(false) {
                        self.bump();
                    }
                    let text = self.src[start..self.pos].to_string();
                    let written = text.clone();
                    // `-->` and `<--` are the main spelling of the arrows,
                    // `->` and `<-` the others (owner, 2026-10-03): one
                    // token, so neither names a custom operator.
                    // `>>` and `>>>` spell `-->`, and `<<` and `<<<` spell `<--`
                    // (owner, 2026-10-06), wherever an arrow goes; the
                    // both-ways binding is `<-->`, or `<->`, `<<>>`, `<<<>>>`
                    // and `<---->` ("Two long arrows"), and with a wall between,
                    // `<--|-->`, `<-|->`, `<<|>>`, `<<<|>>>` (owner: "`b`").
                    let text = match (text.as_str(), text.strip_prefix("-->"), text.strip_prefix("<--")) {
                        ("<-->" | "<<>>" | "<<<>>>" | "<---->" | "<--|-->" | "<-|->" | "<<|>>" | "<<<|>>>", _, _) => "<->".to_string(),
                        (">>" | ">>>", _, _) => "->".to_string(),
                        ("<<" | "<<<", _, _) => "<-".to_string(),
                        (_, Some(rest), _) => format!("->{}", rest),
                        (_, _, Some(rest)) => format!("<-{}", rest),
                        _ => text,
                    };
                    // The arrows are short again, `->`, `<-`, `<->` (owner,
                    // 2026-10-07); the others are old spellings. So are the
                    // comparisons' glyphs, words since that day.
                    let main = match text.as_str() {
                        "->" => Some("->"),
                        "<-" => Some("<-"),
                        "<->" => Some("<->"),
                        "!==" => Some("not_equal"),
                        "</==" => Some("at_most"),
                        ">/==" => Some("at_least"),
                        _ => None,
                    };
                    let alt = main.filter(|m| *m != written).map(|m| (written, m));
                    self.push_alt(Tok::Op(text), start, alt);
                }
                ':' => {
                    self.bump();
                    match self.peek() {
                        Some('=') => {
                            self.bump();
                            self.push(Tok::Walrus, start);
                        }
                        Some(':') => {
                            self.bump();
                            self.push(Tok::ColonColon, start);
                        }
                        _ => self.push(Tok::Colon, start),
                    }
                }
                // `$total` is the variable `'total'` (owner, 2026-10-22: "Why
                // `$`? … it's easier to write"): letters, digits and `_`.
                '$' => {
                    self.bump();
                    let from = self.pos;
                    while self.peek().is_some_and(|c| c.is_alphanumeric() || c == '_') {
                        self.bump();
                    }
                    if self.pos == from {
                        self.push(Tok::Dollar, start);
                    } else {
                        let name = self.src[from..self.pos].to_string();
                        self.push(Tok::Name(name), start);
                    }
                }
                '.' => {
                    self.bump();
                    if self.peek() == Some('.') && self.peek_at(1) == Some('.') {
                        self.bump();
                        self.bump();
                        self.push(Tok::Ellipsis, start);
                    } else {
                        self.push(Tok::Dot, start);
                    }
                }
                ',' => {
                    self.bump();
                    self.push(Tok::Comma, start);
                }
                ';' => {
                    self.bump();
                    self.push(Tok::Semi, start);
                }
                '[' => {
                    self.bump();
                    self.push(Tok::LBracket, start);
                }
                ']' => {
                    self.bump();
                    self.push(Tok::RBracket, start);
                }
                '{' => {
                    self.bump();
                    if self.after_inline_asm() {
                        self.raw_block(start)?;
                    } else {
                        self.push(Tok::LBrace, start);
                    }
                }
                '}' => {
                    self.bump();
                    self.push(Tok::RBrace, start);
                }
                '(' => {
                    self.bump();
                    self.push(Tok::LParen, start);
                }
                ')' => {
                    self.bump();
                    self.push(Tok::RParen, start);
                }
                other => {
                    self.bump();
                    return self.fail(start, "unexpected_text", format!("`{}` is not part of the language", other));
                }
            }
        }
        let end = self.src.len();
        self.toks.push(Token { tok: Tok::Eof, span: Span::new(self.file, end, end), line_first: true, alt: None, labelled: false, call: false });
        Ok(())
    }

    /// `inline` `.` and a language were the last three tokens: the brace
    /// opens raw text (owner, 2026-10-07: `inline.ASM`, `inline.LL`,
    /// `inline.MSL`, and MTM's whole names for them).
    fn after_inline_asm(&self) -> bool {
        let n = self.toks.len();
        n >= 3
            && matches!(&self.toks[n - 1].tok, Tok::Word(w) if matches!(w.as_str(), "ASM" | "Assembly" | "S" | "LL" | "LLVM" | "LLVM_Intermediate_Representation" | "MSL" | "Metal_Shading_Language"))
            && self.toks[n - 2].tok == Tok::Dot
            && matches!(&self.toks[n - 3].tok, Tok::Word(w) if w == "inline")
    }

    fn raw_block(&mut self, start: usize) -> Result<()> {
        let body_start = self.pos;
        let mut depth = 1;
        loop {
            match self.bump() {
                None => return self.fail(start, "unclosed_block", "this `{` is never closed"),
                Some('{') => depth += 1,
                Some('}') => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                Some(_) => {}
            }
        }
        let text = self.src[body_start..self.pos - 1].to_string();
        self.push(Tok::Raw(text), start);
        Ok(())
    }

    fn comment(&mut self, start: usize) -> Result<()> {
        self.bump(); // '#'
        // A template's generated part (owner, 2026-10-13): a marker, not a
        // comment, the number being the lines under it.
        if self.src[self.pos..].starts_with("generated_") {
            let word_end = self.src[self.pos..].find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).map_or(self.src.len(), |e| self.pos + e);
            let word = &self.src[self.pos..word_end];
            let n = word.strip_prefix("generated_").and_then(|r| r.strip_suffix("_lines_below")).and_then(|d| d.parse::<u32>().ok());
            let Some(n) = n else {
                return self.fail(start, "comment_form", format!("`#{}` is not a marker: a template's generated part is marked `#generated_N_lines_below`, N the lines under it", word));
            };
            while self.pos < word_end {
                self.bump();
            }
            self.push(Tok::Generated(n), start);
            return Ok(());
        }
        // The form is whatever letters and digits follow the `#` before a space.
        let form_start = self.pos;
        while self.peek().map(|c| c.is_ascii_alphanumeric()).unwrap_or(false) {
            self.bump();
        }
        let form = &self.src[form_start..self.pos];
        let ends_form = matches!(self.peek(), None | Some(' ') | Some('\t') | Some('\n') | Some('\r'));
        if form.is_empty() {
            self.skip_line();
            return Ok(());
        }
        if !ends_form {
            return self.fail(start, "comment_form", format!("`#{}` is not a comment form: `#` to the end of the line, `#wl` the whole line, `#Nl` N lines, `#Nld` this line and N down; `#strict` is not a comment", form));
        }
        // Not a comment: `#strict` is one of the `#word` forms, like `#wl`.
        if form == "strict" {
            self.push(Tok::Strict, start);
            return Ok(());
        }
        if form == "wl" {
            // The whole line, including the code before the `#`.
            while let Some(t) = self.toks.last() {
                if t.span.start as usize >= self.line_start {
                    self.toks.pop();
                } else {
                    break;
                }
            }
            self.skip_line();
            return Ok(());
        }
        let (digits, suffix) = form.split_at(form.find(|c: char| !c.is_ascii_digit()).unwrap_or(form.len()));
        let n: usize = match digits.parse() {
            Ok(n) if !digits.is_empty() => n,
            _ => return self.fail(start, "comment_form", format!("`#{}` is not a comment form: `#` to the end of the line, `#wl` the whole line, `#Nl` N lines, `#Nld` this line and N down; `#strict` is not a comment", form)),
        };
        match suffix {
            "l" => {
                // N lines counting this one: the rest of this line and N-1 more.
                self.skip_line();
                if n > 1 {
                    if self.peek() == Some('\n') {
                        self.bump();
                        self.line_start = self.pos;
                    }
                    self.skip_lines(n - 1);
                    // skip_lines consumed the newline after the last skipped line; keep it
                    // simple: the last line's newline is consumed, that is fine.
                }
            }
            "ld" => {
                // This line and N more down.
                self.skip_line();
                if self.peek() == Some('\n') {
                    self.bump();
                    self.line_start = self.pos;
                }
                self.skip_lines(n);
            }
            _ => return self.fail(start, "comment_form", format!("`#{}` is not a comment form: `#` to the end of the line, `#wl` the whole line, `#Nl` N lines, `#Nld` this line and N down; `#strict` is not a comment", form)),
        }
        Ok(())
    }

    fn quoted(&mut self, start: usize) -> Result<()> {
        self.bump(); // '
        // A quoted name holds any printable character (owner, 2026-10-16:
        // "Yeah, allow it", then "Allow 1, 2, and 4"): symbols, a `.`
        // (`'v1.2'`, another name than `'v1'.2`), single spaces between
        // words, and a `'` written `\'`, a `\` written `\\`.
        let mut name = String::new();
        loop {
            match self.bump() {
                None | Some('\n') => return self.fail(start, "unclosed_quote", "this `'` never closes: a name is written between two single quotes on one line"),
                Some('\'') => break,
                Some('\\') => match self.bump() {
                    Some(c @ ('\'' | '\\')) => name.push(c),
                    _ => return self.fail(start, "quoted_name_form", "in a quoted name, `\\` comes before `'` or `\\`: `'it\\'s'`, `'a\\\\b'`"),
                },
                Some(c) if c == ' ' || !c.is_whitespace() && !c.is_control() => name.push(c),
                Some(_) => return self.fail(start, "quoted_name_form", "a quoted name holds printable characters and single spaces: no tab, line break or control character"),
            }
        }
        if name.is_empty() {
            return self.fail(start, "quoted_name_form", "a name cannot be empty: write something between the quotes");
        }
        if name.starts_with(' ') || name.ends_with(' ') || name.contains("  ") {
            return self.fail(start, "quoted_name_form", "a quoted name's spaces come one at a time, between words: `'total due'`");
        }
        self.push(Tok::Name(name), start);
        Ok(())
    }

    fn text(&mut self, start: usize) -> Result<()> {
        self.bump(); // "
        let mut out = String::new();
        loop {
            match self.bump() {
                None => return self.fail(start, "unclosed_quote", "this `\"` never closes: a text ends with a second `\"`"),
                Some('"') => break,
                Some('\\') => match self.peek() {
                    Some('"') => {
                        self.bump();
                        out.push('"');
                    }
                    Some('\\') => {
                        self.bump();
                        out.push('\\');
                    }
                    // Anything else after a backslash is literal (strings.md).
                    _ => out.push('\\'),
                },
                Some(c) => out.push(c),
            }
        }
        self.push(Tok::Str(out), start);
        Ok(())
    }

    fn char_lit(&mut self, start: usize) -> Result<()> {
        self.bump(); // `
        let body = self.pos;
        loop {
            match self.bump() {
                None | Some('\n') => return self.fail(start, "unclosed_quote", "this `` ` `` never closes: one character is written between two backticks"),
                Some('`') => break,
                Some(_) => {}
            }
        }
        let text = self.src[body..self.pos - 1].to_string();
        if text.is_empty() {
            return self.fail(start, "character_empty", "a character literal cannot be empty");
        }
        self.push(Tok::Char(text), start);
        Ok(())
    }

    /// The digits after `bin`, `hex` or `oct`, as the decimal number
    /// they are.
    fn based(&mut self, start: usize, word: &str, radix: u32) -> Result<()> {
        let digits_at = self.pos;
        while self.peek().is_some_and(|c| c.is_ascii_alphanumeric() || c == '_') {
            self.bump();
        }
        let written = &self.src[digits_at..self.pos];
        // `_` groups digits: one, between two of them.
        if written.starts_with('_') || written.ends_with('_') || written.contains("__") {
            let shown = match radix {
                2 => "bin 1_0",
                8 => "oct 7_0",
                _ => "hex F_0",
            };
            return self.fail(start, "digit_grouping", format!("`_` groups digits: one between two of them (`{}`)", shown));
        }
        let mut value = crate::bigint::Big::zero();
        let base = crate::bigint::Big::from_i128(radix as i128);
        for c in written.chars().filter(|c| *c != '_') {
            let Some(d) = c.to_digit(radix) else {
                return self.fail(start, "digit_outside_base", match radix {
                    2 => format!("a binary number has only the digits 0 and 1: `{}` in `{} {}`", c, word, written),
                    8 => format!("an octal number has only the digits 0 to 7: `{}` in `{} {}`", c, word, written),
                    _ => format!("a hex number has only 0-9 and A-F: `{}` in `{} {}`", c, word, written),
                });
            };
            value = value.mul(&base).add(&crate::bigint::Big::from_i128(d as i128));
        }
        self.push(Tok::Num { text: value.to_string(), bin: false, percent: false }, start);
        Ok(())
    }

    fn number(&mut self, start: usize) -> Result<()> {
        // `0x1F`, `0b1010`, `0o755`: another base is a word in front now
        // (owner, 2026-10-19).
        if self.peek() == Some('0') && matches!(self.peek_at(1), Some('x' | 'X' | 'b' | 'B' | 'o' | 'O')) && self.peek_at(2).is_some_and(|c| c.is_ascii_alphanumeric()) {
            let word = match self.peek_at(1) {
                Some('x' | 'X') => "hex",
                Some('b' | 'B') => "bin",
                _ => "oct",
            };
            self.bump();
            self.bump();
            let at = self.pos;
            while self.peek().is_some_and(|c| c.is_ascii_alphanumeric() || c == '_') {
                self.bump();
            }
            let digits = self.src[at..self.pos].to_string();
            return self.fail(start, "foreign_spelling", format!("a number in another base is written with a word in front: `{} {}` (owner, 2026-10-19)", word, digits));
        }
        let mut text = String::new();
        let mut bin = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                text.push(c);
                self.bump();
            } else if c == '_' {
                self.bump();
            } else if c == '.' && !bin && self.peek_at(1).map(|d| d.is_ascii_digit()).unwrap_or(false) {
                bin = true;
                text.push('.');
                self.bump();
            } else {
                break;
            }
        }
        // Leading zeros are illegal (2026-09-17): `007`, `000`.
        let whole = text.split('.').next().unwrap();
        if whole.len() > 1 && whole.starts_with('0') {
            let trimmed = whole.trim_start_matches('0');
            let trimmed = if trimmed.is_empty() { "0" } else { trimmed };
            return self.fail(start, "leading_zero", format!("`{}` starts with a zero, and leading zeros are illegal: write `{}`", &self.src[start..self.pos], trimmed));
        }
        // `1,000` is an error pointing at `_`, not the list [1, 0].
        if self.peek() == Some(',') && self.peek_at(1).map(|d| d.is_ascii_digit()).unwrap_or(false) {
            let at = self.pos;
            self.bump();
            return err(Diagnostic::error("`,` is not a digit separator — use `_` (1_000_000); a comma separates values, and one directly between digits reads as neither").named("digit_grouping").at(Span::new(self.file, at, at + 1)));
        }
        // A percentage (owner, 2026-10-16): `80%` or `80 percent` is the
        // number over 100, a bin as `0.80` is, and `exactly 80%` a deci. The
        // `%` touches the number, and is the old `%` of `mod` when an
        // operator, a digit, `'` or `(` follows it (`7%2`); `percent` comes
        // after a space and runs into no letter.
        let sign = self.peek() == Some('%') && !self.peek_at(1).is_some_and(|c| is_op_char(c) || c.is_ascii_digit() || c == '\'' || c == '(');
        let spaces = (0..).take_while(|&k| self.peek_at(k) == Some(' ')).count();
        let word = spaces > 0 && self.src[self.pos + spaces..].starts_with("percent") && !self.src[self.pos + spaces + 7..].chars().next().is_some_and(is_word_char);
        if sign || word {
            for _ in 0..if sign { 1 } else { spaces + 7 } {
                self.bump();
            }
            self.push(Tok::Num { text: over_100(&text), bin: true, percent: true }, start);
            return Ok(());
        }
        // A number running straight into a letter: `1e3`, `0x10`, `5seconds`.
        if let Some(c) = self.peek() {
            if is_word_start(c) {
                return self.fail(start, "space_missing", format!("a number runs straight into `{}`: put a space between a number and a word (`5 secs`), and there is no `e` or `0x` form", c));
            }
        }
        self.push(Tok::Num { text, bin, percent: false }, start);
        Ok(())
    }

    fn word(&mut self, start: usize) -> Result<()> {
        while self.peek().map(is_word_char).unwrap_or(false) {
            self.bump();
        }
        let text = self.src[start..self.pos].to_string();
        // A whole number in another base (owner, 2026-10-19: "(a)"): a word
        // in front, then its digits, the same number a decimal one is.
        if let Some(radix) = match text.as_str() {
            // `bin` and `oct` since 2026-10-20 (owner: "(a)"); `binary` and
            // `octal` are old spellings, refused by `spellings.cyr`'s rows.
            "bin" => Some(2u32),
            "hex" => Some(16),
            "oct" => Some(8),
            _ => None,
        } {
            let spaces = (0..).take_while(|&k| matches!(self.peek_at(k), Some(' ') | Some('\t'))).count();
            if spaces > 0 && self.peek_at(spaces).is_some_and(|c| c.is_ascii_alphanumeric()) {
                for _ in 0..spaces {
                    self.bump();
                }
                return self.based(start, &text, radix);
            }
        }
        // The 2026-10-07 revision's words (owner), read as the tokens the
        // compiler always used: `no_check` is the old `!check`, the
        // comparisons are their glyphs, `newln` the newline piece, `stdkit`
        // the Kit that was `std`.
        let tok = match text.as_str() {
            "no_check" => Tok::Word("nocheck".into()),
            "equal_to" => Tok::Op("==".into()),
            "not_equal" => Tok::Op("!==".into()),
            "at_most" => Tok::Op("</==".into()),
            "at_least" => Tok::Op(">/==".into()),
            "newln" => Tok::Newline,
            // The standard library is `cord` (owner, 2026-10-09: "yea,
            // go"), held as the `std` it always was inside.
            "auto_newln" => Tok::Word("auto_newline".into()),
            _ => Tok::Word(text.clone()),
        };
        // Its old spellings, refused naming the new (owner: "MTM is gone."),
        // as `spellings.cyr` lists the ones that stand anywhere as a word.
        // A word the table also lists with a dot after it (`limb.` is
        // `my.`) is that entry when a dot follows, found where paths are read.
        let dotted = self.peek() == Some('.') && crate::spellings::old_dotted(&text);
        let alt = if dotted { None } else { crate::spellings::old_word(&text, None).and_then(|o| o.write.first()).map(|w| (text.clone(), w.as_str())) };
        self.push_alt(tok, start, alt);
        Ok(())
    }
}
