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
}

struct Open {
    /// What the closer says after `;`: `fn add`, `if`, `%layout.C rec Rect`.
    closer: String,
    /// The opening line's first token.
    at: Span,
    kind: Kind,
    /// An inline block's `with [ … ]`, which the parser reads after the code.
    with: Vec<Token>,
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
                return refuse("line_join_misplaced", "a line ends in `-` only to go on to a block's opening line below it, as `parse(lend $row)-` goes on to `otherwise:`", *at);
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
            if said != open.closer {
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
        self.signatures(&mut line, in_kind)?;
        // `else` goes on with the chain: the branch before it closes.
        let mut reopen = false;
        if is_word(&line[0], "else") {
            if in_kind != Kind::Chain {
                return refuse("unexpected_text", "`else` without an `if` before it", line[0].span);
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
            self.stack.push(Open { closer, at: first, kind, with });
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
            self.stack.push(Open { closer, at: first, kind, with: Vec::new() });
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
            _ => return Ok((self.inner_closer(rest).unwrap_or_else(|| "block".into()), Kind::Body)),
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
                None => return refuse("block_form", format!("a line ending in `:` opens a block, and `{}` opens none here", self.text(head.span)), head.span),
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
                    while j < n && line[j].tok != Tok::LBracket && !matches!(line[j].tok, Tok::Semi | Tok::Colon | Tok::Walrus) {
                        // `<T: Number>` holds a `:`.
                        if is_op(&line[j], "<") {
                            while j < n && !is_op(&line[j], ">") {
                                j += 1;
                            }
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
                Tok::Op(o) => {
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
                    let mapped: Option<&str> = match (w.as_str(), &before) {
                        ("cluster", _) => Some("grapheme"),
                        ("bf16", _) => Some("brain16"),
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
                    match w.as_str() {
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
        // A field.
        if in_kind == Kind::Rec && i == 0 {
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
        if next.is_some_and(|n| is_word(n, "mut") || is_word(n, "immut")) {
            let n = next.unwrap();
            let said = if is_word(n, "mut") { old_said("let mut", "var") } else { old_said("immut", "let") };
            return err(Diagnostic::error(said).named("old_spelling").at(t.span.to(n.span)));
        }
        let mut k = i + 1;
        let shadow = line.get(k).is_some_and(|n| is_word(n, "shadow"));
        if shadow {
            out.push(word("shadow", t.span.to(line[k].span)));
            k += 1;
        } else {
            out.push(word("let", t.span));
        }
        out.push(word(if var { "mut" } else { "immut" }, t.span));
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
        let Some(eq) = line.get(j) else {
            return refuse("declaration_form", format!("a declaration gives its value: `{} $x {} …;`", if var { "var" } else { "let" }, if var { "=" } else { ":=" }), t.span);
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
                    return err(Diagnostic::error(old_said("change $x =", "change $x to")).named("old_spelling").at(line[j].span).fix_edit(line[j].span, Edit::Replace, "to"));
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
