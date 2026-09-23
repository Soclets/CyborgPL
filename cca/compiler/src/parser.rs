//! The parser: tokens to `ast::File`. Every refusal here is a syntax rule
//! from `design/`; the sentence says what to write instead.

use crate::ast::*;
use crate::diag::{err, Diagnostic, Result};
use crate::lexer::{Tok, Token};
use crate::source::Span;

/// Bare words the language owns. A function may be named with one only
/// through a path (`limb.to[…]`); a bare `to[…]` is the keyword.
pub const KEYWORDS: &[&str] = &[
    "fn", "let", "return", "if", "else", "loop", "while", "for", "in", "break", "continue", "give", "copy", "lend", "lent", "op",
    "check", "nocheck", "true", "false", "mut", "immut", "temp", "perm", "range", "seq", "list", "array", "pos", "struct", "pub",
    "const", "module", "limb", "fail", "otherwise", "to", "with", "enum", "match", "interface", "satisfies", "extern", "ptr",
    "layout", "size", "align", "inline", "map", "channel", "concur", "whichever", "from", "str", "char", "bool", "var", "func",
    "int8", "int16", "int32", "int64", "int128", "uint8", "uint16", "uint32", "uint64", "uint128", "bin16", "bin32", "bin64",
    "bin128", "deci32", "deci64", "deci128",
];

/// The word operators; bare, but never reserved (functions.md).
pub const WORD_OPS: &[&str] = &["x", "xx", "mod", "and", "or", "not", "xor", "shl", "shr"];

pub const DURATION_UNITS: &[&str] = &["nanos", "micros", "millis", "seconds", "minutes", "hours", "nano", "micro", "milli", "second", "minute", "hour"];

pub fn is_keyword(w: &str) -> bool {
    KEYWORDS.contains(&w)
}

/// Words that name a built-in type when they head a type path.
pub fn is_primitive_head(w: &str) -> bool {
    matches!(w, "str" | "char" | "bool" | "int" | "uint" | "bin" | "deci")
        || (w.starts_with("int") && w[3..].parse::<u32>().is_ok())
        || (w.starts_with("uint") && w[4..].parse::<u32>().is_ok())
        || (w.starts_with("bin") && w[3..].parse::<u32>().is_ok())
        || (w.starts_with("deci") && w[4..].parse::<u32>().is_ok())
}

struct Parser<'a> {
    toks: &'a [Token],
    pos: usize,
}

