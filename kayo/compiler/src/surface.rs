//! Redesign three (owner, 2026-10-26 to 2026-10-28; `design/revision-2026-10-27.md` and
//! `design/revision-2026-10-28.md`): the one spelling, read from the tokens as written into the
//! tokens the parser has always read.
//!
//! A block's opening line ends in `:` and it closes with `;`, its kind and its name (`;fn add`,
//! `;if`, `;%layout.C rec Rect`); an `if` or `on` chain closes once, its `else` lines inside it.
//! `let` never changes and `var` does, `:=` defines and `=` starts a `var`, `change $x to …`; a
//! parameter says `let`, `var`, `lent` or `lent.mut`; `rec`, `union.tag`, `sig` / `set`, `ret`,
//! `ls`, `arr`; `?` and `??` stand for `otherwise fail` and `otherwise`; `*` marks a jumping
//! handler where it is used. Each is checked here as written, and the old spelling of each is
//! refused naming the new one, as `spellings.cyr` lists them.
//!
//! The parser behind it reads blocks in braces and the words it always did: a `:` at a line's end
//! becomes `{`, a closer `}`, each with the span of what was written, so a message points at the
//! text.

use crate::diag::{err, Diagnostic, Edit, Result};
use crate::lexer::{Tok, Token};
use crate::source::Span;

/// What a block that is open says about the lines inside it.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    /// Statements.
    Body,
    /// An `if` or `on` chain: an `else` line closes one branch and opens the next.
    Chain,
    /// A `rec`: each line a field, `let` or `var`.
    Rec,
    /// A `union.tag` or `%layout.C enum` / `union`: each line a case.
    Cases,
    /// A `sig`: operations only.
    Sig,
    /// A `set`: types only.
    Set,
    /// An inline block: raw text, then the closer.
    Inline,
    /// The items of a file: a `module`, `%extern.C`, a collection, `versions`, a template.
    Items,
    /// A line this pass cannot read as a block's: the parser says what is wrong with it, and any
    /// closer ends it.
    Unknown,
}

struct Open {
    /// What the closer says after `;`: `fn add`, `if`, `%layout.C rec Rect`.
    closer: String,
    /// The opening line's first token.
    at: Span,
    kind: Kind,
    /// An inline block's `with [ … ]`, which the parser reads after the code.
    with: Vec<Token>,
    /// A `sig`'s first type line, and whether it lists an operation.
    types: Option<Span>,
    ops: bool,
}

pub fn read(toks: Vec<Token>, src: &str) -> Result<Vec<Token>> {
    let mut r = Reader { src, toks, out: Vec::new(), stack: Vec::new() };
    r.run()?;
    Ok(r.out)
}

struct Reader<'a> {
    src: &'a str,
    toks: Vec<Token>,
    out: Vec<Token>,
    stack: Vec<Open>,
}

fn tok(t: Tok, span: Span) -> Token {
    Token { tok: t, span, line_first: false, alt: None, labelled: false, call: false, raw: None, marked: false, proc: false }
}

fn word(w: &str, span: Span) -> Token {
    tok(Tok::Word(w.into()), span)
}

fn is_word(t: &Token, w: &str) -> bool {
    matches!(&t.tok, Tok::Word(x) if x == w)
}

fn is_op(t: &Token, o: &str) -> bool {
    matches!(&t.tok, Tok::Op(x) if x == o)
}

fn refuse<T>(name: &'static str, msg: impl Into<String>, at: Span) -> Result<T> {
    err(Diagnostic::error(msg).named(name).at(at))
}

/// The ruling's date after an old spelling's sentence, as `spellings.cyr` gives it.
fn old_said(written: &str, write: &str) -> String {
    match crate::spellings::old_word(written, Some("special")) {
        Some(o) => o.dated(),
        None => format!("`{}` is an old spelling: write `{}`", written, write),
    }
}

impl<'a> Reader<'a> {
    fn text(&self, s: Span) -> &str {
        &self.src[s.start as usize..s.end as usize]
    }

