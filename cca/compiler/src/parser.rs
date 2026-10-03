//! The parser: tokens to `ast::File`. Every refusal here is a syntax rule
//! from `design/`; the sentence says what to write instead.

use crate::ast::*;
use crate::diag::{err, Diagnostic, Result};
use crate::lexer::{Tok, Token};
use crate::source::Span;

/// Bare words the language owns. A function may be named with one only
/// through a path (`limb.to[…]`); a bare `to[…]` is the keyword.
pub const KEYWORDS: &[&str] = &[
    "fn", "let", "change", "return", "if", "else", "loop", "while", "for", "in", "break", "continue", "give", "copy", "lend", "lent", "op",
    "check", "nocheck", "true", "false", "mut", "immut", "temp", "perm", "range", "seq", "para", "list", "array", "struct", "pub",
    "const", "module", "limb", "fail", "otherwise", "to", "with", "enum", "match", "interface", "satisfies", "extern", "ptr",
    "layout", "size", "align", "inline", "map", "channel", "concur", "whichever", "from", "str", "char", "bool", "var", "func",
    "int8", "int16", "int32", "int64", "int128", "uint8", "uint16", "uint32", "uint64", "uint128", "bin16", "bin32", "bin64",
    "bin128", "deci32", "deci64", "deci128",
];

/// The word operators; bare, but never reserved (functions.md).
pub const WORD_OPS: &[&str] = &["x", "xx", "mod", "div", "and", "or", "not", "xor", "shl", "shr"];

pub const DURATION_UNITS: &[&str] = &["nanos", "micros", "millis", "seconds", "minutes", "hours", "nano", "micro", "milli", "second", "minute", "hour"];

pub fn is_keyword(w: &str) -> bool {
    KEYWORDS.contains(&w)
}

/// Words that name a built-in type when they head a type path.
pub fn is_primitive_head(w: &str) -> bool {
    crate::resolve::rust_alias(w).is_some()
        || matches!(w, "str" | "char" | "bool" | "int" | "uint" | "bin" | "deci" | "brain16" | "bfloat16" | "bf16")
        || (w.starts_with("int") && w[3..].parse::<u32>().is_ok())
        || (w.starts_with("uint") && w[4..].parse::<u32>().is_ok())
        || (w.starts_with("bin") && w[3..].parse::<u32>().is_ok())
        || (w.starts_with("deci") && w[4..].parse::<u32>().is_ok())
}

struct Parser<'a> {
    toks: &'a [Token],
    pos: usize,
    /// A `#strict` file (owner, 2026-09-27): `pos` on every position,
    /// every parameter's type, a return type, `mut`/`immut` on every
    /// declaration, `temp`/`perm` on every `for`.
    strict: bool,
    /// Whether the body being parsed has a `return` with a value: in a
    /// strict file such a function says its type.
    returns_value: bool,
    /// What the last `-> …` said about a returned borrow.
    ret_lent: Option<RetLent>,
}

pub fn parse_file(toks: &[Token]) -> Result<File> {
    let mut p = Parser { toks, pos: 0, strict: false, returns_value: false, ret_lent: None };
    let mut strict = None;
    if p.is(&Tok::Strict) {
        strict = Some(p.span().file);
        p.strict = true;
        p.bump();
    }
    let mut items = Vec::new();
    while !p.at_eof() {
        if p.is(&Tok::Strict) {
            return p.fail("`#strict` goes once, at the top of the file, before any item");
        }
        push_item(&mut items, p.item()?);
    }
    Ok(File { items, strict })
}

const NOT_A_COMMENT: &str = "";

impl<'a> Parser<'a> {
    // ------------------------------------------------------------ helpers