pub fn parse_file(toks: &[Token]) -> Result<File> {
    let mut p = Parser { toks, pos: 0 };
    let mut items = Vec::new();
    while !p.at_eof() {
        items.push(p.item()?);
    }
    Ok(File { items })
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
                return self.fail("a Limb's top-level value is a `const`, not a `let`: write `pub const 'name' := …;`");
            }
        }
        let kind = self.item_kind(vis)?;
        let span = start.to(self.prev_span());
        Ok(Item { vis, kind, span })
    }

    fn item_kind(&mut self, vis: Vis) -> Result<ItemKind> {
        let w = match self.peek().clone() {
            Tok::Word(w) => w,
            Tok::Name(n) => return self.fail(format!("'{}' cannot start a top-level item: quotes mark a value; a function is `fn name […]`, a Limb-level value is `const 'name' := …;`", n)),
            other => return self.fail(format!("expected a top-level item (`fn`, `struct`, `enum`, `const`, `module`, `MAIN`), found {}", other.describe())),
        };
        match w.as_str() {
            "MAIN" => {
                self.bump();
                if self.is(&Tok::LBracket) {
                    return self.fail("`MAIN` takes no parameters and cannot be called: it is the program's block — `MAIN { … }`");
                }
                let body = self.block("MAIN")?;
                Ok(ItemKind::Main(body))
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
                if self.eat_word("fn") {
                    let decl = self.fn_decl(false)?;
                    Ok(ItemKind::ExternFn(decl))
                } else if self.eat_word("let") {
                    let mutable = self.eat_word("mut");
                    if self.is_word("immut") {
                        return self.fail("`immut` means nothing on an `extern let`: C's variable is read-only unless you write `mut`");
                    }
                    let name = self.qname("the C variable's name")?;
                    if !self.is(&Tok::Colon) {
                        return self.fail("an `extern let` needs a type: C's variable has a C-shaped type and no value here — `extern let 'x': int32;`");
                    }
                    self.bump();
                    let ty = self.type_()?;
                    if self.is_op("=") || self.is(&Tok::Walrus) {
                        return self.fail("an `extern let` has no value here: C owns the variable; write only its type");
                    }
                    self.expect_semi("the extern declaration")?;
                    Ok(ItemKind::ExternLet { mutable, name, ty })
                } else {
                    self.fail("`extern` is followed by `fn` (a C function) or `let` (a C variable)")
                }
            }
            "struct" => {
                self.bump();
                Ok(ItemKind::Struct(self.struct_decl(false)?))
            }
            "layout" => {
                self.bump();
                self.expect(&Tok::Dot, "`.c` after `layout`")?;
                let which = self.ident("`c` after `layout.`")?;
                if which.name != "c" {
                    return self.fail_at(which.span, format!("the only layout is `layout.c`, C's shape; `layout.{}` does not exist", which.name));
                }
                self.expect_word("struct")?;
                Ok(ItemKind::Struct(self.struct_decl(true)?))
            }
            "enum" => {
                self.bump();
                Ok(ItemKind::Enum(self.enum_decl()?))
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
                    items.push(self.item()?);
                }
                self.bump();
                Ok(ItemKind::Module { name, items })
            }
            "mod" => self.fail("`mod` is the remainder operator; a module is `module name { … }`"),
            "let" => self.fail("a Limb's top-level value is a `const`, not a `let`: `const 'name' := …;` (`let` lives inside a body)"),
            "check" | "nocheck" => {
                if self.is_word_at(1, "fn") {
                    self.fail(format!("`{}` cannot go on a `fn`: the tiers are never global — there is no file-wide or Limb-wide tier; put `{}` in front of the statements inside", w, w))
                } else {
                    self.fail(format!("`{}` goes in front of a statement inside a body, never at the top level: the tiers are never global", w))
                }
            }
            "inline" => self.fail("`inline.ASM` goes inside a body, as a statement, under `nocheck`"),
            "var" => self.fail("`var` became `let`, and a Limb's top-level value is a `const`"),
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

    fn fn_decl(&mut self, has_body: bool) -> Result<FnDecl> {
        if let Tok::Name(n) = self.peek().clone() {
            return self.fail(format!("function names are bare: `fn {} […]`, not `fn '{}'` — quotes mark a value you hold, and a function is not one", n, n));
        }
        if self.is(&Tok::Dot) {
            return self.fail("the return type goes after the parameters — `fn name [params] -> int32 { … }` — not on `fn`");
        }
        let name = self.ident("the function's name")?;
        if name.name == "true" || name.name == "false" {
            return self.fail_at(name.span, format!("`{}` is a boolean literal and cannot name a function", name.name));
        }
        let params = self.params(if has_body { ParamContext::Fn } else { ParamContext::Extern })?;
        if self.is(&Tok::Colon) {
            return self.fail("the return type is `-> int32`, not `: int32` — `:` attaches a type to a name, and a result is not the function's type");
        }
        let (ret, fails) = self.return_spec()?;
        let body = if has_body {
            if self.is(&Tok::Semi) {
                return self.fail(format!("`{}` has no body: a function without one is an `extern fn`", name.name));
            }
            Some(self.block("the function")?)
        } else {
            if self.is(&Tok::LBrace) {
                return self.fail(format!("an `extern fn` has no body: `{}`'s body is in C — end the declaration with `;`", name.name));
            }
            self.expect_semi("the extern declaration")?;
            None
        };
        Ok(FnDecl { name, params, ret, fails, body })
    }

    /// `-> T`, `-> T or fail[…]`, `-> fail[…]`, or nothing.
    fn return_spec(&mut self) -> Result<(Option<Type>, Vec<FailSpec>)> {
        if let Tok::Op(o) = self.peek().clone() {
            if o.starts_with("->") && o != "->" {
                return self.fail(format!("`{}` is one operator: put a space after `->` (`-> <T>`)", o));
            }
        }
        if !self.eat_op("->") {
            return Ok((None, Vec::new()));
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
        Ok(params)
    }

    fn param(&mut self, ctx: ParamContext) -> Result<Param> {
        let start = self.span();
        // An operator's operands are lent unless the op says `give`.
        let mut kind = if ctx == ParamContext::Op { ParamKind::Lent } else { ParamKind::Owned };
        let mut amp = false;
        let mut mutability = None;
        let mut variadic = false;
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
                if ctx == ParamContext::Op {
                    return self.fail_at(start, "an operator's operands are lent by default: write `'a': T` with no `lent`; `give 'a': T` takes it by value");
                }
                if ctx == ParamContext::Extern {
                    return self.fail_at(start, "an `extern fn` parameter is not `lent`: C receives values and pointers, so write `'p': ptr[T]` and hand it a lend at the call");
                }
            }
            Tok::Op(o) if o == "&" => {
                self.bump();
                amp = true;
                if self.eat_word("mut") {
                    kind = ParamKind::LentMut;
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
        Ok(Param { kind, amp, mutability, variadic: false, name, ty, span })
    }

    fn struct_decl(&mut self, layout_c: bool) -> Result<StructDecl> {
        let name = self.ident("the struct's name")?;
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
        Ok(StructDecl { name, layout_c, fields })
    }

    fn enum_decl(&mut self) -> Result<EnumDecl> {
        let name = self.ident("the enum's name")?;
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
            if self.is(&Tok::Comma) {
                return self.fail("no `,` between cases: one per line, nothing between");
            }
            if self.is(&Tok::Semi) {
                return self.fail("no `;` after a case: cases are not statements");
            }
            cases.push(EnumCase { name: cname, payload });
        }
        self.bump();
        if cases.is_empty() {
            return self.fail_at(name.span, format!("enum `{}` has no cases: an enum is one of a few named cases, so name at least one", name.name));
        }
        Ok(EnumDecl { name, cases })
    }

    fn iface_decl(&mut self) -> Result<IfaceDecl> {
        let name = self.ident("the interface's name")?;
        let open = self.expect(&Tok::LBrace, "`{` after the interface's name")?;
        let mut entries = Vec::new();
        while !self.is(&Tok::RBrace) {
            if self.at_eof() {
                return self.fail(format!("interface `{}` is never closed", name.name));
            }
            let start = self.span();
            let ename = if self.eat_word("fn") {
                IfaceEntryName::Fn(self.ident("the function's name")?)
            } else if self.eat_word("op") {
                IfaceEntryName::Op(self.op_name()?)
            } else {
                return self.fail(format!("an interface lists `fn` and `op` signatures, found {}", self.peek().describe()));
            };
            let params = self.params(ParamContext::Iface)?;
            let (ret, fails) = self.return_spec()?;
            if self.is(&Tok::LBrace) {
                return self.fail("an interface lists promises, not code: end the signature with `;` and leave the body out");
            }
            self.expect_semi("the signature")?;
            let span = start.to(self.prev_span());
            entries.push(IfaceEntry { name: ename, params, ret, fails, span });
        }
        self.bump();
        if entries.is_empty() {
            return self.fail_at(open.to(self.prev_span()), format!("interface `{}` promises nothing: list at least one `fn` or `op` signature", name.name));
        }
        Ok(IfaceDecl { name, entries })
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
        let params = self.params(ParamContext::Op)?;
        if params.len() != 2 {
            return self.fail_at(op.span, format!("an operator takes exactly two operands; `{}` is declared with {}", op.name, params.len()));
        }
        if !self.is_op("->") {
            return self.fail(format!("`-> T` is required on an operator: `op {} [...] -> T {{ … }}`", op.name));
        }
        let (ret, fails) = self.return_spec()?;
        if !fails.is_empty() {
            return self.fail_at(op.span, format!("an operator cannot fail; make `{}` a `fn` if it needs `fail[…]`", op.name));
        }
        if ret.is_none() {
            return self.fail_at(op.span, format!("`-> T` is required on an operator: `op {} [...] -> T {{ … }}`", op.name));
        }
        let body = self.block("the operator")?;
        Ok(OpDecl { op, params, ret, body })
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
            return self.fail(format!("expected `:=` or `=`; `{}` goes before the name: `let {} 'x' := …`", self.word_text(), self.word_text()));
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
                let bound = if self.eat(&Tok::Colon) { Some(self.path()?) } else { None };
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

    fn stmt_kind(&mut self) -> Result<StmtKind> {
        let w = match self.peek().clone() {
            Tok::Word(w) => w,
            Tok::LBrace => {
                let b = self.block("the block")?;
                return Ok(StmtKind::Block(b));
            }
            Tok::Semi => return self.fail("expected a statement, found `;` — there are no empty statements"),
            _ => return self.expr_stmt(),
        };
        match w.as_str() {
            "let" => {
                self.bump();
                if self.is(&Tok::Dot) {
                    return self.fail("`let` takes no chain: `mut` / `immut` are bare words before the name — `let mut 'x' := …`");
                }
                let mutability = if self.eat_word("mut") {
                    Some(Mutability::Mut)
                } else if self.eat_word("immut") {
                    Some(Mutability::Immut)
                } else {
                    None
                };
                let name = match self.peek().clone() {
                    Tok::Name(n) => {
                        let span = self.bump().span;
                        QName { name: n, span }
                    }
                    Tok::Word(w) if is_primitive_head(&w) || matches!(self.peek_at(1), Tok::Name(_)) => {
                        return self.fail(format!("a type goes after the name: `let 'x': {} = …;`", w));
                    }
                    Tok::Word(w) => return self.fail(format!("a variable's name is quoted: `let '{}' := …;`", w)),
                    other => return self.fail(format!("expected the variable's name after `let`, found {}", other.describe())),
                };
                let (ty, shadow) = self.let_type_and_marker("let")?;
                let value = self.expr_top()?;
                if self.is_op("=") {
                    return self.fail("`=` assigns; equality is `==`");
                }
                self.expect_semi("the declaration")?;
                Ok(StmtKind::Let { mutability, name, ty, shadow, value })
            }
            "var" => self.fail("`var` became `let`: `let 'x' := …;`"),
            "check" | "nocheck" => {
                let tier_span = self.bump().span;
                let tier = if w == "check" { Tier::Check } else { Tier::Nocheck };
                if self.is(&Tok::Semi) {
                    return self.fail_at(tier_span, format!("`{}` goes in front of a statement, covering its check sites: `{} 'xs'[pos 'i'] = 1;`, `{} {{ … }}`", w, w, w));
                }
                if self.is_word("check") || self.is_word("nocheck") {
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
                Ok(StmtKind::Return(Some(e)))
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
            "inline" => {
                self.bump();
                self.inline_rest()
            }
            "asm" => self.fail("an inline block is `inline.ASM { … }` (the language's name in capitals, like `MAIN`), under `nocheck`"),
            "otherwise" => self.fail("`otherwise` follows a call that can fail: `let 'x' := f[] otherwise 0;`"),
            "fn" | "struct" | "interface" | "op" => self.fail(format!("a `{}` sits at the top level of a file or a module, not inside a body", w)),
            "enum" => self.fail("an `enum` sits at the top level of a file or a module, not inside a body"),
            "const" => self.fail("a `const` sits at the top level: inside a body `let` is the only declaration, and Precog finds the constants itself"),
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
        let mut perm = false;
        if self.eat_word("seq") {
            seq = true;
        }
        if self.eat_word("perm") {
            perm = true;
        } else if self.eat_word("temp") {
            perm = false;
        }
        if self.is_word("seq") {
            return self.fail("expected a quoted loop variable: `seq` goes first, then `temp` / `perm`, then the name — `for seq perm 'i' in …`");
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
            let start = self.expr(0)?;
            self.expect(&Tok::Comma, "`,` between the range's start and stop")?;
            let stop = self.expr(0)?;
            let step = if self.eat(&Tok::Comma) { Some(self.expr(0)?) } else { None };
            self.expect(&Tok::RBracket, "`]` closing the range")?;
            let _ = rstart;
            ForIter::Range { start, stop, step }
        } else {
            ForIter::Over(self.expr(0)?)
        };
        let body = self.block("`for`")?;
        Ok(StmtKind::For { seq, perm, name, ty, iter, body })
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
        Ok(StmtKind::Match { value, arms })
    }

    fn inline_rest(&mut self) -> Result<StmtKind> {
        self.expect(&Tok::Dot, "`.ASM` after `inline`")?;
        let lang = self.ident("the language after `inline.`")?;
        match lang.name.as_str() {
            "ASM" => {}
            "asm" => return self.fail_at(lang.span, "the inline language's name is written in capitals, like `MAIN`, so a block is easy to spot: `inline.ASM`"),
            "C" | "c" => return self.fail_at(lang.span, "only `inline.ASM` exists so far; `inline.C` is not offered yet"),
            other => return self.fail_at(lang.span, format!("only `inline.ASM` exists; `inline.{}` is not a language the compiler can inline", other)),
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
                    other => return self.fail(format!("expected a binding (`'v' -> x0`, `'n' <- x0`, `clobber x16`), found {}", other.describe())),
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
        if self.is(&Tok::Colon) {
            if let ExprKind::Name(n) = &e.kind {
                return self.fail(format!("a declaration starts with `let`: `let '{}': int32 = …;` — a bare `'{}': …` is not a statement", n.name, n.name));
            }
        }
        if self.is_op("=") {
            self.bump();
            match &e.kind {
                ExprKind::Name(_) | ExprKind::Field { .. } | ExprKind::Index { .. } => {}
                ExprKind::ConstPath { .. } => return self.fail_at(e.span, "a const is never assigned: it is a value known at compile time"),
                _ => return self.fail_at(e.span, "only a variable, a field or an element can be assigned to"),
            }
            let value = self.expr_top()?;
            if self.is_op("=") {
                return self.fail("`=` assigns; equality is `==`");
            }
            self.expect_semi("the assignment")?;
            return Ok(StmtKind::Assign { target: e, value });
        }
        if self.is(&Tok::Walrus) {
            return self.fail("a declaration starts with `let`: `let 'x' := …;`");
        }
        match &e.kind {
            ExprKind::Call { .. } | ExprKind::CallValue { .. } | ExprKind::Conv { .. } => {}
            ExprKind::Path(p) => return self.fail_at(e.span, format!("`{}` alone is not a statement: a call needs `[…]` — `{}[…];`", p.spelled(), p.spelled())),
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
            match &mut e.kind {
                ExprKind::Call { otherwise, .. } | ExprKind::CallValue { otherwise, .. } | ExprKind::Conv { otherwise, .. } => {
                    if otherwise.is_some() {
                        return self.fail_at(ow_span, "a call takes one `otherwise`");
                    }
                    *otherwise = Some(Box::new(ow));
                    e.span = e.span.to(self.prev_span());
                }
                _ => {
                    return self.fail_at(e.span, "a call that can fail must be the whole value: put it in its own `let`, then use the result — `otherwise` cannot sit inside a larger expression");
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
    fn binop(&self) -> Option<(BinOp, u8, u8, Span)> {
        let span = self.span();
        let (op, l, r) = match self.peek() {
            Tok::Op(o) => match o.as_str() {
                "+" => (BinOp::Add, 6, 7),
                "-" => (BinOp::Sub, 6, 7),
                "/" => (BinOp::Div, 7, 8),
                "++" => (BinOp::Join, 5, 6),
                "==" => (BinOp::Eq, 4, 5),
                "!==" => (BinOp::Ne, 4, 5),
                "<" => (BinOp::Lt, 4, 5),
                ">" => (BinOp::Gt, 4, 5),
                "<==" => (BinOp::Le, 4, 5),
                ">==" => (BinOp::Ge, 4, 5),
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
                    "mod" => (BinOp::Mod, 7, 8),
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
            let Some((op, l_bp, r_bp, op_span)) = self.binop() else { break };
            if l_bp < min_bp {
                break;
            }
            // consume the operator token(s)
            self.bump();
            match op {
                BinOp::WrapMul | BinOp::WrapPow => {
                    self.bump();
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
                return self.fail_at(op_span, "`++` joins text; there is no increment — write `'i' = 'i' + 1;`");
            }
            // The near-misses the language refuses by name.
            if let BinOp::Custom(c) = &op {
                let hint = match c.as_str() {
                    "!=" => Some("not-equal is `!==`"),
                    ">=" => Some("greater-or-equal is `>==` — a lone `=` is assignment and never sits inside a comparison"),
                    "<=" => Some("less-or-equal is `<==` — a lone `=` is assignment and never sits inside a comparison"),
                    "--" => Some("there is no decrement: write `'i' = 'i' - 1;`"),
                    "+=" | "-=" => Some("there is no compound assignment: write `'i' = 'i' + 1;`"),
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
                if *op == BinOp::Mod || *inner == BinOp::Mod {
                    return self.fail_at(op_span, format!("`{}` next to `{}`: mathematics has no settled reading for `mod` beside another operator — parenthesise (`('i' mod 2) == 0`)", inner.spelled(), op.spelled()));
                }
                if (*op == BinOp::Div && inner.is_multiplicative()) || (op.is_multiplicative() && *inner == BinOp::Div) {
                    return self.fail_at(op_span, format!("`{}` next to `{}`: mathematics has no settled reading here (`6 / 2 x 3`) — parenthesise one side", inner.spelled(), op.spelled()));
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
                "lang" if matches!(self.peek_at(1), Tok::Colon) => self.fail("`lang:` was removed: keywords are bare"),
                w if is_keyword(w) && w != "map" && matches!(self.peek_at(1), Tok::LBracket) => {
                    self.fail(format!("`{}` is a keyword and is never bare: a function named with one is reached only through a path — `limb.{}[…]` inside its own Limb, `name::{}[…]` from outside", w, w, w))
                }
                "let" | "return" | "if" | "else" | "loop" | "while" | "for" | "break" | "continue" | "fail" | "match" | "concur" | "whichever" | "struct" | "enum" | "pub" | "const" | "module" | "extern" | "interface" | "op" | "satisfies" | "with" | "in" | "from" | "pos" | "temp" | "perm" | "seq" | "mut" | "immut" | "inline" | "layout" | "var" | "func" | "MAIN" => {
                    self.fail(format!("expected a value, found `{}`", w))
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

    /// A primary followed by `.field`, `[pos i]`, `[args]`.
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
                    // `'xs'[pos i]` or `'f'[args]`
                    if self.is_word_at(1, "pos") {
                        self.bump();
                        self.bump();
                        let index = self.expr(0)?;
                        self.expect(&Tok::RBracket, "`]` closing the index")?;
                        let span = e.span.to(self.prev_span());
                        e = Expr { kind: ExprKind::Index { base: Box::new(e), index: Box::new(index) }, span };
                    } else {
                        match &e.kind {
                            ExprKind::Name(_) | ExprKind::Field { .. } | ExprKind::CallValue { .. } | ExprKind::Index { .. } => {}
                            _ => break,
                        }
                        self.bump();
                        let args = self.args()?;
                        let span = e.span.to(self.prev_span());
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
            // `pos i` as an argument: `simd.load[lend 'xs', pos 'i']`.
            if self.is_word("pos") {
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
        let captures = if self.eat_word("with") { self.captures()? } else { Vec::new() };
        let body = self.block("the function")?;
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