    /// The source between two tokens' starts and ends, its spaces each one space.
    fn spelled(&self, from: Span, to: Span) -> String {
        let t = &self.src[from.start as usize..to.end as usize];
        t.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    fn run(&mut self) -> Result<()> {
        // `$'total due'`: one name, of any characters.
        let mut merged: Vec<Token> = Vec::with_capacity(self.toks.len());
        let mut k = 0;
        while k < self.toks.len() {
            if self.toks[k].tok == Tok::Dollar && self.toks.get(k + 1).is_some_and(|n| matches!(n.tok, Tok::Name(_)) && n.span.start == self.toks[k].span.end) {
                let mut nt = self.toks[k + 1].clone();
                nt.span = self.toks[k].span.to(nt.span);
                nt.line_first = self.toks[k].line_first;
                merged.push(nt);
                k += 2;
                continue;
            }
            merged.push(self.toks[k].clone());
            k += 1;
        }
        self.toks = merged;
        let n = self.toks.len();
        let mut i = 0;
        while i < n {
            let t = self.toks[i].clone();
            if t.tok == Tok::Eof {
                if let Some(open) = self.stack.last() {
                    return refuse("unclosed_block", format!("this `{}` is never closed: it ends at a line `;{}`", open.closer, open.closer), open.at);
                }
                self.out.push(t);
                break;
            }
            // A closer, `;` first on its line, and what follows it on that line.
            if t.tok == Tok::Semi && t.line_first {
                let mut j = i + 1;
                while j < n && !self.toks[j].line_first {
                    j += 1;
                }
                let line = self.toks[i..j].to_vec();
                self.line(line)?;
                i = j;
                continue;
            }
            if let Tok::Raw(_) | Tok::Generated(_) = t.tok {
                self.out.push(t);
                i += 1;
                continue;
            }
            // One statement: to its `;`, or to the `:` ending its line that opens a block. A `-`
            // ending a line goes on to the opening line below it.
            let mut j = i;
            let mut depth = 0i32;
            let mut stmt: Vec<Token> = Vec::new();
            let mut joins: Vec<Span> = Vec::new();
            while j < n {
                let u = &self.toks[j];
                if j > i && u.line_first && (u.tok == Tok::Semi || matches!(u.tok, Tok::Raw(_))) {
                    break;
                }
                if u.tok == Tok::Eof {
                    break;
                }
                let next_first = self.toks.get(j + 1).map_or(true, |x| x.line_first);
                match &u.tok {
                    Tok::LBracket | Tok::LParen => depth += 1,
                    Tok::RBracket | Tok::RParen => depth -= 1,
                    _ => {}
                }
                if is_op(u, "-") && next_first && j > i {
                    joins.push(u.span);
                    j += 1;
                    continue;
                }
                stmt.push(u.clone());
                j += 1;
                if u.tok == Tok::Semi && depth <= 0 {
                    break;
                }
                if u.tok == Tok::Colon && next_first {
                    break;
                }
                // `else` always starts its own statement.
                if self.toks.get(j).is_some_and(|x| x.line_first && is_word(x, "else")) && depth <= 0 {
                    break;
                }
            }
            let opens = stmt.last().is_some_and(|t| t.tok == Tok::Colon);
            if let (Some(at), false) = (joins.first(), opens) {
                return refuse("line_join_misplaced", "a line-end `-` joins a line to the block's opening line below it (`…(lend $row)-` / `otherwise:`); a statement goes on to the next line by itself, up to its `;` (owner, 2026-10-27)", *at);
            }
            if stmt.is_empty() {
                i = j.max(i + 1);
                continue;
            }
            self.line(stmt)?;
            i = j;
        }
        Ok(())
    }

    fn line(&mut self, mut line: Vec<Token>) -> Result<()> {
        // An inline block's code.
        if let Tok::Raw(_) = line[0].tok {
            self.out.extend(line);
            return Ok(());
        }
        // A closer: `;` and the kind and name of the block it closes.
        if line[0].tok == Tok::Semi {
            let mut k = 1;
            while k < line.len() && !matches!(line[k].tok, Tok::RParen | Tok::Semi | Tok::Comma | Tok::RBracket) {
                k += 1;
            }
            if k == 1 {
                return refuse("unexpected_text", "a `;` alone on a line: a block closes with `;` and its kind and name, `;if`, `;fn add`", line[0].span);
            }
            let said = self.spelled(line[1].span, line[k - 1].span);
            let at = line[0].span.to(line[k - 1].span);
            let Some(open) = self.stack.pop() else {
                return refuse("unexpected_text", format!("`;{}` closes nothing: no block is open here", said), at);
            };
            // A `sig` of types only is a `set`; one with both is two (owner, 2026-10-27).
            if open.kind == Kind::Sig {
                if let Some(at) = open.types {
                    let msg = if open.ops { "a `sig` lists operations only; put its types in a `set` (owner, 2026-10-27)" } else { "a list of types only is a `set`: `set Number:` (owner, 2026-10-27)" };
                    return refuse("interface_form", msg, at);
                }
            }
            if said != open.closer && open.kind != Kind::Unknown {
                let d = Diagnostic::error(format!("`;{}` does not close the `{}` that is open: write `;{}`", said, open.closer, open.closer)).named("closer_mismatch").at(at).why_at(open.at, format!("the `{}` opens here", open.closer));
                return err(d.fix_edit(line[1].span.to(line[k - 1].span), Edit::Replace, open.closer.clone()));
            }
            if open.kind == Kind::Inline {
                self.out.extend(open.with);
                self.out.push(tok(Tok::Semi, at));
            } else {
                let mut t = tok(Tok::RBrace, at);
                t.line_first = true;
                self.out.push(t);
            }
            let rest = line.split_off(k);
            if !rest.is_empty() {
                let kind = self.stack.last().map_or(Kind::Items, |o| o.kind);
                let words = self.words(rest, kind)?;
                self.out.extend(words);
            }
            return Ok(());
        }
        let in_kind = self.stack.last().map_or(Kind::Items, |o| o.kind);
        // `!MAIN`, `!no_check`, `!case`: the attention mark is gone (redesign three).
        if is_op(&line[0], "!") {
            if let Some(Tok::Word(w)) = line.get(1).map(|t| &t.tok) {
                let shown = if w == "nocheck" { "no_check".to_string() } else { w.clone() };
                return err(Diagnostic::error(format!("`!` is gone (owner, 2026-10-27: \"I'm feeling like removing that `!` meaning\"): write `{}`, with no `!`", shown)).named("old_spelling").at(line[0].span).fix_edit(line[0].span, Edit::Replace, ""));
            }
        }
        // `end of if`: a block closes with `;` and its kind and name (redesign three).
        if is_word(&line[0], "end") && line.get(1).is_some_and(|t| is_word(t, "of")) {
            let last = line.iter().rposition(|t| !matches!(t.tok, Tok::Semi)).unwrap_or(1);
            let said = self.spelled(line[0].span, line[last].span);
            let rest = self.spelled(line[2.min(last)].span, line[last].span);
            return err(Diagnostic::error(format!("`{}` is an old spelling: write `;{}` (owner, 2026-10-28)", said, rest)).named("old_spelling").at(line[0].span.to(line[last].span)).fix_edit(line[0].span.to(line[last].span), Edit::Replace, format!(";{}", rest)));
        }
        // A `set` lists types; a `sig` operations (owner, 2026-10-27).
        let op_line = matches!(&line[0].tok, Tok::Word(w) if w == "op" || w == "fn" || w == "proc");
        if in_kind == Kind::Set && op_line {
            return refuse("interface_form", "a `set` lists types only; operations go in a `sig` (owner, 2026-10-27)", line[0].span);
        }
        if in_kind == Kind::Sig {
            let open = self.stack.last_mut().unwrap();
            if op_line {
                open.ops = true;
            } else if open.types.is_none() {
                open.types = Some(line[0].span);
            }
        }
        // A field or a case ends with `;` (owner, 2026-10-27).
        if matches!(in_kind, Kind::Rec | Kind::Cases) && line.last().is_some_and(|t| t.tok == Tok::Comma) {
            return refuse("braces_need_commas", "fields and cases end with `;`, not `,` (owner, 2026-10-27)", line.last().unwrap().span);
        }
        if let Some(k) = (0..line.len().saturating_sub(3)).find(|&k| line[k].marked && is_word(&line[k], "extern") && line[k + 1].tok == Tok::Dot && is_word(&line[k + 2], "C") && matches!(line[k + 3].tok, Tok::Name(_))) {
            let n = &line[k + 3];
            return err(Diagnostic::error(format!("a C variable says whether it can change: `%extern.C let {}: …;` (C's `const`, read only) or `%extern.C var {}: …;` (owner, 2026-10-27)", self.text(n.span), self.text(n.span))).named("declaration_form").at(n.span).fix_edit(n.span, Edit::Insert, "let "));
        }
        if let (Tok::Name(n), true) = (&line[0].tok, self.stack.last().is_some_and(|o| o.closer == "%extern.C")) {
            return err(Diagnostic::error(format!("a C variable says whether it can change: `let ${}: …;` (C's `const`, read only) or `var ${}: …;` (owner, 2026-10-27)", n, n)).named("declaration_form").at(line[0].span).fix_edit(line[0].span, Edit::Insert, "let "));
        }
        // A precedence level's line ends with `;` (owner, 2026-10-27).
        if is_word(&line[0], "level") && self.stack.last().is_some_and(|o| o.closer.starts_with("precedence")) && line.last().is_some_and(|t| t.tok != Tok::Semi) {
            let at = line.last().unwrap().span;
            return err(Diagnostic::error("a level's line ends with `;`: `level 2 [+, -] left_first;`").named("unexpected_text").at(at).fix_edit(Span::new(at.file, at.end as usize, at.end as usize), Edit::Insert, ";"));
        }
        self.signatures(&mut line, in_kind)?;
        // `else` goes on with the chain: the branch before it closes.
        let mut reopen = false;
        if is_word(&line[0], "else") {
            if in_kind != Kind::Chain {
                return refuse("unexpected_text", "an `else` continues its `if`: no `;if` before it; `;if` comes once, after the last branch (owner, 2026-10-27)", line[0].span);
            }
            let mut t = tok(Tok::RBrace, line[0].span);
            t.line_first = true;
            self.out.push(t);
            reopen = true;
        }
        let opens = line.last().is_some_and(|t| t.tok == Tok::Colon);
        if !opens {
            let words = self.words(line, in_kind)?;
            self.out.extend(words);
            return Ok(());
        }
        let colon = line.pop().unwrap();
        let (closer, kind) = self.closer_of(&line, in_kind)?;
        let first = line[0].span;
        // An inline block: its `with [ … ]` read after the code.
        if kind == Kind::Inline {
            let mut with = Vec::new();
            if let Some(w) = line.iter().position(|t| is_word(t, "with")) {
                with = line.split_off(w);
            }
            let head = self.words(line, in_kind)?;
            self.out.extend(head);
            let with = self.words(with, Kind::Body)?;
            self.stack.push(Open { closer, at: first, kind, with, types: None, ops: false });
            return Ok(());
        }
        // `block:` is a block alone, the parser's bare braces.
        if line.len() == 1 && is_word(&line[0], "block") {
            line.clear();
        }
        let words = self.words(line, in_kind)?;
        self.out.extend(words);
        self.out.push(tok(Tok::LBrace, colon.span));
        if !reopen {
            self.stack.push(Open { closer, at: first, kind, with: Vec::new(), types: None, ops: false });
        }
        Ok(())
    }

    /// What closes the block a line opens, and what its lines are.
    fn closer_of(&self, line: &[Token], in_kind: Kind) -> Result<(String, Kind)> {
        let mut i = 0;
        // Marks and words in front of the kind.
        loop {
            match line.get(i) {
                Some(t) if is_word(t, "pub") => {
                    i += 1;
                    if line.get(i).is_some_and(|t| t.tok == Tok::Dot) {
                        i += 2;
                    }
                }
                Some(t) if is_word(t, "bare") => {
                    i += 1;
                    while line.get(i).is_some_and(|t| t.tok == Tok::Dot) {
                        i += 2;
                    }
                }
                Some(t) if is_word(t, "table_bytes") => i += 2,
                Some(t) if (is_word(t, "nocheck") || is_word(t, "check") || is_word(t, "NO_FUCKING_CHECKS") || is_word(t, "no_skip")) && i + 1 < line.len() => i += 1,
                _ => break,
            }
        }
        let rest = &line[i..];
        let Some(head) = rest.first() else {
            return refuse("unexpected_text", "a line that is only `:`", line[0].span);
        };
        // A dotted word from `k`: its tokens' text and how many tokens.
        let dotted = |k: usize| -> (String, usize) {
            let mut n = 1;
            while rest.get(k + n).is_some_and(|t| t.tok == Tok::Dot) && rest.get(k + n + 1).is_some_and(|t| matches!(t.tok, Tok::Word(_))) {
                n += 2;
            }
            (self.spelled(rest[k].span, rest[k + n - 1].span), n)
        };
        let name_after = |k: usize| -> String {
            match rest.get(k) {
                Some(t) if matches!(t.tok, Tok::Word(_)) => dotted(k).0,
                _ => String::new(),
            }
        };
        let with_name = |kind: &str, k: usize| -> String {
            let n = name_after(k);
            if n.is_empty() { kind.to_string() } else { format!("{} {}", kind, n) }
        };
        let w = match &head.tok {
            Tok::Word(w) => w.clone(),
            Tok::LBracket if in_kind == Kind::Items => return Ok(("version".into(), Kind::Body)),
            _ => return Ok(match self.inner_closer(rest) {
                Some(c) => (c, Kind::Body),
                None => (String::new(), Kind::Unknown),
            }),
        };
        let marked = head.marked;
        Ok(match w.as_str() {
            "MAIN" => ("MAIN".into(), Kind::Body),
            "fn" | "proc" | "op" if marked => (dotted(0).0, Kind::Body),
            "fn" | "proc" => {
                let kw = if head.proc { "proc" } else { w.as_str() };
                (with_name(kw, 1), Kind::Body)
            }
            "op" => {
                let name = match rest.get(1) {
                    Some(t) if matches!(t.tok, Tok::Word(_)) => dotted(1).0,
                    Some(t) => self.text(t.span).to_string(),
                    None => String::new(),
                };
                (format!("op {}", name).trim().to_string(), Kind::Body)
            }
            "test" => {
                let kw = rest.get(1).map(|t| if t.proc { "proc".to_string() } else { self.text(t.span).to_string() }).unwrap_or_default();
                (format!("test {} {}", kw, name_after(2)).trim().to_string(), Kind::Body)
            }
            "rec" => (with_name("rec", 1), Kind::Rec),
            "union" if rest.get(1).is_some_and(|t| t.tok == Tok::Dot) && rest.get(2).is_some_and(|t| is_word(t, "tag")) => (with_name("union.tag", 3), Kind::Cases),
            "sig" => (with_name("sig", 1), Kind::Sig),
            "set" => (with_name("set", 1), Kind::Set),
            "layout" if marked => {
                // `%layout.C rec Rect`, `%layout.C enum Key`, `%layout.C union U`.
                let (lay, n) = dotted(0);
                let kind_word = rest.get(n).map(|t| self.text(t.span).to_string()).unwrap_or_default();
                let k = if kind_word == "rec" { Kind::Rec } else { Kind::Cases };
                (format!("{} {} {}", lay, kind_word, name_after(n + 1)).trim().to_string(), k)
            }
            "module" | "versions" => (with_name(&w, 1), Kind::Items),
            "coll" if !rest.get(1).is_some_and(|t| t.tok == Tok::Dot) => ("coll".into(), Kind::Unknown),
            "coll" => {
                let (c, n) = dotted(0);
                let kind = if c.ends_with(".bounded") || c.ends_with(".handler") { Kind::Items } else { Kind::Items };
                (format!("{} {}", c, name_after(n)).trim().to_string(), kind)
            }
            // A member, `member area.square […]:`, or a template's, `member […]:`.
            "member" => (with_name("member", 1), Kind::Body),
            "case" => match rest.get(1) {
                Some(t) if matches!(t.tok, Tok::Name(_)) => (format!("case {}", self.text(t.span)), Kind::Body),
                _ => (with_name("case", 1), Kind::Body),
            },
            "if" => ("if".into(), Kind::Chain),
            "else" => ("if".into(), Kind::Chain),
            "on" => (dotted(0).0, Kind::Chain),
            "for" if in_kind == Kind::Items => ("for".into(), Kind::Items),
            "for" | "while" | "loop" | "block" | "otherwise" | "match" | "whichever" | "nocheck" | "check" | "no_skip" | "NO_FUCKING_CHECKS" => {
                let shown = match w.as_str() {
                    "nocheck" => "no_check".to_string(),
                    _ if marked => self.text(head.span).to_string(),
                    _ => w.clone(),
                };
                if let Some(inner) = self.inner_closer(&rest[1..]) {
                    return Ok((inner, Kind::Body));
                }
                (shown, Kind::Body)
            }
            "inline" => {
                let lang = rest.get(2).map(|t| self.text(t.span).to_string()).unwrap_or_default();
                (format!("inline.{}", lang), Kind::Inline)
            }
            "precedence" => (dotted(0).0, Kind::Items),
            "extern" if marked => (dotted(0).0, Kind::Items),
            "spawn" | "concur" if marked => (dotted(0).0, Kind::Body),
            _ => match self.inner_closer(rest) {
                Some(c) => (c, Kind::Body),
                // The parser says what is wrong with it.
                None => (self.text(head.span).to_string(), Kind::Unknown),
            },
        })
    }

    /// Each parameter list on a line, as the parser reads it: a parameter's `let` is its `immut`
    /// and `var` its `mut`; an operand's `lent` is the parser's default, `let` its `give` and `var`
    /// its `give mut`; `MAIN`'s are `let`. A bare one is refused.
    fn signatures(&self, line: &mut Vec<Token>, in_kind: Kind) -> Result<()> {
        #[derive(PartialEq, Clone, Copy)]
        enum Ctx {
            Fn,
            Op,
            Main,
        }
        // Where each list opens, and whose it is.
        let mut opens: Vec<(usize, Ctx)> = Vec::new();
        let n = line.len();
        let mut k = 0;
        while k < n {
            let t = &line[k];
            let head = match &t.tok {
                Tok::Word(w) if (w == "fn" || w == "proc") && !matches!(k.checked_sub(1).map(|p| &line[p].tok), Some(Tok::Dot) | Some(Tok::ColonColon) | Some(Tok::Colon)) => Some(Ctx::Fn),
                Tok::Word(w) if w == "op" && !matches!(k.checked_sub(1).map(|p| &line[p].tok), Some(Tok::Dot) | Some(Tok::ColonColon)) => Some(Ctx::Op),
                Tok::Word(w) if w == "MAIN" && k == 0 => Some(Ctx::Main),
                Tok::Word(w) if w == "member" && k == 0 => Some(Ctx::Fn),
                Tok::LBracket if k == 0 && in_kind == Kind::Items => {
                    opens.push((0, Ctx::Fn));
                    k += 1;
                    continue;
                }
                _ => None,
            };
            if let Some(ctx) = head {
                // A function type's brackets, `fn[i32]`, hold types: glued to `fn`.
                let glued = line.get(k + 1).is_some_and(|b| b.tok == Tok::LBracket && b.span.start == t.span.end) && ctx == Ctx::Fn;
                if !glued {
                    // An operator's own spelling may be `<`: its list starts after it.
                    let mut j = if ctx == Ctx::Op { k + 2 } else { k + 1 };
                    loop {
                        while j < n && line[j].tok != Tok::LBracket && !matches!(line[j].tok, Tok::Semi | Tok::Colon | Tok::Walrus) {
                            // `<T: Number>`, the old list, holds a `:`.
                            if is_op(&line[j], "<") {
                                while j < n && !is_op(&line[j], ">") {
                                    j += 1;
                                }
                            }
                            j += 1;
                        }
                        // The placeholders' list, `[<T>: Addable]`, and the lifetimes', `[@x]`, come
                        // before the parameters' (owner, 2026-10-29).
                        let skipped = j < n && line[j].tok == Tok::LBracket && line.get(j + 1).is_some_and(|x| is_op(x, "<") || is_op(x, "@"));
                        if !skipped {
                            break;
                        }
                        let mut depth = 0i32;
                        while j < n {
                            match line[j].tok {
                                Tok::LBracket => depth += 1,
                                Tok::RBracket => {
                                    depth -= 1;
                                    if depth == 0 {
                                        break;
                                    }
                                }
                                _ => {}
                            }
                            j += 1;
                        }
                        j += 1;
                    }
                    if j < n && line[j].tok == Tok::LBracket {
                        opens.push((j, ctx));
                    }
                }
            }
            k += 1;
        }
        let mut edits: Vec<(usize, Vec<Token>, usize)> = Vec::new();
        for (open, ctx) in opens {
            let mut depth = 0i32;
            let mut j = open;
            let mut starts = Vec::new();
            while j < n {
                match &line[j].tok {
                    Tok::LBracket | Tok::LParen => {
                        depth += 1;
                        if depth == 1 {
                            starts.push(j + 1);
                        }
                    }
                    Tok::RBracket | Tok::RParen => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    Tok::Comma if depth == 1 => starts.push(j + 1),
                    _ => {}
                }
                j += 1;
            }
            for st in starts {
                let Some(t) = line.get(st) else { continue };
                let t = t.clone();
                let next = line.get(st + 1);
                match (&t.tok, ctx) {
                    (Tok::RBracket, _) => {}
                    (Tok::Word(w), Ctx::Op) if w == "lent" => {
                        if next.is_some_and(|n| n.tok == Tok::Dot) {
                            continue;
                        }
                        edits.push((st, Vec::new(), 1));
                    }
                    (Tok::Word(w), Ctx::Op) if w == "let" => edits.push((st, vec![word("give", t.span)], 1)),
                    (Tok::Word(w), Ctx::Op) if w == "var" => edits.push((st, vec![word("give", t.span), word("mut", t.span)], 1)),
                    (Tok::Word(w), Ctx::Op) if w == "give" => {
                        return err(Diagnostic::error("`give` on an operand is an old spelling: an operand taken whole says `let $a: T` (or `var $a: T`), one looked at `lent $a: T` (owner, 2026-10-27)").named("old_spelling").at(t.span).fix_edit(t.span, Edit::Replace, "let"));
                    }
                    (Tok::Name(nm), Ctx::Op) => {
                        return err(Diagnostic::error(format!("an operand says how it comes: `lent ${}: T` (looked at), `let ${}: T` or `var ${}: T` (taken whole) (owner, 2026-10-27)", nm, nm, nm)).named("parameter_form").at(t.span).fix_edit(t.span, Edit::Insert, "lent "));
                    }
                    (Tok::Word(w), Ctx::Main) if w == "var" => {
                        return err(Diagnostic::error("MAIN's parameters are C's and never change: `let $argc: i32`").named("main_parameters_wrong").at(t.span).fix_edit(t.span, Edit::Replace, "let"));
                    }
                    (Tok::Word(w), _) if w == "let" => edits.push((st, vec![word("immut", t.span)], 1)),
                    (Tok::Word(w), _) if w == "var" => edits.push((st, vec![word("mut", t.span)], 1)),
                    (Tok::Word(w), _) if w == "mut" || w == "immut" => {
                        let (said, new) = if w == "mut" { (old_said("mut", "var"), "var") } else { (old_said("immut", "let"), "let") };
                        return err(Diagnostic::error(said).named("old_spelling").at(t.span).fix_edit(t.span, Edit::Replace, new));
                    }
                    // A preset, `[$word = "hi"]`, names a parameter already said.
                    (Tok::Name(_), _) if next.is_some_and(|n| is_op(n, "=")) => {}
                    (Tok::Name(nm), _) => {
                        return err(Diagnostic::error(format!("a parameter says whether it changes: `let ${}` or `var ${}` (owner, 2026-10-27)", nm, nm)).named("parameter_form").at(t.span).fix_edit(t.span, Edit::Insert, "let "));
                    }
                    _ => {}
                }
            }
        }
        edits.sort_by_key(|e| std::cmp::Reverse(e.0));
        for (at, with, take) in edits {
            line.splice(at..at + take, with);
        }
        Ok(())
    }

    /// A block opened inside a line: a lambda `fn [ … ]:`, a value `match $x:`, a task.
    fn inner_closer(&self, rest: &[Token]) -> Option<String> {
        for k in (0..rest.len()).rev() {
            let t = &rest[k];
            match &t.tok {
                Tok::Word(w) if (w == "fn" || w == "proc") && rest.get(k + 1).is_some_and(|n| n.tok == Tok::LBracket || is_op(n, "<")) => {
                    return Some(if t.proc { "proc".into() } else { w.clone() });
                }
                Tok::Word(w) if w == "match" => return Some("match".into()),
                Tok::Word(w) if w == "otherwise" => return Some("otherwise".into()),
                Tok::Word(w) if t.marked && (w == "spawn" || w == "whichever") => {
                    let mut n = 1;
                    while rest.get(k + n).is_some_and(|t| t.tok == Tok::Dot) {
                        n += 2;
                    }
                    return Some(self.spelled(t.span, rest[k + n - 1].span));
                }
                _ => {}
            }
        }
        None
    }

    /// One line's tokens, its words read as the parser's.
    fn words(&mut self, line: Vec<Token>, in_kind: Kind) -> Result<Vec<Token>> {
        let mut out: Vec<Token> = Vec::with_capacity(line.len());
        let mut i = 0;
        let n = line.len();
        // Inside a field line of a `rec`, or a case line.
        let field_line = in_kind == Kind::Rec;
        while i < n {
            let t = &line[i];
            let prev = out.last().map(|t| t.tok.clone());
            let after_dot = matches!(prev, Some(Tok::Dot) | Some(Tok::ColonColon));
            match &t.tok {
                // `$'total due'`: a name of any characters.
                Tok::Dollar if line.get(i + 1).is_some_and(|n| matches!(n.tok, Tok::Name(_))) => {
                    let mut nt = line[i + 1].clone();
                    nt.span = t.span.to(nt.span);
                    out.push(nt);
                    i += 2;
                    continue;
                }
                Tok::Name(nm) if self.src.as_bytes().get(t.span.start as usize) == Some(&b'\'') => {
                    let d = Diagnostic::error(old_said("'x'", "$x")).named("old_spelling").at(t.span);
                    let fixed = if nm.chars().all(|c| c.is_alphanumeric() || c == '_') { format!("${}", nm) } else { format!("$'{}'", nm) };
                    return err(d.fix_edit(t.span, Edit::Replace, fixed));
                }
                Tok::LBrace | Tok::RBrace => {
                    return err(Diagnostic::error(old_said("{ }", "name:")).named("old_spelling").at(t.span));
                }
                // `[<T>: Addable, <U>: any]`: a generic's placeholders in their own `[ ]` (owner,
                // 2026-10-29), the parser's `<T: Addable, U: any>`.
                Tok::LBracket if line.get(i + 1).is_some_and(|n| is_op(n, "<")) && line.get(i + 2).is_some_and(|n| matches!(n.tok, Tok::Word(_))) && line.get(i + 3).is_some_and(|n| is_op(n, ">")) => {
                    let mut depth = 0i32;
                    let mut j = i;
                    while j < n {
                        match &line[j].tok {
                            Tok::LBracket | Tok::LParen => depth += 1,
                            Tok::RBracket | Tok::RParen => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                        j += 1;
                    }
                    out.push(tok(Tok::Op("<".into()), t.span));
                    let mut k = i + 1;
                    while k < j {
                        let u = &line[k];
                        // `<T>` drops its brackets inside the list: `T`.
                        if is_op(u, "<") && line.get(k + 2).is_some_and(|c| is_op(c, ">")) && matches!(line.get(k + 1).map(|c| &c.tok), Some(Tok::Word(_))) && matches!(line.get(k - 1).map(|c| &c.tok), Some(Tok::LBracket) | Some(Tok::Comma)) {
                            out.push(line[k + 1].clone());
                            k += 3;
                            continue;
                        }
                        let words = self.words(vec![u.clone()], Kind::Body)?;
                        out.extend(words);
                        k += 1;
                    }
                    if j < n {
                        out.push(tok(Tok::Op(">".into()), line[j].span));
                    }
                    i = j + 1;
                    continue;
                }
                Tok::Op(o) => {
                    let written = self.text(t.span).to_string();
                    let ends_value = matches!(prev, Some(Tok::Name(_) | Tok::Num { .. } | Tok::Str(_) | Tok::Char(_) | Tok::RParen | Tok::RBracket | Tok::Word(_)));
                    // C's shifts and its XOR, between two values.
                    if ends_value && (written == "<<" || written == ">>") {
                        let (dir, w) = if written == "<<" { ("left", "shl") } else { ("right", "shr") };
                        return err(Diagnostic::error(format!("`{}` is C's: a shift {} is `{}`", written, dir, w)).named("old_spelling").at(t.span).fix_edit(t.span, Edit::Replace, w));
                    }
                    if ends_value && written == "^" {
                        return err(Diagnostic::error("`^` is not an operator: power is `std::pow`, XOR is `xor` (owner, 2026-10-28)").named("old_spelling").at(t.span));
                    }
                    // `?` ends a call that can fail; C's `? :` is not CyborgPL.
                    if o == "?" && line.get(i + 1).is_some_and(|n| !matches!(n.tok, Tok::Semi | Tok::RParen | Tok::RBracket | Tok::Comma | Tok::Eof)) {
                        return err(Diagnostic::error("`? :` is not CyborgPL: `?` means \"otherwise fail\", after a call that can fail; a choice between values is written with `if` (owner, 2026-10-27)").named("old_spelling").at(t.span));
                    }
                    let mapped = match o.as_str() {
                        "!=" => Some("!=="),
                        "<=" => Some("</=="),
                        ">=" => Some(">/=="),
                        _ => None,
                    };
                    if let Some(m) = mapped {
                        out.push(tok(Tok::Op(m.into()), t.span));
                        i += 1;
                        continue;
                    }
                    // `fn total <T: Addable>`: the placeholders are declared in their own `[ ]` now.
                    if o == "<" && out.len() >= 2 && matches!(&out[out.len() - 1].tok, Tok::Word(_)) && matches!(&out[out.len() - 2].tok, Tok::Word(k) if matches!(k.as_str(), "fn" | "struct" | "enum" | "interface")) && line.get(i + 1).is_some_and(|x| matches!(x.tok, Tok::Word(_))) {
                        return err(Diagnostic::error("a generic's placeholders are declared in their own `[ ]`, before the parameters: `fn total [<T>: Addable] [lent $xs: std::ls[<T>]]` (owner, 2026-10-29)").named("old_spelling").at(t.span));
                    }
                    if o == "?" {
                        out.push(word("otherwise", t.span));
                        out.push(word("fail", t.span));
                        i += 1;
                        continue;
                    }
                    if o == "??" {
                        out.push(word("otherwise", t.span));
                        i += 1;
                        continue;
                    }
                    if o == "!==" || o == "</==" || o == ">/==" {
                        let new = match o.as_str() { "!==" => "!=", "</==" => "<=", _ => ">=" };
                        return err(Diagnostic::error(format!("`{}` is an old spelling: write `{}`", self.text(t.span), new)).named("old_spelling").at(t.span).fix_edit(t.span, Edit::Replace, new));
                    }
                    out.push(t.clone());
                    i += 1;
                    continue;
                }
                // std's names, read as the parser's (redesign two and three, `design/revision-2026-10-27.md`).
                Tok::Word(w) if after_dot || w == "cluster" || w == "clusters" || w == "bf16" => {
                    let before = if out.len() >= 2 { out[out.len() - 2].tok.clone() } else { Tok::Eof };
                    let w = w.clone();
                    // The base's operators take no `std::` (owner, 2026-10-28: "`plus` isn't even `std`").
                    if matches!(prev, Some(Tok::ColonColon)) && matches!(before, Tok::Word(ref k) if k == "std") && matches!(w.as_str(), "plus" | "minus" | "times" | "div" | "mod" | "concat" | "and" | "or" | "not" | "xor") {
                        return err(Diagnostic::error(format!("`std::{}` is the base's, not std's: write it without `std::`", w)).named("std_name_needs_path").at(out[out.len() - 2].span.to(t.span)));
                    }
                    if w == "auto_newline" || w == "auto_newln" {
                        let from = out.iter().rposition(|x| matches!(&x.tok, Tok::Word(k) if k == "print")).map_or(t.span, |k| out[k].span);
                        let written = self.text(from.to(t.span)).to_string();
                        return err(Diagnostic::error(format!("`{}` is an old spelling: write `std::print(…, newln)`; a line ends with `newln`, in every printing form (owner, 2026-10-28)", written)).named("old_spelling").at(t.span));
                    }
                    let mapped: Option<&str> = match (w.as_str(), &before) {
                        ("cluster", _) => Some("grapheme"),
                        ("bf16", _) => Some("brain16"),
                        ("divide", _) => Some("div"),
                        ("truncate", _) => Some("trunc"),
                        ("clusters", _) => Some("graphemes"),
                        ("len", _) => Some("length"),
                        ("ls", _) => Some("list"),
                        ("arr", _) => Some("array"),
                        ("count", Tok::Word(m)) if m == "map" => Some("length"),
                        ("shortest", _) => Some("useful"),
                        ("arrows", Tok::Word(m)) if m == "print" => Some("weird"),
                        ("auto_newline", _) => Some("auto_newline"),
                        _ => None,
                    };
                    let mut c = t.clone();
                    if let Some(m) = mapped {
                        c.tok = Tok::Word(m.into());
                    }
                    out.push(c);
                    i += 1;
                    // `std::print(…)` is `print.shortest`.
                    if w == "print" && matches!(before, Tok::Word(ref k) if k == "std") && line.get(i).is_some_and(|n| n.tok == Tok::LParen || n.tok == Tok::LBracket) {
                        out.push(tok(Tok::Dot, t.span));
                        out.push(word("useful", t.span));
                    }
                    continue;
                }
                Tok::Word(w) if !after_dot => {
                    let w = w.clone();
                    let written = self.text(t.span).to_string();
                    // The retired words, wherever they stand (`spellings.cyr`); in a bound's brackets
                    // a word is the bound's (`%C.str[max: 6 bytes, include zero]`).
                    let in_brackets = line[..i].iter().filter(|x| x.tok == Tok::LBracket).count() > line[..i].iter().filter(|x| x.tok == Tok::RBracket).count();
                    if let (Some(said), false) = (crate::spellings::retired(&written), in_brackets) {
                        return err(Diagnostic::error(said).named("old_spelling").at(t.span));
                    }
                    // The case marks are gone (owner, 2026-10-27): every name is exact.
                    if written == "case" && line.get(i + 1).is_some_and(|d| d.tok == Tok::Dot) {
                        if let Some(m) = line.get(i + 2).filter(|m| matches!(&m.tok, Tok::Word(x) if matches!(x.as_str(), "any" | "exact" | "insensitive" | "sensitive"))) {
                            return err(Diagnostic::error(format!("`case.{}` is gone: every name is exact (owner, 2026-10-27)", self.text(m.span))).named("old_spelling").at(t.span.to(m.span)));
                        }
                    }
                    // `int.sign.32` and the long number types are `i32` and the short ones (redesign three).
                    if (written == "int" || written == "float") && line.get(i + 1).is_some_and(|d| d.tok == Tok::Dot) {
                        let mut j = i;
                        let mut parts: Vec<String> = Vec::new();
                        while j < line.len() && matches!(line[j].tok, Tok::Word(_) | Tok::Num { .. }) {
                            parts.push(self.text(line[j].span).to_string());
                            if line.get(j + 1).is_some_and(|d| d.tok == Tok::Dot) {
                                j += 2;
                            } else {
                                break;
                            }
                        }
                        let last = j.min(line.len() - 1);
                        let long = parts.join(".");
                        let short = short_number(&parts);
                        if let Some(short) = short {
                            return err(Diagnostic::error(format!("`{}` is an old spelling: write `{}` (owner, 2026-10-28)", long, short)).named("old_spelling").at(t.span.to(line[last].span)).fix_edit(t.span.to(line[last].span), Edit::Replace, short));
                        }
                    }
                    // `std::println` was tried and dropped (owner, 2026-10-28).
                    if written == "println" && matches!(prev, Some(Tok::ColonColon)) {
                        let narrowed = line.get(i + 1).is_some_and(|d| d.tok == Tok::Dot) && line.get(i + 2).is_some_and(|w| matches!(w.tok, Tok::Word(_)));
                        let (said, write) = if narrowed {
                            let n = self.text(line[i + 2].span).to_string();
                            (format!("std::println.{}", n), format!("std::print.{}(…, newln)", n))
                        } else {
                            ("std::println".to_string(), "std::print(…, newln)".to_string())
                        };
                        return err(Diagnostic::error(format!("`{}` is an old spelling: write `{}`; a line ends with `newln`, in every printing form (owner, 2026-10-28)", said, write)).named("old_spelling").at(t.span));
                    }
                    // The comparisons were words until redesign three.
                    if let Some(new) = match written.as_str() {
                        "equal_to" => Some("=="),
                        "not_equal" => Some("!="),
                        "at_most" => Some("<="),
                        "at_least" => Some(">="),
                        _ => None,
                    } {
                        return err(Diagnostic::error(old_said(&written, new)).named("old_spelling").at(t.span).fix_edit(t.span, Edit::Replace, new));
                    }
                    // Redesign three: one spelling. The words of the operators with a symbol are old
                    // spellings, their narrowings (`plus.wrap`, `times.ieee`) words still.
                    let narrowed = line.get(i + 1).is_some_and(|d| d.tok == Tok::Dot);
                    let new_spelling = match written.as_str() {
                        "plus" | "minus" | "times" if !narrowed => Some(match written.as_str() { "plus" => "+", "minus" => "-", _ => "x" }),
                        "eq" => Some("=="),
                        "not_eq" | "not_equal_to" => Some("!="),
                        "lt" | "less_than" => Some("<"),
                        "gt" | "greater_than" => Some(">"),
                        "pow" | "xx" | "power" => Some("std::pow"),
                        "negate" => Some("neg"),
                        "concatenate" => Some("concat"),
                        _ => None,
                    };
                    // A case's own name, `lt;`, `case lt:`, is a name.
                    let a_name = in_kind == Kind::Cases || (i == 1 && is_word(&line[0], "case")) || matches!(prev, Some(Tok::Word(ref k)) if k == "fn" || k == "proc" || k == "rec");
                    if let (Some(new), false) = (new_spelling, a_name) {
                        if written == "negate" && narrowed {
                            return err(Diagnostic::error("`negate` is an old spelling: write `neg`: `neg.ieee $x` (owner, 2026-10-28)").named("old_spelling").at(t.span).fix_edit(t.span, Edit::Replace, "neg"));
                        }
                        let d = Diagnostic::error(old_said(&written, new)).named("old_spelling").at(t.span);
                        return err(if new.starts_with("std") || new == "neg" { d } else { d.fix_edit(t.span, Edit::Replace, new) });
                    }
                    // A bare `x` is multiplication (owner, 2026-10-28): something named `x` is reached by its path.
                    let ends_value = matches!(prev, Some(Tok::Name(_) | Tok::Num { .. } | Tok::Str(_) | Tok::Char(_) | Tok::RParen | Tok::RBracket));
                    if written == "x" && !ends_value && line.get(i + 1).is_some_and(|n| n.tok == Tok::LParen && n.span.start == t.span.end) {
                        return err(Diagnostic::error("a bare `x` is multiplication: a function or type named `x` is reached by its path, `my.x(…)` (owner, 2026-10-28)").named("name_reserved").at(t.span).fix_edit(t.span, Edit::Replace, "my.x"));
                    }
                    match w.as_str() {
                        // `divide` and `modulo` are `div` and `mod` (redesign one's words).
                        "divide" | "modulo" if !narrowed => {
                            let mut c = t.clone();
                            c.tok = Tok::Word(if w == "divide" { "div".into() } else { "mod".into() });
                            out.push(c);
                            i += 1;
                            continue;
                        }
                        "let" | "var" => {
                            let (toks, used) = self.declaration(&line, i, in_kind)?;
                            out.extend(toks);
                            i += used;
                            continue;
                        }
                        "proc" => {
                            let mut f = word("fn", t.span);
                            f.proc = true;
                            f.line_first = t.line_first;
                            out.push(f);
                            i += 1;
                            continue;
                        }
                        "no_skip" => {
                            let mut r = word("no_cheat", t.span);
                            r.line_first = t.line_first;
                            out.push(r);
                            i += 1;
                            continue;
                        }
                        // Negation as a value is `op neg` (owner, 2026-10-28); anywhere else, `-5`.
                        "neg" if matches!(prev, Some(Tok::Word(ref o)) if o == "op") || line.get(i + 1).is_some_and(|d| d.tok == Tok::Dot) => {
                            out.push(word("negate", t.span));
                            i += 1;
                            continue;
                        }
                        "neg" => {
                            return err(Diagnostic::error(old_said("neg", "-5")).named("old_spelling").at(t.span));
                        }
                        // A precedence level lists its operators in brackets (owner, 2026-10-27).
                        "level" if self.stack.last().is_some_and(|o| o.closer.starts_with("precedence")) => {
                            out.push(t.clone());
                            i += 1;
                            while i < n && line[i].tok != Tok::LBracket && line[i].tok != Tok::Semi {
                                out.push(line[i].clone());
                                i += 1;
                            }
                            if i < n && line[i].tok == Tok::LBracket {
                                let mut depth = 0;
                                while i < n {
                                    let mut c = line[i].clone();
                                    match c.tok {
                                        Tok::LBracket => {
                                            depth += 1;
                                            if depth == 1 {
                                                c.tok = Tok::LBrace;
                                            }
                                        }
                                        Tok::RBracket => {
                                            depth -= 1;
                                            if depth == 0 {
                                                c.tok = Tok::RBrace;
                                                out.push(c);
                                                i += 1;
                                                break;
                                            }
                                        }
                                        // The language's symbols, the parser's words in a level.
                                        Tok::Op(ref o) if depth == 1 && (o == "+" || o == "-") => c.tok = Tok::Word(if o == "+" { "plus".into() } else { "minus".into() }),
                                        Tok::Op(ref o) if depth == 1 => {
                                            let m = match o.as_str() { "!=" => "!==", "<=" => "</==", ">=" => ">/==", _ => o.as_str() };
                                            c.tok = Tok::Op(m.to_string());
                                        }
                                        // Multiplication's symbol, the parser's `times` in a level.
                                        Tok::Word(ref w) if depth == 1 && w == "x" => c.tok = Tok::Word("times".into()),
                                        _ => {}
                                    }
                                    out.push(c);
                                    i += 1;
                                }
                            }
                            // `left_first;`: the parser's levels end at their `}` and chain word.
                            while i < n {
                                if line[i].tok != Tok::Semi {
                                    out.push(line[i].clone());
                                }
                                i += 1;
                            }
                            continue;
                        }
                        "ret" => {
                            let mut r = word("return", t.span);
                            r.line_first = t.line_first;
                            out.push(r);
                            i += 1;
                            continue;
                        }
                        "rec" => {
                            let mut r = word("struct", t.span);
                            r.line_first = t.line_first;
                            out.push(r);
                            i += 1;
                            continue;
                        }
                        "sig" | "set" => {
                            let mut r = word("interface", t.span);
                            r.line_first = t.line_first;
                            out.push(r);
                            i += 1;
                            continue;
                        }
                        "ls" => {
                            out.push(word("list", t.span));
                            i += 1;
                            continue;
                        }
                        "arr" => {
                            out.push(word("array", t.span));
                            i += 1;
                            continue;
                        }
                        // `coll.proc` is the parser's `coll.fn`, its kind kept; `coll.bounded` its `coll.bound`.
                        "coll" if line.get(i + 1).is_some_and(|d| d.tok == Tok::Dot) && line.get(i + 2).is_some_and(|w| is_word(w, "proc") || is_word(w, "bounded")) => {
                            out.push(t.clone());
                            out.push(line[i + 1].clone());
                            let mut k = line[i + 2].clone();
                            if is_word(&k, "proc") {
                                k.tok = Tok::Word("fn".into());
                                k.proc = true;
                            } else {
                                k.tok = Tok::Word("bound".into());
                            }
                            out.push(k);
                            i += 3;
                            continue;
                        }
                        // `%spawn.task` is the parser's `%concur`.
                        "spawn" if t.marked && line.get(i + 1).is_some_and(|d| d.tok == Tok::Dot) && line.get(i + 2).is_some_and(|w| is_word(w, "task")) => {
                            let mut c = t.clone();
                            c.tok = Tok::Word("concur".into());
                            c.span = t.span.to(line[i + 2].span);
                            out.push(c);
                            i += 3;
                            continue;
                        }
                        // `%extern.C $x: T;`: a C variable says whether it can change (owner, 2026-10-27).
                        "C" if line.get(i + 1).is_some_and(|n| matches!(n.tok, Tok::Name(_))) && i >= 2 && is_word(&line[i - 2], "extern") => {
                            let n = &line[i + 1];
                            return err(Diagnostic::error(format!("a C variable says whether it can change: `%extern.C let {}: …;` (C's `const`, read only) or `%extern.C var {}: …;` (owner, 2026-10-27)", self.text(n.span), self.text(n.span))).named("declaration_form").at(n.span).fix_edit(n.span, Edit::Insert, "let "));
                        }
                        // `union.tag Status:`.
                        "union" if line.get(i + 1).is_some_and(|d| d.tok == Tok::Dot) && line.get(i + 2).is_some_and(|w| is_word(w, "tag")) => {
                            let mut e = word("enum", t.span.to(line[i + 2].span));
                            e.line_first = t.line_first;
                            out.push(e);
                            i += 3;
                            continue;
                        }
                        // `const $x: T := …` is the parser's `const $x: T = …`.
                        "const" if line.get(i + 1).is_some_and(|n| matches!(n.tok, Tok::Name(_))) && line.get(i + 2).is_some_and(|n| n.tok == Tok::Colon) => {
                            let mut depth = 0i32;
                            let mut j = i + 2;
                            while j < line.len() {
                                match &line[j].tok {
                                    Tok::LBracket | Tok::LParen => depth += 1,
                                    Tok::RBracket | Tok::RParen => depth -= 1,
                                    Tok::Walrus if depth == 0 => break,
                                    Tok::Op(o) if o == "=" && depth == 0 => {
                                        return err(Diagnostic::error("a `const` is defined with `:=`: `const $x: i32 := 5;`").named("declaration_form").at(line[j].span).fix_edit(line[j].span, Edit::Replace, ":="));
                                    }
                                    _ => {}
                                }
                                j += 1;
                            }
                            out.push(t.clone());
                            let mid = self.words(line[i + 1..j.min(line.len())].to_vec(), Kind::Body)?;
                            out.extend(mid);
                            if let Some(eq) = line.get(j) {
                                out.push(tok(Tok::Op("=".into()), eq.span));
                            }
                            i = j + 1;
                            continue;
                        }
                        "change" => {
                            let (toks, used) = self.change(&line, i)?;
                            out.extend(toks);
                            i += used;
                            continue;
                        }
                        "case" | "member" if t.line_first || i == 0 => {
                            // `case odd:` is the arm `odd { … }`; `member area.square […]` the member.
                            i += 1;
                            if let Some(nx) = line.get(i) {
                                let mut nx = nx.clone();
                                nx.line_first = true;
                                let mut rest = line[i..].to_vec();
                                rest[0] = nx;
                                let w = self.words(rest, in_kind)?;
                                out.extend(w);
                            }
                            return Ok(out);
                        }
                        _ => {}
                    }
                    if field_line && i == 0 && (w == "immut" || w == "mut") {
                        let (said, new) = if w == "mut" { (old_said("mut", "var"), "var") } else { (old_said("immut", "let"), "let") };
                        return err(Diagnostic::error(said).named("old_spelling").at(t.span).fix_edit(t.span, Edit::Replace, new));
                    }
                    if field_line && i == 0 && matches!(line.get(i + 1).map(|t| &t.tok), Some(Tok::Colon)) {
                        return refuse("field_form", format!("every field says whether it changes: `let {}: …;` or `var {}: …;`", w, w), t.span);
                    }
                    let mut c = t.clone();
                    c.tok = Tok::Word(w);
                    out.push(c);
                    i += 1;
                }
                _ => {
                    out.push(t.clone());
                    i += 1;
                }
            }
        }
        // A rec's field ends in `;` here and in `,` in the parser's braces; a case and a set's type
        // alike.
        if matches!(in_kind, Kind::Rec | Kind::Cases) {
            if let Some(last) = out.last_mut() {
                if last.tok == Tok::Semi {
                    last.tok = Tok::Comma;
                }
            }
        }
        if in_kind == Kind::Set {
            if let Some(last) = out.last() {
                if last.tok == Tok::Semi {
                    out.pop();
                }
            }
        }
        Ok(out)
    }

    /// `let $x := …`, `let $x: T := …`, `var $x = …`, `var $x: T = …`, a parameter `let $a: T`
    /// or `var $a: T`, a field `let a: T` or `var a: T`: the parser's `let immut`, `let mut`,
    /// `immut $a`, `mut $a`, `immut a` and `a`. How many tokens it took.
    fn declaration(&mut self, line: &[Token], i: usize, in_kind: Kind) -> Result<(Vec<Token>, usize)> {
        let t = &line[i];
        let var = is_word(t, "var");
        let prev = if i > 0 { Some(&line[i - 1].tok) } else { None };
        let mut out = Vec::new();
        // A parameter, after `[` or `,`.
        if matches!(prev, Some(Tok::LBracket) | Some(Tok::Comma)) {
            if var {
                out.push(word("mut", t.span));
            } else {
                out.push(word("immut", t.span));
            }
            return Ok((out, 1));
        }
        // A field, its name never a keyword.
        if in_kind == Kind::Rec && i == 0 {
            if let Some(Tok::Word(n)) = line.get(i + 1).map(|x| &x.tok) {
                if crate::spellings::is_keyword(n) || n == "ret" {
                    return refuse("name_reserved", format!("`{}` is a keyword and cannot name a field", n), line[i + 1].span);
                }
            }
            if var {
                return Ok((out, 1));
            }
            out.push(word("immut", t.span));
            return Ok((out, 1));
        }
        // `%extern.C let $x: T;`: a C variable, read only, or `var`, written too.
        let in_extern = i == 0 && self.stack.last().is_some_and(|o| o.closer == "%extern.C");
        if in_extern || prev.is_some_and(|p| matches!(p, Tok::Word(w) if w == "C")) {
            if var {
                out.push(word("mut", t.span));
            }
            return Ok((out, 1));
        }
        let next = line.get(i + 1);
        if next.is_some_and(|n| n.tok == Tok::Dot) {
            return refuse("declaration_form", format!("`{}` is a word of its own: `{} $x {} …;`", if var { "var" } else { "let" }, if var { "var" } else { "let" }, if var { "=" } else { ":=" }), next.unwrap().span);
        }
        // Not a name next: the parser says what it is (`let !$ $x`, an old spelling).
        let named = |k: usize| line.get(k).is_some_and(|n| matches!(n.tok, Tok::Name(_)));
        let marked = |k: usize| line.get(k).is_some_and(|n| is_word(n, "shadow") || is_word(n, "no_shadow"));
        if !named(i + 1) && !(marked(i + 1) && named(i + 2)) && !next.is_some_and(|n| is_word(n, "mut") || is_word(n, "immut")) {
            out.push(word("let", t.span));
            if var {
                out.push(word("mut", t.span));
            }
            return Ok((out, 1));
        }
        if next.is_some_and(|n| is_word(n, "mut") || is_word(n, "immut")) {
            let n = next.unwrap();
            let said = if is_word(n, "mut") { old_said("let mut", "var") } else { old_said("immut", "let") };
            return err(Diagnostic::error(said).named("old_spelling").at(t.span.to(n.span)));
        }
        let mut k = i + 1;
        let shadow = line.get(k).is_some_and(|n| is_word(n, "shadow"));
        let sealed = line.get(k).is_some_and(|n| is_word(n, "no_shadow"));
        if shadow {
            out.push(word("shadow", t.span.to(line[k].span)));
            k += 1;
        } else {
            out.push(word("let", t.span));
        }
        out.push(word(if var { "mut" } else { "immut" }, t.span));
        // `let no_shadow $x`: the name may not be shadowed (owner, 2026-10-07).
        if sealed {
            out.push(line[k].clone());
            k += 1;
        }
        // The name, its type, and then `:=` or `=`.
        let mut depth = 0i32;
        let mut typed = false;
        let mut j = k;
        while j < line.len() {
            match &line[j].tok {
                Tok::LBracket | Tok::LParen => depth += 1,
                Tok::RBracket | Tok::RParen => depth -= 1,
                Tok::Colon if depth == 0 && j == k + 1 => typed = true,
                Tok::Walrus | Tok::Semi if depth == 0 => break,
                Tok::Op(o) if o == "=" && depth == 0 => break,
                _ => {}
            }
            j += 1;
        }
        let Some(eq) = line.get(j).filter(|e| e.tok != Tok::Semi) else {
            return refuse("declaration_form", format!("there is no implicit zero: a declaration gives its value, `{} $x: i32 {} 0;`", if var { "var" } else { "let" }, if var { "=" } else { ":=" }), t.span);
        };
        let walrus = eq.tok == Tok::Walrus;
        let equals = is_op(eq, "=");
        if var && walrus {
            return err(Diagnostic::error("a `var` starts with `=`, its starting value: `var $n = 0;` (`:=` defines a `let` or a `const`)").named("declaration_form").at(eq.span).fix_edit(eq.span, Edit::Replace, "="));
        }
        if !var && equals {
            return err(Diagnostic::error("a `let` is defined with `:=`: `let $x := 5;` (`=` starts a `var`)").named("declaration_form").at(eq.span).fix_edit(eq.span, Edit::Replace, ":="));
        }
        if !walrus && !equals {
            return refuse("declaration_form", format!("a declaration gives its value: `{} $x {} …;`", if var { "var" } else { "let" }, if var { "=" } else { ":=" }), t.span);
        }
        // The parser's: `:=` untyped, `=` typed.
        let mid = self.words(line[k..j].to_vec(), Kind::Body)?;
        out.extend(mid);
        if typed {
            out.push(tok(Tok::Op("=".into()), eq.span));
        } else {
            out.push(tok(Tok::Walrus, eq.span));
        }
        Ok((out, j + 1 - i))
    }

    /// `change $x to …;` is the parser's `change $x = …;`.
    fn change(&mut self, line: &[Token], i: usize) -> Result<(Vec<Token>, usize)> {
        let mut out = vec![line[i].clone()];
        let mut depth = 0i32;
        let mut j = i + 1;
        while j < line.len() {
            match &line[j].tok {
                Tok::LBracket | Tok::LParen => depth += 1,
                Tok::RBracket | Tok::RParen => depth -= 1,
                Tok::Word(w) if w == "to" && depth == 0 => break,
                Tok::Op(o) if o == "=" && depth == 0 => {
                    return err(Diagnostic::error("a change is `change $x to 6;` (owner, 2026-10-27)").named("old_spelling").at(line[j].span).fix_edit(line[j].span, Edit::Replace, "to"));
                }
                // `+=` and its family were tried and dropped (owner, 2026-10-28).
                Tok::Op(o) if depth == 0 && crate::spellings::old_word(o, None).is_some() => {
                    return err(Diagnostic::error(crate::spellings::old_word(o, None).unwrap().dated()).named("old_spelling").at(line[j].span));
                }
                Tok::Semi if depth == 0 => {
                    return refuse("unexpected_text", "a change says what it changes to: `change $x to 6;`", line[j].span);
                }
                _ => {}
            }
            j += 1;
        }
        if j >= line.len() {
            return refuse("unexpected_text", "a change says what it changes to: `change $x to 6;`", line[i].span);
        }
        let target = self.words(line[i + 1..j].to_vec(), Kind::Body)?;
        out.extend(target);
        out.push(tok(Tok::Op("=".into()), line[j].span));
        Ok((out, j + 1 - i))
    }
}

/// A name the parser reads, as a program writes it now (redesign two and three): `print.useful`
/// is `print` (or `print.shortest` beside more words), `list.length` is `ls.len`, `map.length`
/// `map.count`, `grapheme` `cluster`, `brain16` `bf16`, `concur` `spawn.task`.
pub fn spoken(internal: &str) -> String {
    if internal == "print.useful" {
        return "print".into();
    }
    let parts: Vec<&str> = internal.split('.').collect();
    let mut out: Vec<String> = Vec::new();
    for (k, p) in parts.iter().enumerate() {
        let before = if k > 0 { parts[k - 1] } else { "" };
        let w = match (*p, before) {
            ("useful", _) => "shortest",
            ("weird", "print") => "arrows",
            ("list", _) => "ls",
            ("array", _) => "arr",
            ("length", "map") => "count",
            ("length", _) => "len",
            ("grapheme", _) => "cluster",
            ("graphemes", _) => "clusters",
            ("brain16", _) => "bf16",
            ("concur", _) => "spawn.task",
            ("no_cheat", _) => "no_skip",
            ("nocheck", _) => "no_check",
            (other, _) => other,
        };
        out.push(w.to_string());
    }
    out.join(".")
}

/// An operator as the parser names it, as a program writes it now: `plus` is `+`, `equal_to` `==`.
pub fn op_spoken(internal: &str) -> String {
    match internal {
        "plus" => "+",
        "minus" => "-",
        "times" => "x",
        "equal_to" => "==",
        "not_equal" => "!=",
        "at_most" => "<=",
        "at_least" => ">=",
        "less_than" => "<",
        "greater_than" => ">",
        "negate" => "neg",
        "xx" => "std::pow",
        other => other,
    }
    .to_string()
}

/// A long number type's parts, `int.sign.32`, as the short one, `i32` (redesign three).
fn short_number(parts: &[String]) -> Option<String> {
    let p: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();
    let mark = |s: &str| s.starts_with('%');
    Some(match p.as_slice() {
        ["int", "sign", n] if !mark(n) => format!("i{}", n),
        ["int", "unsign", n] if !mark(n) => format!("u{}", n),
        ["int", "sign", n] => format!("%i{}", &n[1..]),
        ["int", "unsign", n] => format!("%u{}", &n[1..]),
        ["int", "sign", "%not_std", n] | ["int", "sign", "not_std", n] => format!("%not_std.i{}", n),
        ["int", "unsign", "%not_std", n] | ["int", "unsign", "not_std", n] => format!("%not_std.u{}", n),
        ["float", "bin", n] if !mark(n) => format!("f{}", n),
        ["float", "bin", n] => format!("%f{}", &n[1..]),
        ["float", "%deci", n] | ["float", "deci", n] => format!("%d{}", n),
        ["float", "%brain", "16"] | ["float", "brain", "16"] => "%bf16".to_string(),
        _ => return None,
    })
}

/// The code in a message, `` `…` ``, in redesign three's spellings (owner, 2026-10-29: kayo's
/// messages use the one spelling): the parser's words for the ones a program writes now,
/// `return` as `ret`, `equal_to` as `==`, `list` as `ls`, `{ … }` as a block's `:`.
pub fn in_message(sentence: &str) -> String {
    if !sentence.contains('`') {
        return sentence.to_string();
    }
    sentence.split('`').enumerate().map(|(n, part)| if n % 2 == 1 { code(part) } else { part.to_string() }).collect::<Vec<_>>().join("`")
}

/// One piece of code, in redesign three's spellings.
pub fn code(text: &str) -> String {
    // Words and the rest, each kept with what stood before it.
    let b = text.as_bytes();
    let mut pieces: Vec<String> = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        let start = i;
        if c.is_ascii_alphabetic() || c == b'_' {
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
        } else {
            i += text[i..].chars().next().map_or(1, |c| c.len_utf8());
        }
        pieces.push(text[start..i].to_string());
    }
    let word = |k: usize| pieces.get(k).map(|s| s.as_str()).unwrap_or("");
    let mut out: Vec<String> = Vec::with_capacity(pieces.len());
    let mut k = 0;
    while k < pieces.len() {
        let w = pieces[k].as_str();
        let before = if k > 0 { word(k - 1) } else { "" };
        let before2 = if k > 1 { word(k - 2) } else { "" };
        let after = word(k + 1);
        let path_before = before == "." || before == ":";
        let narrowed = after == ".";
        let new: Option<String> = match w {
            "return" => Some("ret".into()),
            "struct" => Some("rec".into()),
            "interface" => Some("sig".into()),
            "enum" if !(before == " " && before2 == "C") && !path_before => Some("union.tag".into()),
            "equal_to" => Some("==".into()),
            "not_equal" => Some("!=".into()),
            "at_most" => Some("<=".into()),
            "at_least" => Some(">=".into()),
            "less_than" => Some("<".into()),
            "greater_than" => Some(">".into()),
            "plus" | "minus" | "times" if !narrowed && !path_before => Some(match w { "plus" => "+", "minus" => "-", _ => "x" }.into()),
            "negate" => Some("neg".into()),
            "immut" => Some("let".into()),
            "list" if before != "'" => Some("ls".into()),
            "array" => Some("arr".into()),
            "length" if path_before => Some(if before2 == "map" { "count" } else { "len" }.into()),
            "useful" if path_before && before2 == "print" && after != "." => {
                // `print.useful(…)` is `print`; `print.useful.stderr` keeps its words.
                out.pop();
                Some(String::new())
            }
            "useful" if path_before => Some("shortest".into()),
            "weird" if path_before => Some("arrows".into()),
            "grapheme" => Some("cluster".into()),
            "graphemes" => Some("clusters".into()),
            "brain16" => Some("bf16".into()),
            "concur" => Some("spawn.task".into()),
            "no_cheat" => Some("no_skip".into()),
            "nocheck" => Some("no_check".into()),
            "auto_newln" => Some("auto_newline".into()),
            _ => None,
        };
        // `let mut` is `var`; `mut $x` a `var` parameter; `lend.mut` stays.
        if w == "let" && after == " " && word(k + 2) == "mut" {
            out.push("var".into());
            k += 3;
            continue;
        }
        if w == "mut" && !path_before {
            out.push("var".into());
            k += 1;
            continue;
        }
        match new {
            Some(n) => out.push(n),
            None => out.push(w.to_string()),
        }
        k += 1;
    }
    let s = out.concat();
    // A block in braces, `{ … }`, is `:` and its closer now.
    s.replace(" { … }", ": …").replace(" {…}", ": …").replace("{ … }", ": …").replace(" { }", ":")
}