    fn peek(&self) -> &Tok {
        &self.toks[self.pos].tok
    }
    fn peek_at(&self, n: usize) -> &Tok {
        let i = (self.pos + n).min(self.toks.len() - 1);
        &self.toks[i].tok
    }
    fn span(&self) -> Span {
        self.toks[self.pos].span
    }
    fn span_at(&self, n: usize) -> Span {
        let i = (self.pos + n).min(self.toks.len() - 1);
        self.toks[i].span
    }
    fn prev_span(&self) -> Span {
        self.toks[self.pos.saturating_sub(1)].span
    }
    fn at_eof(&self) -> bool {
        matches!(self.peek(), Tok::Eof)
    }
    fn bump(&mut self) -> Token {
        let t = self.toks[self.pos].clone();
        if !self.at_eof() {
            self.pos += 1;
        }
        t
    }
    fn is_word(&self, w: &str) -> bool {
        matches!(self.peek(), Tok::Word(x) if x == w)
    }
    fn is_word_at(&self, n: usize, w: &str) -> bool {
        matches!(self.peek_at(n), Tok::Word(x) if x == w)
    }
    fn is_op(&self, o: &str) -> bool {
        matches!(self.peek(), Tok::Op(x) if x == o)
    }
    fn is_op_at(&self, n: usize, o: &str) -> bool {
        matches!(self.peek_at(n), Tok::Op(x) if x == o)
    }
    fn is(&self, t: &Tok) -> bool {
        self.peek() == t
    }
    fn eat(&mut self, t: &Tok) -> bool {
        if self.is(t) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn eat_word(&mut self, w: &str) -> bool {
        if self.is_word(w) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn eat_op(&mut self, o: &str) -> bool {
        if self.is_op(o) {
            self.bump();
            true
        } else {
            false
        }
    }
    /// Two tokens are adjacent (no space between).
    fn adjacent(&self, a: usize, b: usize) -> bool {
        self.span_at(a).end == self.span_at(b).start
    }

    fn fail<T>(&self, msg: impl Into<String>) -> Result<T> {
        err(Diagnostic::error(msg).at(self.span()))
    }
    fn fail_at<T>(&self, span: Span, msg: impl Into<String>) -> Result<T> {
        err(Diagnostic::error(msg).at(span))
    }

    fn expect(&mut self, t: &Tok, what: &str) -> Result<Span> {
        if self.is(t) {
            Ok(self.bump().span)
        } else {
            self.fail(format!("expected {}, found {}", what, self.peek().describe()))
        }
    }
    fn expect_semi(&mut self, after: &str) -> Result<()> {
        if self.eat(&Tok::Semi) {
            Ok(())
        } else {
            self.fail(format!("expected `;` after {}, found {}", after, self.peek().describe()))
        }
    }
    fn expect_word(&mut self, w: &str) -> Result<Span> {
        if self.is_word(w) {
            Ok(self.bump().span)
        } else {
            self.fail(format!("expected `{}`, found {}", w, self.peek().describe()))
        }
    }

    fn ident(&mut self, what: &str) -> Result<Ident> {
        match self.peek().clone() {
            Tok::Word(name) => {
                let span = self.bump().span;
                Ok(Ident { name, span })
            }
            Tok::Name(n) => self.fail(format!("expected {}, found the quoted name '{}': {} is bare — quotes mark a value you hold", what, n, what)),
            other => self.fail(format!("expected {}, found {}", what, other.describe())),
        }
    }

    fn qname(&mut self, what: &str) -> Result<QName> {
        match self.peek().clone() {
            Tok::Name(name) => {
                let span = self.bump().span;
                Ok(QName { name, span })
            }
            Tok::Word(w) => self.fail(format!("expected {}, found the bare word `{}`: {} is a value you hold, so it is written in quotes — `'{}'`", what, w, what, w)),
            other => self.fail(format!("expected {}, found {}", what, other.describe())),
        }
    }

    /// `a.b.c` or `std::a.b`. At least one segment.
    fn path(&mut self) -> Result<Path> {
        let start = self.span();
        let mut limb = None;
        let first = self.ident("a name")?;
        let mut segs = Vec::new();
        if self.is(&Tok::ColonColon) {
            self.bump();
            limb = Some(first);
            segs.push(self.ident("a name after `::`")?);
        } else {
            segs.push(first);
        }
        // `.word` continues the path; `.'name'` is a const reached by path and stops it.
        while self.is(&Tok::Dot) && matches!(self.peek_at(1), Tok::Word(_)) {
            self.bump();
            segs.push(self.ident("a name after `.`")?);
        }
        let span = start.to(self.prev_span());
        Ok(Path { limb, segs, span })
    }

    // ------------------------------------------------------------ items

    fn item(&mut self) -> Result<Item> {
        let start = self.span();
        let mut vis = Vis::Private;
        if self.is_word("pub") {
            self.bump();
            if self.eat(&Tok::Dot) {
                let narrowing = self.ident("a word after `pub.`")?;
                if narrowing.name != "limb" {
                    return self.fail_at(narrowing.span, format!("`pub.{}` is not a visibility: the only narrowing of `pub` is `pub.limb` (this Limb only); plain `pub` is other Limbs too", narrowing.name));
                }
                vis = Vis::Limb;
            } else {
                vis = Vis::Pub;
            }
            if self.is_word("bare") {
                return self.fail("there are no bare names from another Limb: what a Limb makes `pub` is reached by its path, and only operators come bare (owner, 2026-09-23)");
            }
            if self.is_word("MAIN") {
                return self.fail("`MAIN` is never `pub`: it is the program's block, not something another Limb can reach");
            }
            if self.is_word("let") {
                return self.fail("a Limb-level value is a `const`, not a `let`: `pub const 'name' := …;`");
            }
            if let Tok::Name(n) = self.peek().clone() {
                return self.fail(format!("a Limb-level value is a `const`, not a variable: `pub const '{}' := …;`", n));
            }
        }
        // `test fn` (owner, 2026-09-29): never `pub`, run by `cca test`.
        if self.is_word("test") && self.is_word_at(1, "fn") {
            if vis != Vis::Private {
                return self.fail("a `test fn` is never `pub`: nothing reaches it but `cca test`");
            }
            self.bump();
            let kind = self.item_kind(vis)?;
            let kind = match kind {
                ItemKind::Fn(mut d) => {
                    d.test = true;
                    ItemKind::Fn(d)
                }
                other => other,
            };
            let span = start.to(self.prev_span());
            return Ok(Item { vis, kind, span });
        }
        let kind = self.item_kind(vis)?;
        let span = start.to(self.prev_span());
        Ok(Item { vis, kind, span })
    }

    fn item_kind(&mut self, vis: Vis) -> Result<ItemKind> {
        let w = match self.peek().clone() {
            Tok::Word(w) => w,
            Tok::Name(n) => return self.fail(format!("a Limb-level value is a `const`: `const '{}' := …;` — a variable lives inside a body", n)),
            other => return self.fail(format!("expected a top-level item (`fn`, `struct`, `enum`, `const`, `module`, `MAIN`), found {}", other.describe())),
        };
        match w.as_str() {
            "MAIN" => {
                self.bump();
                // C's own two (owner, 2026-09-29: "low-level shit,
                // decided"), or none: the types are checked with the rest.
                let params = if self.is(&Tok::LBracket) {
                    let at = self.span();
                    let ps = self.params(ParamContext::Fn)?;
                    match ps.len() {
                        0 | 2 => {}
                        1 => return self.fail_at(at, "`MAIN` takes C's two parameters or none: `,` and the second parameter — `MAIN ['argc': int32, 'argv': ptr[ptr[uint8]]] { … }`"),
                        _ => return self.fail_at(at, "`MAIN` takes C's two parameters or none — `MAIN ['argc': int32, 'argv': ptr[ptr[uint8]]] { … }`"),
                    }
                    ps
                } else {
                    Vec::new()
                };
                let body = self.block("MAIN")?;
                Ok(ItemKind::Main(params, body))
            }
            "fn" => {
                self.bump();
                if self.is_word("MAIN") {
                    return self.fail("`MAIN` is the program's block, not a function: write `MAIN { … }` with no `fn`, no return type, no `return`");
                }
                let decl = self.fn_decl(true)?;
                Ok(ItemKind::Fn(decl))
            }
            "func" => self.fail("`func` became `fn`: `fn name [params] -> type { … }`"),
            "extern" => {
                self.bump();
                // `extern` names its language (owner, 2026-10-02): `extern.C`,
                // a line or a block.
                if !self.eat(&Tok::Dot) {
                    return self.fail("`extern` names its language: `extern.C fn exit ['code': int32];`, or a block, `extern.C { … }`");
                }
                let lang = self.ident("the language after `extern.`")?;
                match lang.name.as_str() {
                    "C" => {}
                    "c" => return self.fail_at(lang.span, "C is written capitalized here, as every language's name is: `extern.C`"),
                    other => return self.fail_at(lang.span, format!("only `extern.C` exists: `extern.{}` is not a language CyborgPL reaches", other)),
                }
                if self.is(&Tok::LBrace) {
                    let open = self.bump().span;
                    let mut lines = Vec::new();
                    while !self.is(&Tok::RBrace) {
                        // Another item's first word at the left margin, or the
                        // end of the file: the block was never closed.
                        if self.at_eof() || self.is_word("MAIN") || self.is_word("struct") || self.is_word("enum") || self.is_word("const") || self.is_word("module") || self.is_word("extern") || self.is_word("layout") {
                            return self.fail_at(open, "the `extern.C {` block is never closed: is its `}` missing?");
                        }
                        // A line may say its own `pub` or `pub.limb`; one
                        // before the block overrides them (owner, 2026-10-02:
                        // "allow mixing, but block overrides").
                        let mut line_vis = Vis::Private;
                        if self.eat_word("pub") {
                            line_vis = Vis::Pub;
                            if self.eat(&Tok::Dot) {
                                let narrowing = self.ident("a word after `pub.`")?;
                                if narrowing.name != "limb" {
                                    return self.fail_at(narrowing.span, format!("`pub.{}` is not a visibility: the only narrowing of `pub` is `pub.limb`", narrowing.name));
                                }
                                line_vis = Vis::Limb;
                            }
                        }
                        lines.push((line_vis, self.extern_line()?));
                    }
                    self.bump();
                    return Ok(ItemKind::ExternBlock(lines));
                }
                self.extern_line()
            }
            "struct" => {
                self.bump();
                Ok(ItemKind::Struct(self.struct_decl(false)?))
            }
            "layout" => {
                self.bump();
                self.expect(&Tok::Dot, "`.C` after `layout`")?;
                let which = self.ident("`C` after `layout.`")?;
                // A language's name is capitalized (owner, 2026-10-02).
                if which.name == "c" {
                    return self.fail_at(which.span, "C is written capitalized here, as every language's name is: `layout.C struct`, `layout.C enum`");
                }
                if which.name != "C" {
                    return self.fail_at(which.span, format!("the only layout is `layout.C`, C's shape; `layout.{}` does not exist", which.name));
                }
                if self.eat_word("enum") {
                    return Ok(ItemKind::Enum(self.enum_decl(true)?));
                }
                if !self.is_word("struct") {
                    return self.fail(format!("`layout.C` is followed by `struct` or `enum`, found {}", self.peek().describe()));
                }
                self.bump();
                Ok(ItemKind::Struct(self.struct_decl(true)?))
            }
            "enum" => {
                self.bump();
                Ok(ItemKind::Enum(self.enum_decl(false)?))
            }
            "interface" => {
                self.bump();
                Ok(ItemKind::Interface(self.iface_decl()?))
            }
            "op" => {
                self.bump();
                Ok(ItemKind::Op(self.op_decl()?))
            }
            "const" => {
                self.bump();
                Ok(ItemKind::Const(self.const_decl()?))
            }
            "module" => {
                self.bump();
                let name = self.ident("the module's name")?;
                if self.is(&Tok::Semi) {
                    return self.fail(format!("there is no include line: a file module is found by its file name, and `module {} {{ … }}` is a block — nothing declares a module with `;`", name.name));
                }
                self.check_module_name(&name)?;
                self.expect(&Tok::LBrace, "`{` after the module's name")?;
                let mut items = Vec::new();
                while !self.is(&Tok::RBrace) {
                    if self.at_eof() {
                        return self.fail(format!("module `{}` is never closed", name.name));
                    }
                    push_item(&mut items, self.item()?);
                }
                self.bump();
                Ok(ItemKind::Module { name, items })
            }
            "mod" => self.fail("`mod` is the remainder operator; a module is `module name { … }`"),
            "let" => self.fail("a Limb-level value is a `const`, not a `let`: `const 'name' := …;`"),
            "check" | "nocheck" => {
                if self.is_word_at(1, "fn") {
                    self.fail(format!("`{}` cannot go on a `fn`: the tiers are never global — there is no file-wide or Limb-wide tier; put `{}` in front of the statements inside", w, w))
                } else {
                    self.fail(format!("`{}` goes in front of a statement inside a body, never at the top level: the tiers are never global", w))
                }
            }
            "inline" => self.fail("`inline.ASM` goes inside a body, as a statement, under `nocheck`"),
            "var" => self.fail("`var` is gone, and a Limb's top-level value is a `const`: `const 'name' := …;`"),
            _ => {
                // `Vec satisfies Ordered;`
                if !is_keyword(&w) {
                    let save = self.pos;
                    if let Ok(ty) = self.type_() {
                        if self.eat_word("satisfies") {
                            let iface = self.path()?;
                            self.expect_semi("the claim")?;
                            let _ = vis;
                            return Ok(ItemKind::Satisfies { ty, iface });
                        }
                    }
                    self.pos = save;
                    if self.is(&Tok::LBracket) || matches!(self.peek_at(1), Tok::LBracket) {
                        return self.fail(format!("a call to `{}` cannot stand at the top level: statements go inside `MAIN {{ }}` or a `fn`", w));
                    }
                }
                self.fail(format!("expected a top-level item (`fn`, `struct`, `enum`, `const`, `module`, `MAIN`), found `{}`", w))
            }
        }
    }

    fn check_module_name(&self, name: &Ident) -> Result<()> {
        let n = &name.name;
        let ok = n.chars().next().map(|c| c.is_ascii_lowercase()).unwrap_or(false) && n.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !ok {
            return self.fail_at(name.span, format!("`{}` cannot name a module: module names are lowercase ASCII letters, digits and `_`, starting with a letter (lowercase only, so the name works on every file system)", n));
        }
        if is_device_name(n) {
            return self.fail_at(name.span, format!("`{}` is a reserved device name on Windows and cannot name a module", n));
        }
        if is_keyword(n) {
            return self.fail_at(name.span, format!("`{}` is a keyword and cannot name a module", n));
        }
        Ok(())
    }

    /// `>` closing a `<T>`.
    fn expect_close_angle(&mut self, name: &str) -> Result<()> {
        match self.peek().clone() {
            Tok::Op(o) if o == ">" => {
                self.bump();
                Ok(())
            }
            Tok::Op(o) if o.starts_with('>') => self.fail(format!("`{}` is one operator: put a space after the `>` closing `<{}>`", o, name)),
            Tok::Colon => self.fail(format!("the bound goes after the bracket: `<{}>: any`, not `<{}: any>`", name, name)),
            other => self.fail(format!("expected `>` closing `<{}>`, found {}", name, other.describe())),
        }
    }

    /// What a declaration announces right after its name (owner,
    /// 2026-10-02): each generic in its own `< >` with what it accepts
    /// after a `:` — `<T>: Number and Hashable`, `<A>: any` — then the
    /// lifetime groups, `@x @y`, with a `|` between the two kinds only.
    fn announce(&mut self, owner: &str, what: &str) -> Result<Announce> {
        let mut a = Announce::default();
        while self.is_op("<") && matches!(self.peek_at(1), Tok::Word(_)) {
            self.bump();
            let name = self.ident("the generic's name")?;
            self.expect_close_angle(&name.name)?;
            if !self.eat(&Tok::Colon) {
                return self.fail_at(name.span, format!("`<{}>` says what it accepts, never every type by default: `{} {} <{}>: any`, or an interface — `<{}>: Number`", name.name, what, owner, name.name, name.name));
            }
            let mut bounds = Vec::new();
            if self.is_word("any") {
                let any = self.bump().span;
                if self.is_word("and") {
                    return self.fail_at(any, format!("`any` stands alone: `<{}>: any` accepts every type, so nothing joins it with `and`", name.name));
                }
            } else {
                loop {
                    if self.is_word("any") {
                        return self.fail(format!("`any` stands alone: `<{}>: any` accepts every type, so nothing joins it with `and`", name.name));
                    }
                    bounds.push(self.path()?);
                    if !self.eat_word("and") {
                        break;
                    }
                }
            }
            a.generics.push(Generic { name, bounds });
        }
        let bar = if self.is_op("|") {
            let at = self.bump().span;
            if a.generics.is_empty() {
                return self.fail_at(at, format!("a `|` only separates generics from lifetime groups; with groups alone they follow the name: `{} {} @x […]`", what, owner));
            }
            true
        } else {
            false
        };
        while self.is_op("@") {
            let at = self.bump().span;
            match self.peek().clone() {
                Tok::Word(_) => a.groups.push(self.ident("the lifetime group's name")?),
                _ => return self.fail_at(at, "a lifetime group is named after `@`: `@x`"),
            }
            if !bar && !a.generics.is_empty() {
                return self.fail_at(at, format!("a `|` separates the generics from the lifetime groups: `{} {} <{}>: … | @{} […]`", what, owner, a.generics[0].name.name, a.groups[0].name));
            }
        }
        if bar && a.groups.is_empty() {
            return self.fail(format!("a lifetime group follows the `|`: `| @x`, found {}", self.peek().describe()));
        }
        Ok(a)
    }

    /// One of C's names after `extern.C`, alone or in its block: `fn …;`,
    /// or a variable, `'x': T;` (`mut 'x': T;` to write it; owner,
    /// 2026-09-25).
    fn extern_line(&mut self) -> Result<ItemKind> {
        if self.eat_word("fn") {
            let decl = self.fn_decl(false)?;
            return Ok(ItemKind::ExternFn(decl));
        }
        if self.is_word("let") {
            return self.fail("C's variable has no `let`: `extern.C 'x': T;` (`extern.C mut 'x': T;` to write it)");
        }
        if !(matches!(self.peek(), Tok::Name(_)) || self.is_word("mut") || self.is_word("immut")) {
            return self.fail("`extern.C` is followed by `fn` (a C function) or a quoted name (a C variable: `extern.C 'x': int32;`)");
        }
        let mutable = self.eat_word("mut");
        if self.is_word("immut") {
            return self.fail("`immut` means nothing on C's variable: it is read-only unless you write `mut`");
        }
        let name = self.qname("the C variable's name")?;
        if !self.is(&Tok::Colon) {
            return self.fail("C's variable needs a type: it has a C-shaped type and no value here — `extern.C 'x': int32;`");
        }
        self.bump();
        let ty = self.type_()?;
        if self.is_op("=") || self.is(&Tok::Walrus) {
            return self.fail("C's variable has no value here: C owns it; write only its type");
        }
        self.expect_semi("the extern declaration")?;
        Ok(ItemKind::ExternLet { mutable, name, ty })
    }

    fn fn_decl(&mut self, has_body: bool) -> Result<FnDecl> {
        if let Tok::Name(n) = self.peek().clone() {
            return self.fail(format!("function names are bare: `fn {} […]`, not `fn '{}'` — quotes mark a value you hold, and a function is not one", n, n));
        }
        if self.is(&Tok::Dot) {
            return self.fail("the return type goes after the parameters — `fn name [params] -> int32 { … }` — not on `fn`");
        }
        // A keyword may name a function — `fn true`, `fn to` — reached by
        // path, never bare (limbs.md, 2026-09-19): `limb.true[…]`.
        let name = self.ident("the function's name")?;
        let announce = self.announce(&name.name, "fn")?;
        let params = self.params(if has_body { ParamContext::Fn } else { ParamContext::Extern })?;
        if self.is(&Tok::Colon) {
            return self.fail("the return type is `-> int32`, not `: int32` — `:` attaches a type to a name, and a result is not the function's type");
        }
        let (ret, fails) = self.return_spec()?;
        let ret_lent = self.ret_lent.take();
        // A strict caller sees only signatures (owner, 2026-09-29).
        if let (true, Some(RetLent { group: None, span, .. })) = (self.strict, &ret_lent) {
            return self.fail_at(*span, format!("`{}` hands back a borrow, so it names what the borrow depends on: `-> lent@x …`, with `lent@x` on those parameters — a `#strict` file writes it out", name.name));
        }
        let body = if has_body {
            if self.is(&Tok::Semi) {
                return self.fail(format!("`{}` has no body: a function without one is an `extern fn`", name.name));
            }
            let outer = std::mem::replace(&mut self.returns_value, false);
            let b = self.block("the function")?;
            if self.strict && self.returns_value && ret.is_none() {
                return self.fail_at(name.span, format!("`{}` returns a value, so it says its type: `fn {} […] -> int32 {{ … }}` — a `#strict` file writes it out", name.name, name.name));
            }
            self.returns_value = outer;
            Some(b)
        } else {
            if self.is(&Tok::LBrace) {
                return self.fail(format!("an `extern fn` has no body: `{}`'s body is in C — end the declaration with `;`", name.name));
            }
            self.expect_semi("the extern declaration")?;
            None
        };
        Ok(FnDecl { name, announce, params, ret, ret_lent, fails, body, test: false })
    }

    /// The group's name, right after its `@`.
    fn group_after_at(&mut self, shape: &str) -> Result<Ident> {
        match self.peek().clone() {
            Tok::Word(_) => self.ident("the lifetime group's name after `@`"),
            _ => self.fail(format!("a lifetime group is named after `@`: `{}`", shape)),
        }
    }

    /// `-> T`, `-> T or fail[…]`, `-> fail[…]`, or nothing.
    fn return_spec(&mut self) -> Result<(Option<Type>, Vec<FailSpec>)> {
        if let Tok::Op(o) = self.peek().clone() {
            if o.starts_with("->") && o != "->" {
                return self.fail(format!("`{}` is one operator: put a space after `->` (`-> <T>`)", o));
            }
        }
        self.ret_lent = None;
        if !self.eat_op("->") {
            return Ok((None, Vec::new()));
        }
        // `-> lent T`, `-> lent.mut@x T`: a borrow handed back (owner,
        // 2026-09-29). `lent[…]` would be a call, and never follows `->`.
        if let Tok::Op(o) = self.peek().clone() {
            if o == "&" || o == "&@" {
                let start = self.bump().span;
                let mutable = o == "&" && self.eat_word("mut");
                let group = if o == "&@" {
                    Some(self.group_after_at("&@x")?)
                } else if self.eat_op("@") {
                    Some(self.group_after_at(if mutable { "&mut@x" } else { "&@x" })?)
                } else {
                    None
                };
                self.ret_lent = Some(RetLent { mutable, group, span: start.to(self.prev_span()) });
            }
        }
        if self.ret_lent.is_none() && self.is_word("lent") {
            let start = self.bump().span;
            let mutable = if self.eat(&Tok::Dot) {
                let m = self.ident("`mut` after `lent.`")?;
                if m.name != "mut" {
                    return self.fail_at(m.span, format!("`lent.{}` is not a borrow: `-> lent T` or `-> lent.mut T`", m.name));
                }
                true
            } else {
                false
            };
            let group = if self.eat_op("@") { Some(self.group_after_at("lent@x")?) } else { None };
            self.ret_lent = Some(RetLent { mutable, group, span: start.to(self.prev_span()) });
        }
        if self.is_word("fail") {
            self.bump();
            let fails = self.fail_list()?;
            return Ok((None, fails));
        }
        let ret = self.type_()?;
        let mut fails = Vec::new();
        if self.is_word("or") {
            self.bump();
            self.expect_word("fail")?;
            fails = self.fail_list()?;
        }
        Ok((Some(ret), fails))
    }

    /// `[a, b: T]` after `fail`.
    fn fail_list(&mut self) -> Result<Vec<FailSpec>> {
        self.expect(&Tok::LBracket, "`[` after `fail`")?;
        let mut specs = Vec::new();
        loop {
            if self.is(&Tok::RBracket) {
                break;
            }
            if let Tok::Name(n) = self.peek().clone() {
                return self.fail(format!("a failure's name is bare — `fail[{}]`, not `fail['{}']` — it is a reason, not a value you hold", n, n));
            }
            let name = self.ident("a failure name")?;
            let payload = if self.eat(&Tok::Colon) { Some(self.type_()?) } else { None };
            specs.push(FailSpec { name, payload });
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(&Tok::RBracket, "`]` closing the failure list")?;
        if specs.is_empty() {
            return self.fail("`fail[]` lists no failure: name at least one reason");
        }
        Ok(specs)
    }

    fn params(&mut self, ctx: ParamContext) -> Result<Vec<Param>> {
        if !self.is(&Tok::LBracket) {
            if ctx == ParamContext::Fn && self.is_op("=") {
                return self.fail("not yet: `pub fn short = module.name;` gives other Limbs a shorter name (owner, 2026-09-23), and CCA builds one Limb so far");
            }
            return self.fail(format!("expected `[` starting the parameters, found {}", self.peek().describe()));
        }
        self.bump();
        let mut params = Vec::new();
        loop {
            if self.is(&Tok::RBracket) {
                break;
            }
            let p = self.param(ctx)?;
            let variadic = p.variadic;
            params.push(p);
            if !self.eat(&Tok::Comma) {
                break;
            }
            if variadic && !self.is(&Tok::RBracket) {
                return self.fail("a variadic `...'name'` must be the last parameter: the rest of the arguments go to it");
            }
        }
        self.expect(&Tok::RBracket, "`]` closing the parameters")?;
        if self.strict {
            if let Some(p) = params.iter().find(|p| p.ty.is_none()) {
                return self.fail_at(p.name.span, format!("parameter '{}' says its type: `'{}': int32` — a `#strict` file writes every parameter's type", p.name.name, p.name.name));
            }
        }
        Ok(params)
    }

    fn param(&mut self, ctx: ParamContext) -> Result<Param> {
        let start = self.span();
        // An operator's operands are lent unless the op says `give`.
        let mut kind = if ctx == ParamContext::Op { ParamKind::Lent } else { ParamKind::Owned };
        let mut amp = false;
        let mut mutability = None;
        let mut variadic = false;
        let mut group = None;
        match self.peek().clone() {
            Tok::Word(w) if w == "lent" => {
                self.bump();
                if self.eat(&Tok::Dot) {
                    let m = self.ident("`mut` after `lent.`")?;
                    if m.name != "mut" {
                        return self.fail_at(m.span, format!("`lent.{}` is not a parameter kind: `lent` (looks at it) or `lent.mut` (may modify it)", m.name));
                    }
                    kind = ParamKind::LentMut;
                } else {
                    kind = ParamKind::Lent;
                }
                if self.eat_op("@") {
                    group = Some(self.group_after_at("lent@x")?);
                }
                if ctx == ParamContext::Op {
                    return self.fail_at(start, "an operator's operands are lent by default: write `'a': T` with no `lent`; `give 'a': T` takes it by value");
                }
                if ctx == ParamContext::Extern {
                    return self.fail_at(start, "an `extern fn` parameter is not `lent`: C receives values and pointers, so write `'p': ptr[T]` and hand it a lend at the call");
                }
            }
            // `&` is `lent` everywhere (owner, 2026-10-02): `&@x`, `&mut@x`.
            Tok::Op(o) if o == "&" || o == "&@" => {
                self.bump();
                amp = true;
                if o == "&@" {
                    kind = ParamKind::Lent;
                    group = Some(self.group_after_at("&@x")?);
                } else if self.eat_word("mut") {
                    kind = ParamKind::LentMut;
                    if self.eat_op("@") {
                        group = Some(self.group_after_at("&mut@x")?);
                    }
                } else {
                    kind = ParamKind::Lent;
                }
            }
            Tok::Word(w) if w == "lend" => {
                return self.fail("a parameter that receives a borrow is `lent 'x': T` (the caller writes `lend`); `lend` is the caller's word");
            }
            Tok::Word(w) if w == "give" => {
                self.bump();
                if ctx != ParamContext::Op {
                    return self.fail_at(start, "an owning parameter has no keyword — `'x': T` — and the caller writes `give` or `copy`");
                }
                kind = ParamKind::Owned;
                let p = self.param_rest(start, kind, amp, mutability, true)?;
                return Ok(p);
            }
            Tok::Word(w) if w == "var" => {
                return self.fail("a parameter has no `var`: an owning parameter is bare — `'a': int32`");
            }
            Tok::Word(w) if w == "mut" || w == "immut" => {}
            Tok::Word(w) if is_primitive_head(&w) || (!is_keyword(&w) && matches!(self.peek_at(1), Tok::Name(_))) => {
                return self.fail("a parameter is written name first, then its type: `'a': int32`");
            }
            Tok::Ellipsis => {
                self.bump();
                variadic = true;
            }
            _ => {}
        }
        if self.is_word("mut") {
            self.bump();
            mutability = Some(Mutability::Mut);
        } else if self.is_word("immut") {
            self.bump();
            mutability = Some(Mutability::Immut);
        }
        if !variadic && self.is(&Tok::Ellipsis) {
            self.bump();
            variadic = true;
        }
        let mut p = self.param_rest(start, kind, amp, mutability, false)?;
        p.variadic = variadic;
        p.group = group;
        if variadic && ctx == ParamContext::Extern {
            // C's own varargs: allowed, with no type.
        }
        Ok(p)
    }

    fn param_rest(&mut self, start: Span, kind: ParamKind, amp: bool, mutability: Option<Mutability>, gives: bool) -> Result<Param> {
        let _ = gives;
        let name = match self.peek().clone() {
            Tok::Name(n) => {
                let span = self.bump().span;
                QName { name: n, span }
            }
            Tok::Word(w) if w == "lent" && matches!(self.peek_at(1), Tok::Dot) => {
                return self.fail("`lent` and the type both follow their own rules: `lent 'text': str.utf8` — the verb before the name, the type after it");
            }
            Tok::Word(w) => {
                return self.fail(format!("a parameter's name is quoted: `'{}': int32`", w));
            }
            other => return self.fail(format!("expected a parameter name, found {}", other.describe())),
        };
        if self.is(&Tok::Ellipsis) {
            return self.fail("a variadic parameter is `...'name'`: the dots go before the name");
        }
        let ty = if self.eat(&Tok::Colon) { Some(self.type_()?) } else { None };
        let span = start.to(self.prev_span());
        Ok(Param { kind, group: None, amp, mutability, variadic: false, name, ty, span })
    }

    fn struct_decl(&mut self, layout_c: bool) -> Result<StructDecl> {
        let name = self.ident("the struct's name")?;
        if let Some(t) = crate::resolve::rust_alias(&name.name) {
            return self.fail_at(name.span, format!("`{}` is another name for `{}` (Rust's spelling of it), so it cannot name a struct", name.name, t));
        }
        let announce = self.announce(&name.name, "struct")?;
        if !announce.groups.is_empty() {
            return self.fail_at(announce.groups[0].span, "not yet: a struct holding a borrow, and so a lifetime group, is not decided (asked)");
        }
        self.expect(&Tok::LBrace, "`{` after the struct's name")?;
        let mut fields = Vec::new();
        while !self.is(&Tok::RBrace) {
            if self.at_eof() {
                return self.fail(format!("struct `{}` is never closed", name.name));
            }
            let mut immut = false;
            if self.is_word("immut") {
                self.bump();
                if self.is(&Tok::Dot) {
                    return self.fail("`immut` is a bare word before the field's name: `immut number: int32`");
                }
                immut = true;
            } else if self.is_word("mut") {
                return self.fail("a struct field is not `mut`: a struct always owns its fields and may change them unless the field is `immut`");
            } else if self.is_word("let") || self.is_word("var") {
                return self.fail("a field has no `let`: a struct always owns its fields — write `name: type`");
            }
            let fname = match self.peek().clone() {
                Tok::Word(w) => {
                    if is_keyword(&w) && !matches!(self.peek_at(1), Tok::Name(_)) {
                        if matches!(self.peek_at(1), Tok::Colon) {
                            return self.fail(format!("`{}` is a keyword and cannot name a field", w));
                        }
                    }
                    if !matches!(self.peek_at(1), Tok::Colon) {
                        if let Tok::Name(n) = self.peek_at(1).clone() {
                            return self.fail(format!("a field is written name first, then its type: `{}: {}`", n, w));
                        }
                        return self.fail(format!("a field's type follows its name after `:` — `{}: int32`", w));
                    }
                    self.ident("a field name")?
                }
                Tok::Name(n) => return self.fail(format!("a field's name is bare — `{}: int32` — it is a path into the struct, not a value you hold", n)),
                other => return self.fail(format!("expected a field (`name: type`) or `}}`, found {}", other.describe())),
            };
            self.expect(&Tok::Colon, "`:` after the field's name")?;
            let ty = self.type_()?;
            if self.is(&Tok::Semi) {
                return self.fail("no `;` after a field: fields are not statements — one per line, nothing between");
            }
            if self.is(&Tok::Comma) {
                return self.fail("no `,` between fields: one per line, nothing between");
            }
            fields.push(Field { immut, name: fname, ty });
        }
        self.bump();
        Ok(StructDecl { name, announce, layout_c, fields })
    }

    fn enum_decl(&mut self, layout_c: bool) -> Result<EnumDecl> {
        let name = self.ident("the enum's name")?;
        if let Some(t) = crate::resolve::rust_alias(&name.name) {
            return self.fail_at(name.span, format!("`{}` is another name for `{}` (Rust's spelling of it), so it cannot name a enum", name.name, t));
        }
        let announce = self.announce(&name.name, "enum")?;
        if !announce.groups.is_empty() {
            return self.fail_at(announce.groups[0].span, "not yet: an enum holding a borrow, and so a lifetime group, is not decided (asked)");
        }
        self.expect(&Tok::LBrace, "`{` after the enum's name")?;
        let mut cases = Vec::new();
        while !self.is(&Tok::RBrace) {
            if self.at_eof() {
                return self.fail(format!("enum `{}` is never closed", name.name));
            }
            let cname = match self.peek().clone() {
                Tok::Word(_) => self.ident("a case name")?,
                Tok::Name(n) => return self.fail(format!("a case's name is bare — `{}` — it is a path into the type, not a value you hold", n)),
                Tok::Comma => return self.fail("no `,` between cases: one per line, nothing between"),
                other => return self.fail(format!("expected a case name or `}}`, found {}", other.describe())),
            };
            let payload = if self.eat(&Tok::Colon) { Some(self.type_()?) } else { None };
            // `a = 65`: a C enum's case says its number (owner, 2026-09-27).
            let number = if self.is(&Tok::Op("=".into())) {
                let at = self.span();
                self.bump();
                let negative = self.is(&Tok::Op("-".into()));
                if negative {
                    self.bump();
                }
                let n = match self.peek().clone() {
                    Tok::Num { text, bin: false } => {
                        self.bump();
                        let v: Option<i128> = text.parse().ok();
                        match v {
                            Some(v) => v,
                            None => return self.fail(format!("C's enum is 32 bits, C's `int`: {} does not fit int32", text)),
                        }
                    }
                    other => return self.fail(format!("a C enum's case number is a whole number, found {}", other.describe())),
                };
                Some((if negative { -n } else { n }, at.to(self.prev_span())))
            } else {
                None
            };
            if self.is(&Tok::Comma) {
                return self.fail("no `,` between cases: one per line, nothing between");
            }
            if self.is(&Tok::Semi) {
                return self.fail("no `;` after a case: cases are not statements");
            }
            cases.push(EnumCase { name: cname, payload, number });
        }
        self.bump();
        if cases.is_empty() {
            return self.fail_at(name.span, format!("enum `{}` has no cases: an enum is one of a few named cases, so name at least one", name.name));
        }
        Ok(EnumDecl { name, announce, cases, layout_c })
    }

    fn iface_decl(&mut self) -> Result<IfaceDecl> {
        let name = self.ident("the interface's name")?;
        if let Some(t) = crate::resolve::rust_alias(&name.name) {
            return self.fail_at(name.span, format!("`{}` is another name for `{}` (Rust's spelling of it), so it cannot name a interface", name.name, t));
        }
        if name.name == "any" {
            return self.fail_at(name.span, "`any` is the built-in that accepts every type (`<T>: any`), so it cannot name an interface");
        }
        // Its own `<T>`, which accepts nothing in particular: the bound is
        // the interface itself (owner, 2026-10-02).
        let own = if self.is_op("<") {
            self.bump();
            let t = self.ident("the interface's type's name")?;
            self.expect_close_angle(&t.name)?;
            if self.is(&Tok::Colon) {
                return self.fail(format!("an interface's own `<{}>` accepts nothing in particular — the interface is what it accepts: `interface {} <{}> {{ … }}`", t.name, name.name, t.name));
            }
            Some(t)
        } else {
            None
        };
        let open = self.expect(&Tok::LBrace, "`{` after the interface's name")?;
        let mut entries = Vec::new();
        let mut types: Vec<Type> = Vec::new();
        while !self.is(&Tok::RBrace) {
            if self.at_eof() {
                return self.fail(format!("interface `{}` is never closed", name.name));
            }
            let start = self.span();
            let ename = if self.eat_word("fn") {
                IfaceEntryName::Fn(self.ident("the function's name")?)
            } else if self.eat_word("op") {
                IfaceEntryName::Op(self.op_name()?)
            } else if matches!(self.peek(), Tok::Word(_)) {
                // A type it accepts, one per line, as enum cases go.
                if !types.is_empty() && !self.toks[self.pos].line_first {
                    return self.fail(format!("an interface lists its types one per line, as an enum lists its cases: `interface {} {{` and then each type on a line of its own", name.name));
                }
                types.push(self.type_()?);
                continue;
            } else {
                return self.fail(format!("an interface lists types, one per line, and `fn` and `op` signatures; found {}", self.peek().describe()));
            };
            let params = self.params(ParamContext::Iface)?;
            let (ret, fails) = self.return_spec()?;
        if let Some(r) = self.ret_lent.take() {
            return self.fail_at(r.span, "not yet: only a named `fn` hands back a borrow (`fn f [lent 'a': T] -> lent T`)");
        }
            if self.is(&Tok::LBrace) {
                return self.fail("an interface lists promises, not code: end the signature with `;` and leave the body out");
            }
            self.expect_semi("the signature")?;
            let span = start.to(self.prev_span());
            entries.push(IfaceEntry { name: ename, params, ret, fails, span });
        }
        self.bump();
        if entries.is_empty() && types.is_empty() {
            return self.fail_at(open.to(self.prev_span()), format!("interface `{}` promises nothing: list at least one `fn` or `op` signature, or the types it accepts, one per line", name.name));
        }
        if own.is_none() && !entries.is_empty() {
            return self.fail_at(name.span, format!("interface `{}` lists operations, so it announces the type they are about: `interface {} <T> {{ … }}`", name.name, name.name));
        }
        Ok(IfaceDecl { name, own, entries, types })
    }

    fn op_name(&mut self) -> Result<Ident> {
        match self.peek().clone() {
            Tok::Op(o) => {
                let span = self.bump().span;
                Ok(Ident { name: o, span })
            }
            Tok::Word(w) => {
                let span = self.bump().span;
                let mut name = w;
                // `and.bit` style word operators are the language's; a user op is one word.
                if self.is(&Tok::Dot) {
                    self.bump();
                    let n = self.ident("a word after `.`")?;
                    name = format!("{}.{}", name, n.name);
                }
                Ok(Ident { name, span: span.to(self.prev_span()) })
            }
            other => self.fail(format!("expected the operator's spelling (a glyph run or a word), found {}", other.describe())),
        }
    }

    fn op_decl(&mut self) -> Result<OpDecl> {
        let op = self.op_name()?;
        let announce = self.announce(&op.name, "op")?;
        let params = self.params(ParamContext::Op)?;
        if params.len() != 2 {
            return self.fail_at(op.span, format!("an operator takes exactly two operands; `{}` is declared with {}", op.name, params.len()));
        }
        if !self.is_op("->") {
            return self.fail(format!("`-> T` is required on an operator: `op {} [...] -> T {{ … }}`", op.name));
        }
        let (ret, fails) = self.return_spec()?;
        if let Some(r) = self.ret_lent.take() {
            return self.fail_at(r.span, "not yet: only a named `fn` hands back a borrow (`fn f [lent 'a': T] -> lent T`)");
        }
        if !fails.is_empty() {
            return self.fail_at(op.span, format!("an operator cannot fail; make `{}` a `fn` if it needs `fail[…]`", op.name));
        }
        if ret.is_none() {
            return self.fail_at(op.span, format!("`-> T` is required on an operator: `op {} [...] -> T {{ … }}`", op.name));
        }
        let body = self.block("the operator")?;
        Ok(OpDecl { op, announce, params, ret, body })
    }

    fn const_decl(&mut self) -> Result<ConstDecl> {
        if self.is_word("mut") || self.is_word("immut") {
            return self.fail("a `const` is neither `mut` nor `immut`: it is a value known at compile time, never assigned");
        }
        let name = match self.peek().clone() {
            Tok::Name(n) => {
                let span = self.bump().span;
                QName { name: n, span }
            }
            Tok::Word(w) => return self.fail(format!("a const's name is quoted, like a variable's: `const '{}' := …;`", w)),
            other => return self.fail(format!("expected the const's name, found {}", other.describe())),
        };
        let (ty, _shadow) = self.let_type_and_marker("const")?;
        let value = self.expr_top()?;
        self.expect_semi("the const")?;
        Ok(ConstDecl { name, ty, value })
    }

    /// After a `let`/`const` name: `:= `, `: T =`, and the `$` forms.
    fn let_type_and_marker(&mut self, what: &str) -> Result<(Option<Type>, bool)> {
        let ty = if self.eat(&Tok::Colon) { Some(self.type_()?) } else { None };
        let mut shadow = false;
        if self.eat(&Tok::Dollar) {
            if what == "const" {
                return self.fail("a `const` cannot shadow anything and cannot be shadowed");
            }
            shadow = true;
        }
        if self.eat(&Tok::Walrus) {
            if ty.is_some() {
                return self.fail_at(self.prev_span(), "`:=` means Precog chooses the type, so it takes no type; with a type written, use `=` — you said everything");
            }
        } else if self.eat_op("=") {
            if ty.is_none() {
                return self.fail_at(self.prev_span(), "`=` means you said everything, so it needs the type: `let 'x': int32 = …;` — or `:=` to let Precog choose");
            }
        } else if self.is(&Tok::Semi) {
            return self.fail(format!("a {} needs a value: there is no implicit zero — write `= 0` (or whatever it should hold)", what));
        } else if self.is_word("mut") || self.is_word("immut") {
            return self.fail(format!("expected `:=` or `=`; `{}` goes before the name: `{} 'x' := …`", self.word_text(), self.word_text()));
        } else {
            return self.fail(format!("expected `:=` or `=`, found {}", self.peek().describe()));
        }
        Ok((ty, shadow))
    }

    fn word_text(&self) -> String {
        match self.peek() {
            Tok::Word(w) => w.clone(),
            _ => String::new(),
        }
    }

    // ------------------------------------------------------------ types

    fn type_(&mut self) -> Result<Type> {
        self.type_in(false)
    }

    /// `conv`: after `to.`, where a final non-bound bracket is the value to convert.
    fn type_in(&mut self, conv: bool) -> Result<Type> {
        let start = self.span();
        let kind = match self.peek().clone() {
            Tok::Num { text, bin } => {
                let span = self.bump().span;
                if bin {
                    return self.fail_at(span, "a count is a whole number");
                }
                match text.parse::<u32>() {
                    Ok(n) => TypeKind::Count(n),
                    Err(_) => return self.fail_at(span, "this number is too large to be a count"),
                }
            }
            Tok::Op(o) if o == "<" => {
                self.bump();
                let name = match self.peek().clone() {
                    Tok::Word(_) => self.ident("the blank's name")?,
                    Tok::Name(n) => return self.fail(format!("a type variable's name is bare: `<{}>`, not `<'{}'>`", n, n)),
                    other => return self.fail(format!("expected the blank's name after `<`, found {}", other.describe())),
                };
                if self.is(&Tok::Colon) {
                    return self.fail(format!("what `<{}>` accepts is said once, where it is announced (`fn f <{}>: any […]`): a mention is `<{}>` alone", name.name, name.name, name.name));
                }
                let bound = None;
                match self.peek().clone() {
                    Tok::Op(o) if o == ">" => {
                        self.bump();
                    }
                    Tok::Op(o) if o.starts_with('>') => {
                        return self.fail(format!("`{}` is one operator: put a space after the `>` closing `<{}>`", o, name.name));
                    }
                    other => return self.fail(format!("expected `>` closing `<{}>`, found {}", name.name, other.describe())),
                }
                TypeKind::Var { name, bound }
            }
            Tok::Op(o) if o.starts_with('<') => {
                return self.fail(format!("`{}` is one operator: put a space around a `<T>` so `<` stands alone", o));
            }
            Tok::Word(w) if w == "fn" => {
                self.bump();
                self.expect(&Tok::LBracket, "`[` after `fn` in a function type")?;
                let mut params = Vec::new();
                while !self.is(&Tok::RBracket) {
                    let mut kind = ParamKind::Owned;
                    let mut amp = false;
                    if self.eat_word("lent") {
                        kind = ParamKind::Lent;
                        if self.eat(&Tok::Dot) {
                            let m = self.ident("`mut`")?;
                            if m.name != "mut" {
                                return self.fail_at(m.span, "`lent` or `lent.mut`");
                            }
                            kind = ParamKind::LentMut;
                        }
                    } else if self.is_op("&") {
                        self.bump();
                        amp = true;
                        kind = if self.eat_word("mut") { ParamKind::LentMut } else { ParamKind::Lent };
                    }
                    if let Tok::Name(n) = self.peek().clone() {
                        return self.fail(format!("a function type has no parameter names: `fn[int32] -> bool`, not `fn['{}': int32]` — it is the signature without the names", n));
                    }
                    let ty = self.type_()?;
                    params.push(FnTypeParam { kind, amp, ty });
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
                self.expect(&Tok::RBracket, "`]` closing the function type's parameters")?;
                let (ret, fails) = self.return_spec()?;
        if let Some(r) = self.ret_lent.take() {
            return self.fail_at(r.span, "not yet: only a named `fn` hands back a borrow (`fn f [lent 'a': T] -> lent T`)");
        }
                TypeKind::Fn { params, ret: ret.map(Box::new), fails }
            }
            Tok::Word(w) if w == "list" && matches!(self.peek_at(1), Tok::Dot) => {
                return self.fail("the element type goes in brackets: `list[int32]`, not `list.int32` (a dot picks a fixed variant, brackets supply a type)");
            }
            Tok::Word(w) if w == "int" || w == "uint" || w == "bin" || w == "deci" => {
                return self.fail(format!("`{}` needs a width: the typed form is always fully explicit — `{}32`, `{}64`, …", w, w, w));
            }
            Tok::Word(w) if w == "str" && !matches!(self.peek_at(1), Tok::Dot) => {
                return self.fail("`str` is never bare: say the encoding — `str.utf8`, `str.utf16`, `str.utf32`, `str.ascii`");
            }
            Tok::Word(w) if w == "char" && !matches!(self.peek_at(1), Tok::Dot) => {
                return self.fail("`char` is never bare: say the unit — `char.point`, `char.grapheme`, `char.byte`");
            }
            Tok::Word(w) if w == "range" => {
                return self.fail("`range` builds its bounds from values: `for 'i': int32 in range[0, 3]` — the type goes on the counter's name");
            }
            Tok::Word(_) => {
                let path = self.path()?;
                let head = path.head().to_string();
                let mut args = Vec::new();
                let container = path.limb.is_none() && matches!(head.as_str(), "list" | "array" | "map" | "channel" | "ptr");
                let user_type = !is_primitive_head(&head) && !is_keyword(&head) || path.limb.is_some();
                let takes_type_args = container || (user_type && !conv);
                if self.is(&Tok::LBracket) && takes_type_args && !self.bracket_is_bound() {
                    self.bump();
                    while !self.is(&Tok::RBracket) {
                        args.push(self.type_()?);
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                    self.expect(&Tok::RBracket, "`]` closing the type parameters")?;
                } else if self.is(&Tok::LBracket) && is_primitive_head(&head) && !self.bracket_is_bound() && !conv {
                    if head == "bool" || head.starts_with("str") || head.starts_with("char") {
                        return self.fail(format!("`{}` takes no type parameters; a bracket after it is a bound — `[lo, hi]`", path.spelled()));
                    }
                    return self.fail(format!("`{}` takes no type parameters; a bracket after it is a range — `{}[0, 100]`", path.spelled(), path.spelled()));
                }
                TypeKind::Path { path, args }
            }
            Tok::Name(n) => return self.fail(format!("expected a type, found the name '{}'", n)),
            other => return self.fail(format!("expected a type, found {}", other.describe())),
        };
        let mut ty = Type { span: start.to(self.prev_span()), kind };
        // A bound: `T[lo, hi]`, `str.utf8[1, 40 graphemes]`, `list[T][0, 10]`.
        while self.is(&Tok::LBracket) && self.bracket_is_bound() {
            let bstart = self.span();
            self.bump();
            let lo = self.bound_value()?;
            self.expect(&Tok::Comma, "`,` between the bounds")?;
            let hi = self.bound_value()?;
            let unit = if let Tok::Word(w) = self.peek().clone() {
                if !is_keyword(&w) && w != "x" {
                    Some(self.ident("a unit")?)
                } else {
                    None
                }
            } else {
                None
            };
            self.expect(&Tok::RBracket, "`]` closing the bound")?;
            let span = bstart.to(self.prev_span());
            ty = Type { span: start.to(self.prev_span()), kind: TypeKind::Bound { inner: Box::new(ty), lo: Box::new(lo), hi: Box::new(hi), unit, span } };
        }
        Ok(ty)
    }

    /// Is the bracket at the cursor `[expr, expr [unit]]` — a bound — rather
    /// than type arguments or a call's argument? Decided by a trial parse.
    fn bracket_is_bound(&mut self) -> bool {
        let save = self.pos;
        let mut ok = false;
        if self.eat(&Tok::LBracket) {
            if self.bound_end() && self.eat(&Tok::Comma) && self.bound_end() {
                if let Tok::Word(w) = self.peek().clone() {
                    if !is_keyword(&w) {
                        self.bump();
                    }
                }
                ok = self.is(&Tok::RBracket);
            }
        }
        self.pos = save;
        ok
    }

    /// One end of a bound as written: `min` or `max`, the base type's own
    /// limit (`int32[1, max]`), or a value.
    fn bound_value(&mut self) -> Result<Expr> {
        if let Tok::Word(w) = self.peek().clone() {
            if w == "min" || w == "max" {
                let span = self.bump().span;
                let path = Path { limb: None, segs: vec![Ident { name: w, span }], span };
                return Ok(Expr { kind: ExprKind::Path(path), span });
            }
        }
        self.expr(0)
    }

    /// One end of a bound: a number, possibly negative, or `min` / `max`.
    fn bound_end(&mut self) -> bool {
        if let Tok::Word(w) = self.peek() {
            if w == "min" || w == "max" {
                self.bump();
                return true;
            }
        }
        if self.is_op("-") {
            self.bump();
        }
        if let Tok::Num { .. } = self.peek() {
            self.bump();
            true
        } else {
            false
        }
    }

    // ------------------------------------------------------------ blocks and statements

    fn block(&mut self, what: &str) -> Result<Block> {
        let start = match self.peek() {
            Tok::LBrace => self.bump().span,
            Tok::Raw(_) => return self.fail("only `inline.ASM` holds raw text"),
            other => return self.fail(format!("expected `{{` opening {}'s body, found {}", what, other.describe())),
        };
        let mut stmts = Vec::new();
        while !self.is(&Tok::RBrace) {
            if self.at_eof() {
                return self.fail_at(start, format!("{}'s `{{` is never closed", what));
            }
            stmts.push(self.stmt()?);
        }
        let end = self.bump().span;
        Ok(Block { stmts, span: start.to(end) })
    }

    fn stmt(&mut self) -> Result<Stmt> {
        let start = self.span();
        let kind = self.stmt_kind()?;
        Ok(Stmt { kind, span: start.to(self.prev_span()) })
    }

    /// A declaration, from `let` to its `;` (owner, 2026-10-02): `let 'x'
    /// := …`, `let 'x': T = …`, `mut`/`immut`/`!mut` and `!$` between `let`
    /// and the name, `$` before the `:=`/`=` to shadow.
    fn declaration(&mut self) -> Result<StmtKind> {
        if self.is(&Tok::Dot) {
            return self.fail("`let` is a word of its own: `let mut 'x' := …;`");
        }
        let mut sealed = false;
        let mut mutability = None;
        loop {
            if self.is_op("!") {
                self.bump();
                if self.eat(&Tok::Dollar) {
                    sealed = true;
                } else if self.eat_word("mut") {
                    mutability = Some(Mutability::Immut);
                } else {
                    return self.fail("`!` in front of a declaration is `!$` (the name cannot be shadowed) or `!mut` (the value cannot be changed)");
                }
            } else if self.eat_word("mut") {
                mutability = Some(Mutability::Mut);
            } else if self.eat_word("immut") {
                mutability = Some(Mutability::Immut);
            } else {
                break;
            }
        }
        if self.strict && mutability.is_none() {
            if let Tok::Name(n) = self.peek().clone() {
                return self.fail(format!("a declaration says whether it changes: `let mut '{}' := …;` or `let immut '{}' := …;` — a `#strict` file writes it out", n, n));
            }
        }
        let name = match self.peek().clone() {
            Tok::Name(n) => {
                let span = self.bump().span;
                QName { name: n, span }
            }
            Tok::Word(w) if is_primitive_head(&w) || matches!(self.peek_at(1), Tok::Name(_)) => {
                return self.fail(format!("a type goes after the name: `let 'x': {} = …;`", w));
            }
            Tok::Word(w) => return self.fail(format!("a variable's name is quoted: `let '{}' := …;`", w)),
            other => return self.fail(format!("expected the variable's name, found {}", other.describe())),
        };
        if self.is_word("mut") || self.is_word("immut") {
            return self.fail(format!("`{}` goes before the name: `let {} '{}' := …;`", self.word_text(), self.word_text(), name.name));
        }
        let (ty, shadow) = self.let_type_and_marker("declaration")?;
        let value = self.expr_top()?;
        if self.is_op("=") {
            return self.fail("`=` gives a declaration its value once; equality is `==`");
        }
        self.expect_semi("the declaration")?;
        Ok(StmtKind::Let { mutability, name, ty, shadow, value, sealed })
    }

    fn stmt_kind(&mut self) -> Result<StmtKind> {
        let w = match self.peek().clone() {
            Tok::Word(w) => w,
            Tok::LBrace => {
                let b = self.block("the block")?;
                return Ok(StmtKind::Block(b));
            }
            Tok::Semi => return self.fail("expected a statement, found `;` — there are no empty statements"),
            Tok::Strict => return self.fail("`#strict` goes once, at the top of the file, before any item"),
            // A declaration starts with `let` (owner, 2026-10-02).
            Tok::Name(n) => {
                return match self.peek_at(1) {
                    Tok::Colon | Tok::Walrus | Tok::Dollar => self.fail(format!("a declaration starts with `let`: `let '{}' := …;`", n)),
                    Tok::Word(m) if m == "mut" || m == "immut" => self.fail_at(self.span(), format!("a declaration starts with `let`, and `{}` goes before the name: `let {} '{}' := …;`", m, m, n)),
                    _ => self.expr_stmt(),
                };
            }
            Tok::Op(o) if o == "!" => return self.fail("a declaration starts with `let`: `let !$ 'x' := …;`, `let !mut 'x' := …;`"),
            _ => return self.expr_stmt(),
        };
        match w.as_str() {
            "let" => {
                self.bump();
                self.declaration()
            }
            "var" => self.fail("`var` is gone: a declaration starts with `let` — `let 'x' := …;`"),
            "mut" | "immut" => self.fail(format!("a declaration starts with `let`: `let {} 'x' := …;`", w)),
            // Changing says so (owner, 2026-09-25): `change 'x' = …;`.
            "change" => {
                self.bump();
                let target = self.expr_top()?;
                match &target.kind {
                    ExprKind::Name(_) | ExprKind::Field { .. } | ExprKind::Index { .. } => {}
                    ExprKind::ConstPath { .. } => return self.fail_at(target.span, "a const is never changed: it is a value known at compile time"),
                    _ => return self.fail_at(target.span, "only a variable, a field or an element can be changed"),
                }
                if !self.eat_op("=") {
                    return self.fail(format!("expected `=` after what is changed, found {}", self.peek().describe()));
                }
                let value = self.expr_top()?;
                if self.is_op("=") {
                    return self.fail("`=` changes once; equality is `==`");
                }
                self.expect_semi("the change")?;
                Ok(StmtKind::Assign { target, value })
            }
            w if is_primitive_head(w) && matches!(self.peek_at(1), Tok::Name(_)) => self.fail(format!("a type goes after the name: `let 'x': {} = …;`", w)),
            "check" | "nocheck" | "NO_FUCKING_CHECKS" => {
                let tier_span = self.bump().span;
                let tier = match w.as_str() {
                    "check" => Tier::Check,
                    "nocheck" => Tier::Nocheck,
                    _ => Tier::NoChecks,
                };
                if self.is(&Tok::Semi) {
                    return self.fail_at(tier_span, format!("`{}` goes in front of a statement, covering its check sites: `{} change 'xs'['i'] = 1;`, `{} {{ … }}`", w, w, w));
                }
                if self.is_word("check") || self.is_word("nocheck") || self.is_word("NO_FUCKING_CHECKS") {
                    return self.fail(format!("one tier per statement: `{}` and `{}` cannot both cover it", w, self.word_text()));
                }
                if self.is_word("fn") {
                    return self.fail(format!("`{}` goes in front of a statement, not a `fn`: a function's safety is the sum of its statements", w));
                }
                if self.is_word("struct") || self.is_word("const") || self.is_word("module") || self.is_word("enum") {
                    return self.fail(format!("`{}` goes in front of a statement, not a `{}`", w, self.word_text()));
                }
                let stmt = self.stmt()?;
                Ok(StmtKind::Tier { tier, stmt: Box::new(stmt) })
            }
            "if" => {
                self.bump();
                self.if_rest()
            }
            "else" => self.fail("`else` without an `if` before it"),
            "loop" => {
                self.bump();
                if self.is(&Tok::Dot) {
                    self.bump();
                    let what = self.ident("a word")?;
                    return match what.name.as_str() {
                        "while" => self.fail_at(what.span, "`loop.while` became `while`: `while 'x' < 3 { … }`"),
                        "for" | "temp" | "perm" | "seq" | "range" => self.fail_at(what.span, "`loop.temp.for` / `loop.range` became `for`: `for 'x' in lend 'xs' { … }`, `for 'i' in range[0, 9] { … }`"),
                        _ => self.fail_at(what.span, "`loop` takes no chain: `loop { … }` runs until `break`; `while` and `for` are bare words"),
                    };
                }
                let body = self.block("`loop`")?;
                Ok(StmtKind::Loop(body))
            }
            "while" => {
                self.bump();
                let cond = self.expr(0)?;
                let body = self.block("`while`")?;
                Ok(StmtKind::While { cond, body })
            }
            "for" => {
                self.bump();
                self.for_rest()
            }
            "return" => {
                self.bump();
                if self.eat(&Tok::Semi) {
                    return Ok(StmtKind::Return(None));
                }
                let e = self.expr_top()?;
                self.expect_semi("the return value")?;
                self.returns_value = true;
                Ok(StmtKind::Return(Some(e)))
            }
            // `must` (owner, 2026-09-29): `must …;`, `must.fail …;`,
            // `must.fail.<reason> …;`, `must.stop …;`. A function called
            // `must` is still `must[…]`.
            "must" if !matches!(self.peek_at(1), Tok::LBracket) => {
                self.bump();
                let kind = if self.eat(&Tok::Dot) {
                    let w = self.ident("`fail` or `stop` after `must.`")?;
                    match w.name.as_str() {
                        "fail" => {
                            if self.eat(&Tok::Dot) {
                                MustKind::Fail(Some(self.ident("the reason after `must.fail.`")?))
                            } else {
                                MustKind::Fail(None)
                            }
                        }
                        "stop" => MustKind::Stop,
                        other => return self.fail_at(w.span, format!("`must.{}` is not one of the family: `must …` (what must hold), `must.fail …` (it must fail), `must.stop …` (a check must fire)", other)),
                    }
                } else {
                    MustKind::Hold
                };
                let expr = self.expr_top()?;
                self.expect_semi("the `must`")?;
                Ok(StmtKind::Must { kind, expr })
            }
            "fail" => {
                self.bump();
                let name = self.ident("the failure's name")?;
                let payload = if self.is(&Tok::LBracket) {
                    self.bump();
                    let e = self.expr(0)?;
                    self.expect(&Tok::RBracket, "`]` closing the payload")?;
                    Some(e)
                } else {
                    None
                };
                self.expect_semi("the failure")?;
                Ok(StmtKind::Fail { name, payload })
            }
            "break" => {
                self.bump();
                self.expect_semi("`break`")?;
                Ok(StmtKind::Break)
            }
            "continue" => {
                self.bump();
                self.expect_semi("`continue`")?;
                Ok(StmtKind::Continue)
            }
            "concur" => {
                self.bump();
                self.concur_rest()
            }
            "whichever" => {
                self.bump();
                self.whichever_rest()
            }
            "match" => {
                self.bump();
                self.match_rest()
            }
            // Spelled `giveS` since 2026-09-29, to keep it apart from `give`.
            "gives" if !matches!(self.peek_at(1), Tok::LBracket | Tok::Dot) => self.fail("`gives` is spelled `giveS`: `{ giveS 1; }` in a `#strict` file"),
            // `giveS` (owner, 2026-09-29: b) is a strict file's; elsewhere
            // a case's value stands alone. `giveS[…]` is a call.
            "giveS" if !matches!(self.peek_at(1), Tok::LBracket | Tok::Dot) => {
                let at = self.bump().span;
                if !self.strict {
                    return self.fail_at(at, "`giveS` belongs to `#strict` files; elsewhere a case's value stands alone: `{ 1 }`");
                }
                let value = self.expr_top()?;
                self.expect_semi("the case's value")?;
                Ok(StmtKind::Gives { value, bare: false })
            }
            "inline" => {
                self.bump();
                self.inline_rest()
            }
            "asm" => self.fail("an inline block is `inline.ASM { … }` (the language's name in capitals, like `MAIN`), under `nocheck`"),
            "otherwise" => self.fail("`otherwise` follows a call that can fail: `let 'x' := f[] otherwise 0;`"),
            "fn" | "struct" | "interface" | "op" => self.fail(format!("a `{}` sits at the top level of a file or a module, not inside a body", w)),
            "enum" => self.fail("an `enum` sits at the top level of a file or a module, not inside a body"),
            "const" => self.fail("a `const` sits at the top level: inside a body a declaration is `let 'x' := …;`, and Precog finds the constants itself"),
            "module" => self.fail("a `module` sits at the top level of a file, not inside a body"),
            "pub" => self.fail("`pub` goes at the top level, on a `fn`, `struct`, `enum` or `const`"),
            "extern" => self.fail("an `extern` declaration sits at the top level, not inside a body"),
            "MAIN" => self.fail("`MAIN` cannot be nested or called: it is the program's one block"),
            "lang" if matches!(self.peek_at(1), Tok::Colon) => self.fail("`lang:` was removed: keywords are bare — `let 'x' := 1;`"),
            _ => self.expr_stmt(),
        }
    }

    fn if_rest(&mut self) -> Result<StmtKind> {
        let cond = self.expr(0)?;
        let then = self.block("`if`")?;
        let else_ = if self.is_word("else") {
            self.bump();
            if self.is(&Tok::Dot) {
                return self.fail("`else.if` became `else if`: two bare words");
            }
            if self.is_word("if") {
                let start = self.span();
                self.bump();
                let kind = self.if_rest()?;
                Some(ElseArm::If(Box::new(Stmt { kind, span: start.to(self.prev_span()) })))
            } else {
                Some(ElseArm::Block(self.block("`else`")?))
            }
        } else {
            None
        };
        Ok(StmtKind::If { cond, then, else_ })
    }

    fn for_rest(&mut self) -> Result<StmtKind> {
        let mut seq = false;
        let mut para = false;
        let mut perm = false;
        // `!c`, "no cheat", first: the strongest modifier (owner, 2026-10-01).
        let nocheat = self.is_op("!") && matches!(self.peek_at(1), Tok::Word(w) if w == "c");
        if nocheat {
            self.bump();
            self.bump();
        }
        // `seq` keeps a loop in order, `para` makes it parallel (owner,
        // 2026-09-30); never both. CCA runs every loop in order so far.
        if self.eat_word("seq") {
            seq = true;
        } else if self.eat_word("para") {
            para = true;
        }
        if self.is_word("seq") || self.is_word("para") {
            return self.fail("a `for` is `seq` or `para`, not both");
        }
        if self.is_op("!") && matches!(self.peek_at(1), Tok::Word(w) if w == "c") {
            return self.fail("`!c` comes first, the strongest modifier first: `for !c seq …`, `for !c para …`");
        }
        if self.eat_word("perm") {
            perm = true;
        } else if self.eat_word("temp") {
            perm = false;
        } else if self.strict && !self.is_word("seq") && !self.is_word("para") {
            return self.fail("a `for` says whether its variable outlives it: `for temp 'i' in …` or `for perm 'i' in …` — a `#strict` file writes it out");
        }
        if self.is_word("seq") || self.is_word("para") {
            return self.fail("expected a quoted loop variable: `seq` or `para` goes first, then `temp` / `perm`, then the name — `for seq perm 'i' in …`");
        }
        let name = match self.peek().clone() {
            Tok::Name(n) => {
                let span = self.bump().span;
                QName { name: n, span }
            }
            Tok::Word(w) => return self.fail(format!("expected a quoted loop variable, found `{}`: `for '{}' in …`", w, w)),
            other => return self.fail(format!("expected a quoted loop variable, found {}", other.describe())),
        };
        let ty = if self.eat(&Tok::Colon) { Some(self.type_()?) } else { None };
        // `for 'i', 'x' in …`: the position first, then the item, as in
        // `'xs'['i']` (owner, 2026-09-29).
        let (pos, name) = if self.eat(&Tok::Comma) {
            let item = match self.peek().clone() {
                Tok::Name(n) => {
                    let span = self.bump().span;
                    QName { name: n, span }
                }
                other => return self.fail(format!("expected the item's quoted name after the position's: `for 'i', 'x' in …`, found {}", other.describe())),
            };
            (Some(name), item)
        } else {
            (None, name)
        };
        if self.is_word("temp") || self.is_word("perm") {
            return self.fail(format!("expected `in`; `{}` goes before the name — `for {} '{}' in …`", self.word_text(), self.word_text(), name.name));
        }
        if !self.eat_word("in") {
            return self.fail(format!("expected `in` after the loop variable, found {}", self.peek().describe()));
        }
        let iter = if self.is_word("range") {
            let rstart = self.span();
            self.bump();
            if self.is(&Tok::Dot) {
                return self.fail("`range` builds its bounds from values, and the counter's type goes on its name: `for 'i': int32 in range[0, 3]`");
            }
            self.expect(&Tok::LBracket, "`[` after `range`")?;
            if self.is(&Tok::Walrus) || self.is_op("=") {
                return self.fail("the bounds of a `range` are plain values: `range[0, 3]` — the `:=` / `=` markers are gone (they would say the same thing twice)");
            }
            // A strict file labels each bound and always writes the step
            // (owner, 2026-09-27): `range[from 2, to 8, by 2]`.
            let (start, stop, step) = if self.strict {
                let labelled = |this: &mut Self, word: &str| -> Result<Expr> {
                    if this.range_label() != Some(word) {
                        return this.fail(format!("each bound says what it is: `range[from 2, to 8, by 2]` — `{}` goes here, in a `#strict` file", word));
                    }
                    this.bump();
                    this.expr(0)
                };
                let start = labelled(self, "from")?;
                self.expect(&Tok::Comma, "`,` between the range's start and stop")?;
                let stop = labelled(self, "to")?;
                if !self.eat(&Tok::Comma) {
                    return self.fail("a range writes its step too, even 1: `range[from 2, to 8, by 1]` — a `#strict` file writes it out");
                }
                let step = labelled(self, "by")?;
                (start, stop, Some(step))
            } else {
                let unlabelled = |this: &mut Self| -> Result<Expr> {
                    if let Some(w) = this.range_label() {
                        return this.fail(format!("label a range in a `#strict` file; elsewhere its bounds are bare — `range[2, 8, 2]`, not `{} …`", w));
                    }
                    this.expr(0)
                };
                let start = unlabelled(self)?;
                self.expect(&Tok::Comma, "`,` between the range's start and stop")?;
                let stop = unlabelled(self)?;
                let step = if self.eat(&Tok::Comma) { Some(unlabelled(self)?) } else { None };
                (start, stop, step)
            };
            self.expect(&Tok::RBracket, "`]` closing the range")?;
            let _ = rstart;
            ForIter::Range { start, stop, step }
        } else {
            ForIter::Over(self.expr(0)?)
        };
        if let (Some(p), ForIter::Range { .. }) = (&pos, &iter) {
            return self.fail_at(p.span, format!("a range's counter already is the position: `for '{}' in range[…]`, one name", p.name));
        }
        let body = self.block("`for`")?;
        Ok(StmtKind::For { nocheat, seq, para, perm, name, pos, ty, iter, body })
    }

    /// `from`, `to` or `by` heading a range bound; `to.int32[…]` is a
    /// conversion, not a label.
    fn range_label(&self) -> Option<&'static str> {
        if self.is_word("from") {
            Some("from")
        } else if self.is_word("by") {
            Some("by")
        } else if self.is_word("to") && !matches!(self.peek_at(1), Tok::Dot) {
            Some("to")
        } else {
            None
        }
    }

    fn captures(&mut self) -> Result<Vec<Capture>> {
        // after `with`
        self.expect(&Tok::LBracket, "`[` after `with`")?;
        let mut caps = Vec::new();
        while !self.is(&Tok::RBracket) {
            let start = self.span();
            let (verb, amp) = match self.peek().clone() {
                Tok::Word(w) if w == "give" => {
                    self.bump();
                    (Verb::Give, false)
                }
                Tok::Word(w) if w == "copy" => {
                    self.bump();
                    (Verb::Copy, false)
                }
                Tok::Word(w) if w == "lend" => {
                    self.bump();
                    if self.eat(&Tok::Dot) {
                        let m = self.ident("`mut`")?;
                        if m.name != "mut" {
                            return self.fail_at(m.span, "`lend` or `lend.mut`");
                        }
                        (Verb::LendMut, false)
                    } else {
                        (Verb::Lend, false)
                    }
                }
                Tok::Op(o) if o == "&" => {
                    self.bump();
                    if self.eat_word("mut") {
                        (Verb::LendMut, true)
                    } else {
                        (Verb::Lend, true)
                    }
                }
                Tok::Name(n) => return self.fail(format!("nothing is captured silently: say how '{}' comes in — `lend '{}'`, `copy '{}'` or `give '{}'`", n, n, n, n)),
                other => return self.fail(format!("expected a capture (`lend 'x'`, `copy 'x'`, `give 'x'`), found {}", other.describe())),
            };
            let name = self.qname("the captured variable")?;
            caps.push(Capture { verb, amp, name, span: start.to(self.prev_span()) });
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(&Tok::RBracket, "`]` closing the captures")?;
        Ok(caps)
    }

    fn concur_rest(&mut self) -> Result<StmtKind> {
        let mut detached = false;
        if self.eat(&Tok::Dot) {
            let w = self.ident("`detached` after `concur.`")?;
            if w.name != "detached" {
                return self.fail_at(w.span, format!("`concur.{}` does not exist: `concur` (structured) or `concur.detached` (lives as long as the program)", w.name));
            }
            detached = true;
        }
        let captures = if self.eat_word("with") { self.captures()? } else { Vec::new() };
        let body = if self.is(&Tok::LBrace) {
            ConcurBody::Block(self.block("the task")?)
        } else {
            let e = self.expr_top()?;
            ConcurBody::Call(e)
        };
        let otherwise = if self.is_word("otherwise") {
            self.bump();
            Some(self.otherwise_rest()?)
        } else {
            None
        };
        if matches!(body, ConcurBody::Call(_)) {
            self.expect_semi("the task's call")?;
        } else {
            self.eat(&Tok::Semi);
        }
        Ok(StmtKind::Concur { detached, captures, body, otherwise })
    }

    fn whichever_rest(&mut self) -> Result<StmtKind> {
        let open = self.expect(&Tok::LBrace, "`{` after `whichever`")?;
        let mut arms = Vec::new();
        while !self.is(&Tok::RBrace) {
            if self.at_eof() {
                return self.fail_at(open, "`whichever` is never closed");
            }
            let name = self.qname("the arm's value name")?;
            if !self.eat_word("from") {
                return self.fail(format!("expected `from` after '{}': each arm is `'value' from lend 'channel' {{ … }}`", name.name));
            }
            let channel = self.expr(0)?;
            let body = self.block("the arm")?;
            arms.push(WhicheverArm { name, channel, body });
        }
        self.bump();
        if arms.is_empty() {
            return self.fail_at(open.to(self.prev_span()), "`whichever` needs at least one arm: `'v' from lend 'c' { … }`");
        }
        let otherwise = if self.is_word("otherwise") {
            self.bump();
            Some(self.otherwise_rest()?)
        } else {
            None
        };
        self.eat(&Tok::Semi);
        Ok(StmtKind::Whichever { arms, otherwise })
    }

    fn match_rest(&mut self) -> Result<StmtKind> {
        let (value, arms) = self.match_parts()?;
        Ok(StmtKind::Match { value, arms })
    }

    fn match_parts(&mut self) -> Result<(Expr, Vec<MatchArm>)> {
        let value = self.expr(0)?;
        let open = self.expect(&Tok::LBrace, "`{` after the matched value")?;
        let mut arms = Vec::new();
        while !self.is(&Tok::RBrace) {
            if self.at_eof() {
                return self.fail_at(open, "`match` is never closed");
            }
            let case = match self.peek().clone() {
                Tok::Word(_) => self.ident("a case name")?,
                Tok::Name(n) => return self.fail(format!("an arm starts with the case's bare name, then a quoted name for its data: `claimed '{}' {{ … }}`", n)),
                other => return self.fail(format!("expected a case name, found {}", other.describe())),
            };
            let bind = if let Tok::Name(_) = self.peek() { Some(self.qname("a name")?) } else { None };
            let body = self.block("the arm")?;
            arms.push(MatchArm { case, bind, body });
        }
        self.bump();
        if arms.is_empty() {
            return self.fail_at(open.to(self.prev_span()), "`match` needs an arm per case");
        }
        Ok((value, arms))
    }

    fn inline_rest(&mut self) -> Result<StmtKind> {
        self.expect(&Tok::Dot, "`.ASM` after `inline`")?;
        let lang = self.ident("the language after `inline.`")?;
        match lang.name.as_str() {
            // `S` is another name for it (owner, 2026-10-02), as in `.s` files.
            "ASM" | "S" => {}
            "asm" => return self.fail_at(lang.span, "the inline language's name is written in capitals, like `MAIN`, so a block is easy to spot: `inline.ASM`"),
            "s" => return self.fail_at(lang.span, "the inline language's name is written in capitals, like `MAIN`, so a block is easy to spot: `inline.S` (or `inline.ASM`)"),
            "C" | "c" => return self.fail_at(lang.span, "only `inline.ASM` (also `inline.S`) exists so far; `inline.C` is not decided"),
            other => return self.fail_at(lang.span, format!("only `inline.ASM` (also `inline.S`) exists; `inline.{}` is not a language the compiler can inline", other)),
        }
        let text = match self.peek().clone() {
            Tok::Raw(t) => {
                self.bump();
                t
            }
            Tok::LBracket => return self.fail("`inline.ASM` holds raw assembly in braces, not a list of strings: `inline.ASM { mov x16, #4 }`"),
            other => return self.fail(format!("expected `{{` holding the assembly, found {}", other.describe())),
        };
        let mut bindings = Vec::new();
        if self.eat_word("with") {
            self.expect(&Tok::LBracket, "`[` after `with`")?;
            while !self.is(&Tok::RBracket) {
                match self.peek().clone() {
                    Tok::Word(w) if w == "clobber" => {
                        self.bump();
                        let reg = self.ident("a register after `clobber`")?;
                        bindings.push(AsmBinding::Clobber(reg));
                    }
                    Tok::Word(w) if w == "in" || w == "out" || w == "inout" => {
                        return self.fail(format!("there is no `{}` word: the arrow is the direction — `'v' -> x0` copies in, `'v' <- x0` stores out, `'v' <-> x0` both", w));
                    }
                    // A lend satisfies a pointer, as at an extern call
                    // (owner, 2026-09-30: a): `lend 'text' -> x1`.
                    Tok::Word(w) if w == "lend" => {
                        self.bump();
                        let mutable = if self.eat(&Tok::Dot) {
                            let m = self.ident("`mut`")?;
                            if m.name != "mut" {
                                return self.fail_at(m.span, "`lend` or `lend.mut`");
                            }
                            true
                        } else {
                            false
                        };
                        let name = self.qname("the variable lent")?;
                        match self.peek().clone() {
                            Tok::Op(o) if o == "->" => {}
                            Tok::Op(o) if o == "<-" || o == "<->" => {
                                return self.fail(format!("a lend goes into a register only: `{} '{}' -> x1` puts the address of '{}' there", if mutable { "lend.mut" } else { "lend" }, name.name, name.name));
                            }
                            other => return self.fail(format!("expected `->` after `lend '{}'`: its address goes into a register, found {}", name.name, other.describe())),
                        }
                        self.bump();
                        let reg = self.ident("a register")?;
                        bindings.push(AsmBinding::Lend { name, mutable, reg });
                    }
                    Tok::Name(_) => {
                        let name = self.qname("a variable")?;
                        let dir = match self.peek().clone() {
                            Tok::Op(o) if o == "->" => AsmDir::In,
                            Tok::Op(o) if o == "<-" => AsmDir::Out,
                            Tok::Op(o) if o == "<->" => AsmDir::InOut,
                            other => return self.fail(format!("expected an arrow after '{}' (`->` in, `<-` out, `<->` both), found {}", name.name, other.describe())),
                        };
                        self.bump();
                        let reg = self.ident("a register")?;
                        bindings.push(AsmBinding::Reg { name, dir, reg });
                    }
                    other => return self.fail(format!("expected a binding (`'v' -> x0`, `'n' <- x0`, `lend 'text' -> x1`, `clobber x16`), found {}", other.describe())),
                }
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
            self.expect(&Tok::RBracket, "`]` closing the bindings")?;
        }
        self.expect_semi("the inline block")?;
        Ok(StmtKind::InlineAsm { text, bindings })
    }

    /// An expression statement or an assignment.
    fn expr_stmt(&mut self) -> Result<StmtKind> {
        let e = self.expr_top()?;
        // A value alone before the `}` is its block's value — a `match`
        // case's or an `otherwise` arm's (owner, 2026-09-29: a).
        if self.is(&Tok::RBrace) {
            if self.strict {
                return self.fail_at(e.span, "a case says its value with `giveS`: `{ giveS 1; }` — a `#strict` file writes it out");
            }
            return Ok(StmtKind::Gives { value: e, bare: true });
        }
        if self.is(&Tok::Colon) {
            if let ExprKind::Name(n) = &e.kind {
                return self.fail(format!("a declaration is the name, its type, then `=`: `let '{}': int32 = …;`", n.name));
            }
        }
        if self.is_op("=") {
            let spelled = match &e.kind {
                ExprKind::Name(n) => format!("'{}'", n.name),
                _ => "'x'".to_string(),
            };
            return self.fail_at(e.span, format!("changing a variable says so: `change {} = …;`", spelled));
        }
        if self.is(&Tok::Walrus) {
            return self.fail("a declaration is a name alone: `let 'x' := …;`");
        }
        match &e.kind {
            ExprKind::Call { .. } | ExprKind::CallValue { .. } | ExprKind::Conv { .. } => {}
            ExprKind::Path(p) => return self.fail_at(e.span, format!("`{}` alone is not a statement: a call needs `[…]` — `{}[…];`", p.spelled(), p.spelled())),
            // One value in brackets: a list read, or a call of a function
            // held in a variable — the checker knows which.
            ExprKind::Index { .. } => {}
            _ => {
                if self.is(&Tok::Semi) {
                    return self.fail_at(e.span, "a value alone is not a statement: assign it, or call something with it");
                }
                return self.fail(format!("expected `;` after the statement, found {}", self.peek().describe()));
            }
        }
        if !self.is(&Tok::Semi) {
            return self.fail(format!("expected `;` after the call, found {}", self.peek().describe()));
        }
        self.bump();
        Ok(StmtKind::Expr(e))
    }

    // ------------------------------------------------------------ expressions

    /// A whole value: an expression, optionally followed by `otherwise` when
    /// it is a call that can fail.
    fn expr_top(&mut self) -> Result<Expr> {
        let mut e = self.expr(0)?;
        if self.is_word("otherwise") {
            let ow_span = self.span();
            self.bump();
            let ow = self.otherwise_rest()?;
            // `'f'[2] otherwise …` is a call: a list read never fails.
            if let ExprKind::Index { .. } = e.kind {
                let ExprKind::Index { base, index } = std::mem::replace(&mut e.kind, ExprKind::Bool(false)) else { unreachable!() };
                e.kind = ExprKind::CallValue { callee: base, args: vec![Arg::Positional(*index)], otherwise: None };
            }
            match &mut e.kind {
                ExprKind::Call { otherwise, .. } | ExprKind::CallValue { otherwise, .. } | ExprKind::Conv { otherwise, .. } => {
                    if otherwise.is_some() {
                        return self.fail_at(ow_span, "a call takes one `otherwise`");
                    }
                    *otherwise = Some(Box::new(ow));
                    e.span = e.span.to(self.prev_span());
                }
                _ => {
                    return self.fail_at(e.span, "a call that can fail must be the whole value: put it in its own declaration, then use the result — `otherwise` cannot sit inside a larger expression");
                }
            }
        }
        Ok(e)
    }

    fn otherwise_rest(&mut self) -> Result<Otherwise> {
        match self.peek().clone() {
            Tok::Semi | Tok::RBrace | Tok::Eof => self.fail("`otherwise` needs what to do: a value, `return …`, `fail`, `break`, `continue`, or `{ reason { … } … }` arms"),
            Tok::Word(w) if w == "fail" => {
                let span = self.span();
                self.bump();
                if let Tok::Word(_) = self.peek() {
                    let name = self.ident("the failure's name")?;
                    let payload = if self.is(&Tok::LBracket) {
                        self.bump();
                        let e = self.expr(0)?;
                        self.expect(&Tok::RBracket, "`]`")?;
                        Some(e)
                    } else {
                        None
                    };
                    let s = Stmt { kind: StmtKind::Fail { name, payload }, span: span.to(self.prev_span()) };
                    Ok(Otherwise::Leave(Box::new(s)))
                } else {
                    Ok(Otherwise::PassUp(span))
                }
            }
            Tok::Word(w) if w == "return" => {
                let span = self.span();
                self.bump();
                let value = if self.is(&Tok::Semi) || self.is(&Tok::RBrace) { None } else { Some(self.expr_top()?) };
                let s = Stmt { kind: StmtKind::Return(value), span: span.to(self.prev_span()) };
                Ok(Otherwise::Leave(Box::new(s)))
            }
            Tok::Word(w) if w == "break" => {
                let span = self.bump().span;
                Ok(Otherwise::Leave(Box::new(Stmt { kind: StmtKind::Break, span })))
            }
            Tok::Word(w) if w == "continue" => {
                let span = self.bump().span;
                Ok(Otherwise::Leave(Box::new(Stmt { kind: StmtKind::Continue, span })))
            }
            Tok::LBrace => {
                let open = self.bump().span;
                let mut arms = Vec::new();
                while !self.is(&Tok::RBrace) {
                    if self.at_eof() {
                        return self.fail_at(open, "the `otherwise` arms are never closed");
                    }
                    let name = match self.peek().clone() {
                        Tok::Word(_) => self.ident("a failure name")?,
                        Tok::Name(n) => return self.fail(format!("an arm starts with the reason's bare name, then a quoted name for its payload: `bad_item '{}' {{ … }}`", n)),
                        other => return self.fail(format!("expected a failure name, found {}", other.describe())),
                    };
                    let bind = if let Tok::Name(_) = self.peek() { Some(self.qname("a name")?) } else { None };
                    let body = self.block("the arm")?;
                    arms.push(FailArm { name, bind, body });
                }
                let close = self.bump().span;
                if arms.is_empty() {
                    return self.fail_at(open.to(close), "`otherwise { }` has no arms: one arm per reason, `reason { … }`");
                }
                Ok(Otherwise::Arms(arms, open.to(close)))
            }
            _ => Ok(Otherwise::Value(self.expr(0)?)),
        }
    }

    /// Binding powers. `None` for a token that is not a binary operator here.
    /// The operator after `op`: any a value can be, whatever follows it.
    /// The rounding after `div` or `mod`, the word already taken: `.floor`,
    /// `.ceil`, `.trunc` or `.near`, or none — which the checker refuses,
    /// naming the one meant.
    fn rounding(&mut self, w: &str) -> Result<Round> {
        if !self.is(&Tok::Dot) {
            return Ok(Round::Bare);
        }
        self.bump();
        let what = self.ident("`floor`, `ceil`, `trunc` or `near`")?;
        match Round::from_word(&what.name) {
            Some(r) => Ok(r),
            None => self.fail_at(what.span, format!("no `{w}.{}`: a division rounds as `{w}.floor`, `{w}.ceil`, `{w}.trunc` or `{w}.near`", what.name)),
        }
    }

    fn op_as_value(&mut self) -> Result<(BinOp, String)> {
        let at = self.span();
        let op = match self.peek().clone() {
            Tok::Op(o) => {
                self.bump();
                match o.as_str() {
                    "+" => BinOp::Add,
                    "-" => BinOp::Sub,
                    "/" => return self.fail_at(at, "division is `div.floor`, `div.ceil`, `div.trunc` or `div.near`: `op div.floor`"),
                    "++" => BinOp::Join,
                    "==" => BinOp::Eq,
                    "!==" => BinOp::Ne,
                    "<" => BinOp::Lt,
                    ">" => BinOp::Gt,
                    "</==" => BinOp::Le,
                    ">/==" => BinOp::Ge,
                    "<==" => return self.fail_at(at, "less-or-equal is `</==`: `op </==`"),
                    ">==" => return self.fail_at(at, "greater-or-equal is `>/==`: `op >/==`"),
                    "+~" => BinOp::WrapAdd,
                    "-~" => BinOp::WrapSub,
                    "=" => return self.fail_at(at, "`=` is not an operator: equality is `op ==`"),
                    other => BinOp::Custom(other.to_string()),
                }
            }
            Tok::Word(w) => {
                let wraps = self.is_op_at(1, "~") && self.adjacent(0, 1);
                self.bump();
                match w.as_str() {
                    "x" | "xx" if wraps => {
                        self.bump();
                        if w == "x" { BinOp::WrapMul } else { BinOp::WrapPow }
                    }
                    "x" => BinOp::Mul,
                    "xx" => BinOp::Pow,
                    "mod" | "div" => {
                        let r = self.rounding(&w)?;
                        if w == "div" { BinOp::Div(r) } else { BinOp::Mod(r) }
                    }
                    "and" | "or" if self.eat(&Tok::Dot) => {
                        let what = self.ident("`then`, `else` or `bit`")?;
                        match (w.as_str(), what.name.as_str()) {
                            ("and", "then") => BinOp::AndThen,
                            ("or", "else") => BinOp::OrElse,
                            ("and", "bit") => BinOp::AndBit,
                            ("or", "bit") => BinOp::OrBit,
                            _ => return self.fail_at(what.span, format!("`{}.{}` is not an operator", w, what.name)),
                        }
                    }
                    "and" => BinOp::And,
                    "or" => BinOp::Or,
                    "xor" => BinOp::Xor,
                    "shl" => BinOp::Shl,
                    "shr" => BinOp::Shr,
                    other if is_keyword(other) => return self.fail_at(at, format!("`{}` is a keyword, not an operator: `op` takes one, as in `op +`", other)),
                    other => BinOp::Custom(other.to_string()),
                }
            }
            other => return self.fail(format!("`op` as a value takes an operator: `op +`, `op x`, `op <`, found {}", other.describe())),
        };
        let spelled = op.spelled().to_string();
        Ok((op, spelled))
    }

    fn binop(&self) -> Option<(BinOp, u8, u8, Span)> {
        let span = self.span();
        let (op, l, r) = match self.peek() {
            Tok::Op(o) => match o.as_str() {
                "+" => (BinOp::Add, 6, 7),
                "-" => (BinOp::Sub, 6, 7),
                "++" => (BinOp::Join, 5, 6),
                "==" => (BinOp::Eq, 4, 5),
                "!==" => (BinOp::Ne, 4, 5),
                "<" => (BinOp::Lt, 4, 5),
                ">" => (BinOp::Gt, 4, 5),
                "</==" => (BinOp::Le, 4, 5),
                ">/==" => (BinOp::Ge, 4, 5),
                "+~" => (BinOp::WrapAdd, 6, 7),
                "-~" => (BinOp::WrapSub, 6, 7),
                "=" => return None,
                other => (BinOp::Custom(other.to_string()), 0, 1),
            },
            Tok::Word(w) => {
                let wraps = self.is_op_at(1, "~") && self.adjacent(0, 1);
                match w.as_str() {
                    "x" if wraps => (BinOp::WrapMul, 7, 8),
                    "xx" if wraps => (BinOp::WrapPow, 9, 9),
                    "x" => (BinOp::Mul, 7, 8),
                    "xx" => (BinOp::Pow, 9, 9),
                    // The rounding word is read when the operator is taken.
                    "mod" => (BinOp::Mod(Round::Bare), 7, 8),
                    "div" => (BinOp::Div(Round::Bare), 7, 8),
                    "and" if matches!(self.peek_at(1), Tok::Dot) && self.is_word_at(2, "then") => (BinOp::AndThen, 2, 3),
                    "or" if matches!(self.peek_at(1), Tok::Dot) && self.is_word_at(2, "else") => (BinOp::OrElse, 1, 2),
                    "and" if matches!(self.peek_at(1), Tok::Dot) => (BinOp::AndBit, 0, 1),
                    "or" if matches!(self.peek_at(1), Tok::Dot) => (BinOp::OrBit, 0, 1),
                    "and" => (BinOp::And, 2, 3),
                    "or" => (BinOp::Or, 1, 2),
                    "xor" => (BinOp::Xor, 0, 1),
                    "shl" => (BinOp::Shl, 0, 1),
                    "shr" => (BinOp::Shr, 0, 1),
                    w if is_keyword(w) || DURATION_UNITS.contains(&w) => return None,
                    // a word operator needs a right operand; a unit word before `]` is not one
                    _ if matches!(self.peek_at(1), Tok::RBracket | Tok::RParen | Tok::Comma | Tok::Semi | Tok::LBrace | Tok::RBrace | Tok::Eof) => return None,
                    other => (BinOp::Custom(other.to_string()), 0, 1),
                }
            }
            _ => return None,
        };
        Some((op, l, r, span))
    }

    fn expr(&mut self, min_bp: u8) -> Result<Expr> {
        let mut lhs = self.prefix()?;
        loop {
            let Some((mut op, l_bp, r_bp, op_span)) = self.binop() else { break };
            if l_bp < min_bp {
                break;
            }
            // consume the operator token(s)
            self.bump();
            match op {
                BinOp::WrapMul | BinOp::WrapPow => {
                    self.bump();
                }
                BinOp::AndThen | BinOp::OrElse => {
                    self.bump(); // .
                    self.bump(); // then / else
                }
                BinOp::Div(_) | BinOp::Mod(_) => {
                    let w = if op.is_div() { "div" } else { "mod" };
                    let r = self.rounding(w)?;
                    op = if op.is_div() { BinOp::Div(r) } else { BinOp::Mod(r) };
                }
                BinOp::AndBit | BinOp::OrBit => {
                    self.bump(); // .
                    let b = self.ident("`bit`")?;
                    if b.name != "bit" {
                        return self.fail_at(b.span, format!("`{}.{}` is not an operator: the bitwise forms are `and.bit` and `or.bit`", op.spelled().trim_end_matches(".bit"), b.name));
                    }
                }
                _ => {}
            }
            let op_span = op_span.to(self.prev_span());
            if op == BinOp::Join && self.is(&Tok::Semi) {
                return self.fail_at(op_span, "`++` joins text; there is no increment — write `change 'i' = 'i' + 1;`");
            }
            // The near-misses the language refuses by name.
            if let BinOp::Custom(c) = &op {
                let hint = match c.as_str() {
                    "!=" => Some("not-equal is `!==`"),
                    ">=" | ">==" => Some("greater-or-equal is `>/==`, the slash read as \"or\""),
                    "<=" | "<==" => Some("less-or-equal is `</==`, the slash read as \"or\""),
                    "/" => Some("division is `div.floor`, `div.ceil`, `div.trunc` or `div.near`: `'a' div.floor 'b'`"),
                    "--" => Some("there is no decrement: write `change 'i' = 'i' - 1;`"),
                    "+=" | "-=" => Some("there is no compound assignment: write `change 'i' = 'i' + 1;`"),
                    _ => None,
                };
                if let Some(h) = hint {
                    return self.fail_at(op_span, format!("`{}` is not an operator here: {}", c, h));
                }
            }
            let rhs = self.expr(r_bp)?;
            let span = lhs.span.to(rhs.span);
            self.check_precedence(&op, op_span, &lhs, &rhs)?;
            lhs = Expr { kind: ExprKind::Binary { op, op_span, lhs: Box::new(lhs), rhs: Box::new(rhs) }, span };
        }
        Ok(lhs)
    }

    /// Where mathematics has no settled reading, parentheses are required.
    fn check_precedence(&self, op: &BinOp, op_span: Span, lhs: &Expr, rhs: &Expr) -> Result<()> {
        let bare_op = |e: &Expr| -> Option<BinOp> {
            match &e.kind {
                ExprKind::Binary { op, .. } => Some(op.clone()),
                _ => None,
            }
        };
        let bare_unary = |e: &Expr| matches!(e.kind, ExprKind::Unary { .. });
        let nofix = |o: &BinOp| o.is_bitwise() || matches!(o, BinOp::Custom(_));
        for side in [lhs, rhs] {
            let inner = bare_op(side);
            if nofix(op) && (inner.is_some() || bare_unary(side)) {
                let kind = if op.is_bitwise() { "a bitwise operator has no precedence" } else { "a custom operator has no precedence" };
                return self.fail_at(op_span, format!("`{}` next to another operator: {} — parenthesise one side", op.spelled(), kind));
            }
            if let Some(inner) = &inner {
                if nofix(inner) {
                    let kind = if inner.is_bitwise() { "a bitwise operator has no precedence" } else { "a custom operator has no precedence" };
                    return self.fail_at(op_span, format!("`{}` next to `{}`: {} — parenthesise", inner.spelled(), op.spelled(), kind));
                }
                if op.is_mod() || inner.is_mod() {
                    let m = if op.is_mod() { op.spelled() } else { inner.spelled() };
                    return self.fail_at(op_span, format!("`{}` next to `{}`: mathematics has no settled reading for `{}` beside another operator — parenthesise (`('i' {} 2) == 0`)", inner.spelled(), op.spelled(), m, if m.contains('.') { m.clone() } else { format!("{}.floor", m) }));
                }
                if (op.is_div() && inner.is_multiplicative()) || (op.is_multiplicative() && inner.is_div()) {
                    return self.fail_at(op_span, format!("`{}` next to `{}`: mathematics has no settled reading here (`6 div.floor 2 x 3`) — parenthesise one side", inner.spelled(), op.spelled()));
                }
                if op.is_comparison() && inner.is_comparison() {
                    return self.fail_at(op_span, format!("`{}` next to `{}`: comparisons do not chain — write `('a' < 'b') and ('b' < 'c')`", inner.spelled(), op.spelled()));
                }
            }
            if op.is_comparison() && matches!(side.kind, ExprKind::Unary { op: UnOp::Not, .. }) {
                return self.fail_at(op_span, format!("`not` next to `{}`: mathematics has no settled reading — parenthesise (`not ('a' == 'b')`)", op.spelled()));
            }
        }
        Ok(())
    }

    fn prefix(&mut self) -> Result<Expr> {
        let start = self.span();
        match self.peek().clone() {
            // `*0.1`: exactly this decimal, a `deci` (owner, 2026-10-02).
            Tok::Op(o) if o == "*" && matches!(self.peek_at(1), Tok::Num { .. }) => {
                self.bump();
                let Tok::Num { text, bin } = self.peek().clone() else { unreachable!() };
                let at = self.bump().span;
                if !bin {
                    return self.fail_at(start.to(at), format!("`{}` is exact already: an integer is held as written, so `*` adds nothing — write `{}`", text, text));
                }
                let span = start.to(at);
                self.postfix_on(Expr { kind: ExprKind::Exact(text.replace('_', "")), span })
            }
            Tok::Op(o) if o == "-" => {
                self.bump();
                let operand = self.expr(9)?;
                let span = start.to(operand.span);
                Ok(Expr { kind: ExprKind::Unary { op: UnOp::Neg, expr: Box::new(operand) }, span })
            }
            Tok::Op(o) if o == "&" => {
                self.bump();
                let (verb, mut_word) = if self.is_word("mut") {
                    self.bump();
                    (Verb::LendMut, true)
                } else {
                    (Verb::Lend, false)
                };
                match self.peek() {
                    Tok::Name(_) => {}
                    Tok::Word(w) if !mut_word && w.starts_with("mut") => return self.fail(format!("`&` lends a variable: `&'x'` or `&mut 'x'`; `&{}` is neither", w)),
                    _ => return self.fail("`&` lends a variable: write `&'x'` (`lend 'x'`) or `&mut 'x'` (`lend.mut 'x'`)"),
                }
                let operand = self.postfix()?;
                let span = start.to(operand.span);
                Ok(Expr { kind: ExprKind::Verb { verb, amp: true, expr: Box::new(operand) }, span })
            }
            Tok::Op(o) => self.fail(format!("expected a value, found `{}`: an operator sits between two values", o)),
            Tok::Word(w) => match w.as_str() {
                "not" => {
                    self.bump();
                    if self.eat(&Tok::Dot) {
                        let b = self.ident("`bit`")?;
                        if b.name != "bit" {
                            return self.fail_at(b.span, format!("`not.{}` is not an operator: `not` or `not.bit`", b.name));
                        }
                        let operand = self.expr(9)?;
                        let span = start.to(operand.span);
                        return Ok(Expr { kind: ExprKind::Unary { op: UnOp::NotBit, expr: Box::new(operand) }, span });
                    }
                    let operand = self.expr(4)?;
                    if let ExprKind::Binary { op, op_span, .. } = &operand.kind {
                        if op.is_comparison() {
                            return self.fail_at(*op_span, format!("`not` next to `{}`: mathematics has no settled reading — parenthesise (`not ('a' == 'b')`)", op.spelled()));
                        }
                    }
                    let span = start.to(operand.span);
                    Ok(Expr { kind: ExprKind::Unary { op: UnOp::Not, expr: Box::new(operand) }, span })
                }
                "give" | "copy" | "lend" => {
                    self.bump();
                    let verb = match w.as_str() {
                        "give" => Verb::Give,
                        "copy" => Verb::Copy,
                        _ => {
                            if self.eat(&Tok::Dot) {
                                let m = self.ident("`mut` after `lend.`")?;
                                if m.name != "mut" {
                                    return self.fail_at(m.span, format!("`lend.{}` is not a verb: `lend` (shared) or `lend.mut` (exclusive, may modify)", m.name));
                                }
                                Verb::LendMut
                            } else {
                                Verb::Lend
                            }
                        }
                    };
                    if self.is_word("lend") || self.is_word("give") || self.is_word("copy") {
                        return self.fail("one verb per value");
                    }
                    let operand = self.postfix()?;
                    let span = start.to(operand.span);
                    Ok(Expr { kind: ExprKind::Verb { verb, amp: false, expr: Box::new(operand) }, span })
                }
                "lent" => self.fail("`lent` is the parameter's word; at a call the caller writes `lend 'x'` (or `lend.mut 'x'`)"),
                "check" | "nocheck" => self.fail(format!("`{}` goes in front of a statement, not inside a value: if half a statement needs it, split the statement", w)),
                "true" | "false" => {
                    self.bump();
                    // Bare, it is the boolean; a function of that name is
                    // reached by path (limbs.md, 2026-09-19).
                    if self.is(&Tok::LBracket) {
                        return self.fail(format!("bare `{}` is the boolean, so `{}[…]` is no call: a function named `{}` is reached by path — `limb.{}[…]`", w, w, w, w));
                    }
                    let e = Expr { kind: ExprKind::Bool(w == "true"), span: start };
                    self.postfix_on(e)
                }
                "fn" => {
                    self.bump();
                    self.fn_lit(start)
                }
                "to" => {
                    self.bump();
                    self.conv(start)
                }
                "channel" if !matches!(self.peek_at(1), Tok::Dot) => {
                    self.bump();
                    self.expect(&Tok::LBracket, "`[` after `channel`: `channel[T]`")?;
                    let elem = self.type_()?;
                    self.expect(&Tok::RBracket, "`]` closing the channel's type")?;
                    let bound = if self.is(&Tok::LBracket) {
                        self.bump();
                        let lo = self.expr(0)?;
                        self.expect(&Tok::Comma, "`,` between the channel's bounds")?;
                        let hi = self.expr(0)?;
                        self.expect(&Tok::RBracket, "`]` closing the bound")?;
                        Some(Box::new((lo, hi)))
                    } else {
                        None
                    };
                    let span = start.to(self.prev_span());
                    Ok(Expr { kind: ExprKind::Channel { elem, bound }, span })
                }
                // `ptr.cast[T]['p']` (owner, 2026-09-30: a); the other
                // `ptr.` operations are ordinary calls.
                "ptr" if matches!(self.peek_at(1), Tok::Dot) && matches!(self.peek_at(2), Tok::Word(c) if c == "cast") => {
                    self.bump();
                    self.bump();
                    self.bump();
                    self.expect(&Tok::LBracket, "`[` and the type it points at, after `ptr.cast`: `ptr.cast[Point]['p']`")?;
                    let ty = self.type_()?;
                    self.expect(&Tok::RBracket, "`]`")?;
                    self.expect(&Tok::LBracket, "`[` and the pointer: `ptr.cast[Point]['p']`")?;
                    let value = self.expr(0)?;
                    self.expect(&Tok::RBracket, "`]`")?;
                    let span = start.to(self.prev_span());
                    Ok(Expr { kind: ExprKind::PtrCast { ty, value: Box::new(value) }, span })
                }
                "size" | "align" => {
                    self.bump();
                    if !self.eat(&Tok::Dot) {
                        return self.fail(format!("`{}` is followed by `.of`: `{}.of[int32]`", w, w));
                    }
                    let of = self.ident("`of`")?;
                    if of.name != "of" {
                        return self.fail_at(of.span, format!("`{}.{}` does not exist: `{}.of[T]`", w, of.name, w));
                    }
                    self.expect(&Tok::LBracket, &format!("`[` after `{}.of`", w))?;
                    let ty = self.type_()?;
                    self.expect(&Tok::RBracket, "`]`")?;
                    let span = start.to(self.prev_span());
                    Ok(Expr { kind: ExprKind::SizeOf { align: w == "align", ty }, span })
                }
                "range" => {
                    self.bump();
                    if self.is(&Tok::Dot) {
                        return self.fail("`range` builds its bounds from values; a counter's type goes on its name: `for 'i': int32 in range[0, 3]`");
                    }
                    self.expect(&Tok::LBracket, "`[` after `range`")?;
                    let s = self.expr(0)?;
                    self.expect(&Tok::Comma, "`,`")?;
                    let e = self.expr(0)?;
                    let step = if self.eat(&Tok::Comma) { Some(Box::new(self.expr(0)?)) } else { None };
                    self.expect(&Tok::RBracket, "`]`")?;
                    let span = start.to(self.prev_span());
                    Ok(Expr { kind: ExprKind::Range { start: Box::new(s), stop: Box::new(e), step }, span })
                }
                "otherwise" => self.fail("`otherwise` follows a call that can fail: `let 'x' := f[] otherwise 0;`"),
                // `op +` (owner, 2026-09-29): the operator as a value.
                "op" if !matches!(self.peek_at(1), Tok::LBracket) => {
                    let start = self.bump().span;
                    // `op neg` / `op not`: the operators of one operand.
                    let unary = if self.is_word("neg") {
                        self.bump();
                        Some((UnOp::Neg, "neg"))
                    } else if self.is_word("not") && !matches!(self.peek_at(1), Tok::Dot) {
                        self.bump();
                        Some((UnOp::Not, "not"))
                    } else if self.is_word("not") && self.is_word_at(2, "bit") {
                        self.bump();
                        self.bump();
                        self.bump();
                        Some((UnOp::NotBit, "not.bit"))
                    } else {
                        None
                    };
                    let (op, spelled, unary) = match unary {
                        Some((u, s)) => (BinOp::Custom(s.to_string()), s.to_string(), Some(u)),
                        None => {
                            let (op, spelled) = self.op_as_value()?;
                            (op, spelled, None)
                        }
                    };
                    let span = start.to(self.prev_span());
                    Ok(Expr { kind: ExprKind::OpValue { op, spelled, unary }, span })
                }
                "match" => {
                    let start = self.bump().span;
                    let (value, arms) = self.match_parts()?;
                    let span = start.to(self.prev_span());
                    Ok(Expr { kind: ExprKind::Match { value: Box::new(value), arms }, span })
                }
                "lang" if matches!(self.peek_at(1), Tok::Colon) => self.fail("`lang:` was removed: keywords are bare"),
                w if is_keyword(w) && w != "map" && matches!(self.peek_at(1), Tok::LBracket) => {
                    self.fail(format!("`{}` is a keyword and is never bare: a function named with one is reached only through a path — `limb.{}[…]` inside its own Limb, `name::{}[…]` from outside", w, w, w))
                }
                "let" | "return" | "if" | "else" | "loop" | "while" | "for" | "break" | "continue" | "fail" | "concur" | "whichever" | "struct" | "enum" | "pub" | "const" | "module" | "extern" | "interface" | "op" | "satisfies" | "with" | "in" | "from" | "temp" | "perm" | "seq" | "mut" | "immut" | "inline" | "layout" | "var" | "func" | "MAIN" => {
                    self.fail(format!("expected a value, found `{}`", w))
                }
                // Another Limb's const, straight after `::`: `areas::'pi'`
                // (`limbs.md`).
                _ if matches!(self.peek_at(1), Tok::ColonColon) && matches!(self.peek_at(2), Tok::Name(_)) => {
                    let limb = self.ident("a Limb's name")?;
                    self.bump();
                    let Tok::Name(n) = self.peek().clone() else { unreachable!() };
                    let span = self.bump().span;
                    let path = Path { limb: Some(limb.clone()), segs: Vec::new(), span: limb.span.to(span) };
                    let e = Expr { span: start.to(span), kind: ExprKind::ConstPath { path, name: QName { name: n, span } } };
                    self.postfix_on(e)
                }
                _ => {
                    let path = self.path()?;
                    // a const by path: `a.'k'`, `std::'pi'`, `limb.'scale'`
                    if self.is(&Tok::Dot) {
                        if let Tok::Name(n) = self.peek_at(1).clone() {
                            self.bump();
                            let span = self.bump().span;
                            let name = QName { name: n, span };
                            let e = Expr { span: start.to(span), kind: ExprKind::ConstPath { path, name } };
                            return self.postfix_on(e);
                        }
                    }
                    if path.segs.len() == 1 && path.limb.is_none() && path.head() == "limb" && !self.is(&Tok::Dot) {
                        return self.fail_at(path.span, "`limb` is followed by `.` and the root item it reaches: `limb.helper[…]`");
                    }
                    // `std::print.weird…` takes each value after a `<-`, C++'s
                    // `cout <<` (owner, 2026-10-01), and no `[ ]`.
                    let weird = path.limb.as_ref().is_some_and(|l| l.name == "std") && path.segs.len() >= 2 && path.segs[0].name == "print" && path.segs.iter().any(|s| s.name == "weird");
                    if weird && self.is(&Tok::LBracket) {
                        return self.fail(format!("`{}` takes its values with arrows, not `[ ]`: `{} <- \"n is \" <- 'n' <- \\n;`", path.spelled(), path.spelled()));
                    }
                    if weird && self.is_op("<-") {
                        let mut args = Vec::new();
                        while self.eat_op("<-") {
                            // Each value up to the next arrow: a custom
                            // operator binds loosest, so it stops there.
                            let v = self.expr(1)?;
                            args.push(Arg::Positional(v));
                        }
                        let span = start.to(self.prev_span());
                        let e = Expr { kind: ExprKind::Call { path, args, otherwise: None }, span };
                        return Ok(e);
                    }
                    if weird {
                        return self.fail(format!("`{}` takes at least one value, each after a `<-`: `{} <- 'n' <- \\n;`", path.spelled(), path.spelled()));
                    }
                    if self.is(&Tok::LBracket) {
                        self.bump();
                        let args = self.args()?;
                        let span = start.to(self.prev_span());
                        let e = Expr { kind: ExprKind::Call { path, args, otherwise: None }, span };
                        self.postfix_on(e)
                    } else {
                        let span = path.span;
                        let e = Expr { kind: ExprKind::Path(path), span };
                        self.postfix_on(e)
                    }
                }
            },
            Tok::Ellipsis => {
                self.bump();
                let verb = if self.eat_word("copy") {
                    Some(Verb::Copy)
                } else if self.eat_word("give") {
                    Some(Verb::Give)
                } else if self.is_word("lend") {
                    return self.fail("`...` with no verb already lends each element; `...lend` is not written");
                } else {
                    None
                };
                let operand = self.postfix()?;
                let span = start.to(operand.span);
                Ok(Expr { kind: ExprKind::Spread { verb, expr: Box::new(operand) }, span })
            }
            _ => self.postfix(),
        }
    }

    /// A primary followed by `.field`, `[i]`, `[args]`.
    fn postfix(&mut self) -> Result<Expr> {
        let start = self.span();
        let e = match self.peek().clone() {
            Tok::Name(n) => {
                self.bump();
                Expr { kind: ExprKind::Name(QName { name: n, span: start }), span: start }
            }
            Tok::Str(s) => {
                self.bump();
                Expr { kind: ExprKind::Str(s), span: start }
            }
            Tok::Char(c) => {
                self.bump();
                Expr { kind: ExprKind::Char(c), span: start }
            }
            Tok::Newline => {
                self.bump();
                Expr { kind: ExprKind::Newline, span: start }
            }
            Tok::Num { text, bin } => {
                self.bump();
                if let Tok::Word(u) = self.peek().clone() {
                    if DURATION_UNITS.contains(&u.as_str()) {
                        let unit = self.ident("a unit")?;
                        if bin {
                            return self.fail_at(start.to(unit.span), "a duration is a whole number of its unit: `1500 millis`, not `1.5 seconds`");
                        }
                        let span = start.to(unit.span);
                        return Ok(Expr { kind: ExprKind::Duration { value: text, unit }, span });
                    }
                }
                Expr { kind: if bin { ExprKind::Bin(text) } else { ExprKind::Int(text) }, span: start }
            }
            Tok::LParen => {
                self.bump();
                let inner = self.expr(0)?;
                self.expect(&Tok::RParen, "`)`")?;
                let span = start.to(self.prev_span());
                Expr { kind: ExprKind::Paren(Box::new(inner)), span }
            }
            Tok::LBracket => {
                self.bump();
                let mut items = Vec::new();
                let mut trailing = false;
                while !self.is(&Tok::RBracket) {
                    items.push(self.expr(0)?);
                    trailing = false;
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                    trailing = true;
                }
                self.expect(&Tok::RBracket, "`]` closing the list")?;
                let span = start.to(self.prev_span());
                if items.len() == 1 && !trailing {
                    // `[expr]` is grouping: it builds nothing.
                    let inner = items.pop().unwrap();
                    Expr { kind: ExprKind::Paren(Box::new(inner)), span }
                } else {
                    Expr { kind: ExprKind::List(items), span }
                }
            }
            Tok::Word(_) | Tok::Op(_) | Tok::Ellipsis => return self.prefix(),
            Tok::Raw(_) => return self.fail("only `inline.ASM` holds raw text"),
            other => return self.fail(format!("expected a value, found {}", other.describe())),
        };
        self.postfix_on(e)
    }

    fn postfix_on(&mut self, mut e: Expr) -> Result<Expr> {
        loop {
            match self.peek().clone() {
                Tok::Dot => {
                    // `'x'.field`; `'x'.'y'` is an error
                    match self.peek_at(1).clone() {
                        Tok::Word(_) => {
                            self.bump();
                            let field = self.ident("a field name")?;
                            let span = e.span.to(field.span);
                            e = Expr { kind: ExprKind::Field { base: Box::new(e), field }, span };
                        }
                        Tok::Name(n) => {
                            self.bump();
                            return self.fail(format!("a field's name is bare: `.{}`, not `.'{}'` — quotes mark the value you hold, the route into it is bare", n, n));
                        }
                        _ => break,
                    }
                }
                Tok::LBracket => {
                    // `'xs'[2]` reads a list and `'f'[2]` calls a function
                    // (owner, 2026-09-27: `pos` is removed). One plain value
                    // is read as an index here; the checker makes it a call
                    // when the value holds a function.
                    match &e.kind {
                        ExprKind::Name(_) | ExprKind::Field { .. } | ExprKind::CallValue { .. } | ExprKind::Index { .. } => {}
                        _ => break,
                    }
                    self.bump();
                    let mut args = self.args()?;
                    let span = e.span.to(self.prev_span());
                    // A strict file says `pos` on every position, so there
                    // a read is `[pos …]` and anything else is a call.
                    let one = if self.strict {
                        args.len() == 1 && matches!(&args[0], Arg::Positional(x) if matches!(x.kind, ExprKind::Pos(_)))
                    } else {
                        args.len() == 1 && matches!(&args[0], Arg::Positional(x) if !matches!(x.kind, ExprKind::Verb { .. } | ExprKind::Spread { .. }))
                    };
                    if one {
                        let Some(Arg::Positional(index)) = args.pop() else { unreachable!() };
                        let index = match index.kind {
                            ExprKind::Pos(inner) => *inner,
                            _ => index,
                        };
                        e = Expr { kind: ExprKind::Index { base: Box::new(e), index: Box::new(index) }, span };
                    } else {
                        e = Expr { kind: ExprKind::CallValue { callee: Box::new(e), args, otherwise: None }, span };
                    }
                }
                _ => break,
            }
        }
        // Two values side by side with nothing between.
        match self.peek() {
            Tok::Str(_) | Tok::Char(_) | Tok::Newline | Tok::Num { .. } | Tok::Name(_) => {
                let here = self.span();
                return self.fail_at(e.span.to(here), "two values next to each other with nothing between: join text with `++`, separate values with `,`");
            }
            _ => {}
        }
        Ok(e)
    }

    /// The contents of a `[ … ]` after a path: positional values, `name = value`
    /// fields, or `key = value` pairs. Consumes the closing `]`.
    fn args(&mut self) -> Result<Vec<Arg>> {
        let mut args = Vec::new();
        while !self.is(&Tok::RBracket) {
            if self.at_eof() {
                return self.fail("this `[` is never closed");
            }
            // `field = value`
            if let Tok::Word(w) = self.peek().clone() {
                if self.is_op_at(1, "=") && !is_keyword(&w) {
                    let name = self.ident("a field")?;
                    self.bump(); // =
                    let value = self.expr(0)?;
                    args.push(Arg::Named(name, value));
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                    continue;
                }
            }
            if let Tok::Name(n) = self.peek().clone() {
                if self.is_op_at(1, "=") {
                    return self.fail(format!("a field's name is bare in a construction: `{} = …`, not `'{}' = …`", n, n));
                }
            }
            // `list.fill[0, count 10]`: a strict file labels the count
            // (owner, 2026-09-30); elsewhere it is refused, as `pos` is.
            if self.is_word("count") && !matches!(self.peek_at(1), Tok::LBracket | Tok::Dot | Tok::ColonColon | Tok::Comma | Tok::RBracket) {
                if !self.strict {
                    return self.fail("`count` belongs to `#strict` files: `list.fill[0, 10]`");
                }
                let start = self.span();
                self.bump();
                let inner = self.expr(0)?;
                let span = start.to(self.prev_span());
                args.push(Arg::Positional(Expr { kind: ExprKind::Count(Box::new(inner)), span }));
                if !self.eat(&Tok::Comma) {
                    break;
                }
                continue;
            }
            // `pos` the old keyword, not a function someone named `pos`.
            if self.is_word("pos") && !matches!(self.peek_at(1), Tok::LBracket | Tok::Dot | Tok::ColonColon | Tok::Comma | Tok::RBracket) {
                if !self.strict {
                    return self.fail("`pos` is gone: a position is written bare — `'xs'[2]`, `list.slice[lend 'xs', 2, 4]`");
                }
                // A strict file says every position (owner, 2026-09-27).
                let start = self.span();
                self.bump();
                let inner = self.expr(0)?;
                let span = start.to(self.prev_span());
                args.push(Arg::Positional(Expr { kind: ExprKind::Pos(Box::new(inner)), span }));
                if !self.eat(&Tok::Comma) {
                    break;
                }
                continue;
            }
            let e = self.expr(0)?;
            if self.is(&Tok::Walrus) {
                return self.fail("a field's value is given with `=`, not `:=`: the blueprint fixed the types");
            }
            if self.is_op("=") {
                self.bump();
                let v = self.expr(0)?;
                args.push(Arg::Pair(e, v));
            } else {
                args.push(Arg::Positional(e));
            }
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(&Tok::RBracket, "`]` closing the arguments")?;
        Ok(args)
    }

    fn fn_lit(&mut self, start: Span) -> Result<Expr> {
        let params = self.params(ParamContext::Fn)?;
        let (ret, fails) = self.return_spec()?;
        if let Some(r) = self.ret_lent.take() {
            return self.fail_at(r.span, "not yet: only a named `fn` hands back a borrow (`fn f [lent 'a': T] -> lent T`)");
        }
        let captures = if self.eat_word("with") { self.captures()? } else { Vec::new() };
        let outer = std::mem::replace(&mut self.returns_value, false);
        let body = self.block("the function")?;
        if self.strict && self.returns_value && ret.is_none() {
            return self.fail_at(start, "this function returns a value, so it says its type: `fn [ … ] -> int32 { … }` — a `#strict` file writes it out");
        }
        self.returns_value = outer;
        let span = start.to(self.prev_span());
        Ok(Expr { kind: ExprKind::FnLit(Box::new(FnLit { params, ret, fails, captures, body })), span })
    }

    /// After `to`: `.type[value]`, with an optional bound bracket first.
    fn conv(&mut self, start: Span) -> Result<Expr> {
        if !self.is(&Tok::Dot) {
            return self.fail("the target type follows `to` after a dot: `to.int32[…]`, `to.str.utf8[…]`");
        }
        self.bump();
        // `to.list[give 'a']`, `to.array[copy 'l']`: the element type stays,
        // so it is not written and the bracket is the value (`lists.md`).
        // They take ownership like a call, so they are parsed as one.
        if let Tok::Word(w) = self.peek().clone() {
            if (w == "list" || w == "array") && matches!(self.peek_at(1), Tok::LBracket) {
                let wspan = self.bump().span;
                self.bump();
                let args = self.args()?;
                let path = Path { limb: None, segs: vec![Ident { name: "to".into(), span: start }, Ident { name: w, span: wspan }], span: start.to(wspan) };
                if self.is(&Tok::LBracket) && !self.bracket_is_bound() {
                    let what = path.spelled();
                    return self.fail(format!("`{}` keeps the element type, so it is not written: `{}[copy 'a']`", what, what));
                }
                let span = start.to(self.prev_span());
                let e = Expr { kind: ExprKind::Call { path, args, otherwise: None }, span };
                return self.postfix_on(e);
            }
        }
        let target = self.type_in(true)?;
        if !self.is(&Tok::LBracket) {
            return self.fail(format!("`to.{}` takes the value to convert in brackets: `to.{}[…]`", crate::names::type_spelling(&target), crate::names::type_spelling(&target)));
        }
        self.bump();
        let e = self.expr(0)?;
        if self.is(&Tok::Comma) {
            return self.fail("a conversion takes exactly one value");
        }
        self.expect(&Tok::RBracket, "`]` closing the conversion")?;
        let span = start.to(self.prev_span());
        let e = Expr { kind: ExprKind::Conv { target, expr: Box::new(e), otherwise: None }, span };
        self.postfix_on(e)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParamContext {
    Fn,
    Op,
    Extern,
    Iface,
}

pub fn is_device_name(n: &str) -> bool {
    matches!(n, "con" | "nul" | "aux" | "prn")
        || (n.len() == 4 && (n.starts_with("com") || n.starts_with("lpt")) && n.as_bytes()[3].is_ascii_digit() && n.as_bytes()[3] != b'0')
}

#[allow(dead_code)]
const _: &str = NOT_A_COMMENT;

/// An item into its file's or module's list: an `extern.C { … }` block's
/// lines, each with the block's visibility.
fn push_item(items: &mut Vec<Item>, it: Item) {
    match it.kind {
        ItemKind::ExternBlock(lines) => items.extend(lines.into_iter().map(|(v, kind)| Item { vis: if it.vis == Vis::Private { v } else { it.vis }, kind, span: it.span })),
        kind => items.push(Item { vis: it.vis, kind, span: it.span }),
    }
}
