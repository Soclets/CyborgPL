//! Recursive descent over the token list. Grammar is in the comments of each fn.
//!
//! Syntax revised 2026-09-19 (design/syntax-sketch.md): `let`/`fn`, the type
//! after the name (`'x': int32`), `-> T` for a return type, bare `if`/`else if`/
//! `while`/`for`, and `[ ]` only where it builds a value (text join, list, call,
//! construction, index, `range[…]`).

use crate::ast::*;
use crate::lexer::{Tok, Token};

pub const KEYWORDS: &[&str] = &[
    "fn", "let", "return", "if", "else", "loop", "while", "for", "in", "break", "continue",
    "give", "copy", "lend", "lent", "op", "check", "nocheck", "true", "false", "mut", "immut",
    "temp", "perm", "range", "seq", "list", "array", "pos", "struct", "pub", "const",
    // retired 2026-09-19, still reserved so the errors can point at the new spelling
    "var", "func",
];

const OP_WORDS: &[&str] = &["x", "xx", "mod", "and", "or"];

#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub msg: String,
}

pub struct Parser {
    toks: Vec<Token>,
    pos: usize,
}

type PResult<T> = Result<T, ParseError>;

impl Parser {
    pub fn new(toks: Vec<Token>) -> Self { Parser { toks, pos: 0 } }

    fn peek(&self) -> &Tok { &self.toks[self.pos].tok }
    fn peek_at(&self, n: usize) -> &Tok { &self.toks[(self.pos + n).min(self.toks.len() - 1)].tok }
    fn line(&self) -> usize { self.toks[self.pos].line }
    fn advance(&mut self) -> Token { let t = self.toks[self.pos].clone(); if self.pos < self.toks.len() - 1 { self.pos += 1; } t }

    fn err<T>(&self, msg: impl Into<String>) -> PResult<T> {
        let t = &self.toks[self.pos];
        Err(ParseError { line: t.line, col: t.col, msg: msg.into() })
    }

    fn expect(&mut self, want: Tok, what: &str) -> PResult<Token> {
        if *self.peek() == want { Ok(self.advance()) } else { self.err(format!("expected {what}, found {}", self.peek())) }
    }

    fn is_word(&self, w: &str) -> bool { matches!(self.peek(), Tok::Word(s) if s == w) }

    fn expect_word(&mut self, w: &str) -> PResult<()> {
        if self.is_word(w) { self.advance(); Ok(()) } else { self.err(format!("expected `{w}`, found {}", self.peek())) }
    }

    // program := (fn | struct | MAIN block)* Eof
    pub fn program(&mut self) -> PResult<Program> {
        let mut funcs = Vec::new();
        let mut structs = Vec::new();
        let mut consts = Vec::new();
        let mut main = None;
        loop {
            // `pub` before fn / struct / const (decided 2026-09-19); the oracle is one
            // Limb, so it records the flag and enforces nothing.
            let mut is_pub = false;
            if self.is_word("pub") {
                self.advance();
                is_pub = true;
                match self.peek().clone() {
                    Tok::Word(w) if w == "fn" || w == "struct" || w == "const" => {}
                    Tok::Word(w) if w == "let" => return self.err("a Limb-level value is a `const`, not a `let`: `pub const 'x' := 1;`"),
                    Tok::Word(w) if w == "MAIN" => return self.err("MAIN is the program's entry, never `pub`"),
                    other => return self.err(format!("`pub` goes before `fn`, `struct`, or `const`, found {other}")),
                }
            }
            match self.peek().clone() {
                Tok::Eof => break,
                Tok::Word(w) if w == "fn" => { let mut f = self.func()?; f.is_pub = is_pub; funcs.push(f) }
                Tok::Word(w) if w == "struct" => { let mut d = self.struct_def()?; d.is_pub = is_pub; structs.push(d) }
                Tok::Word(w) if w == "const" => { let mut c = self.const_def()?; c.is_pub = is_pub; consts.push(c) }
                Tok::Word(w) if w == "MAIN" => {
                    if main.is_some() { return self.err("a program has exactly one MAIN"); }
                    self.advance();
                    if *self.peek() == Tok::LBracket { return self.err("MAIN is a block, not a function: `MAIN { … }`, no parameter list"); }
                    main = Some(self.block()?);
                }
                Tok::Word(w) if w == "op" => return self.err("custom operators (`op`) are not decided yet; the oracle refuses them"),
                Tok::Word(w) if w == "func" => return self.err("`func` became `fn` (2026-09-19): `fn add ['a': int32] -> int32 { … }`"),
                Tok::Word(w) if w == "let" || w == "var" => return self.err("a `let` cannot sit at the top level; it goes inside MAIN or a fn — a Limb-level value is a `const`"),
                other => return self.err(format!("expected `fn`, `struct`, `const`, or `MAIN` at the top level, found {other}")),
            }
        }
        Ok(Program { funcs, structs, consts, main })
    }

    // const := 'const' Var (':' type)? (':=' | '=') rhs ';'   — top level only (decided 2026-09-19)
    fn const_def(&mut self) -> PResult<ConstDef> {
        let line = self.line();
        self.expect_word("const")?;
        if self.is_word("mut") || self.is_word("immut") { return self.err("a `const` is neither `mut` nor `immut`; it is a constant"); }
        let name = match self.advance().tok {
            Tok::Var(v) => v,
            Tok::Word(w) => return self.err(format!("a const is named like a variable, quoted: `const '{w}'`")),
            other => return self.err(format!("expected a quoted const name, found {other}")),
        };
        let ty = if *self.peek() == Tok::Colon { self.advance(); Some(self.ty()?) } else { None };
        match (self.peek().clone(), &ty) {
            (Tok::Walrus, None) | (Tok::Eq, Some(_)) => { self.advance(); }
            (Tok::Walrus, Some(_)) => return self.err("`:=` means Precog chooses the type; with an explicit type write `=`"),
            (Tok::Eq, None) => return self.err("`=` means you said everything; with no type write `:=`"),
            (Tok::ShadowWalrus | Tok::ShadowEq, _) => return self.err("a const cannot shadow anything; there is nothing above the top level"),
            (other, _) => return self.err(format!("expected `:=` or `=` after '{name}', found {other}")),
        }
        let value = self.rhs()?;
        self.expect(Tok::Semi, "`;`")?;
        Ok(ConstDef { name, ty, value, is_pub: false, line })
    }

    // struct := 'struct' Word '{' (('mut' | 'immut')? Var ':' type)* '}'   — no `;`, no `let`
    fn struct_def(&mut self) -> PResult<StructDef> {
        let line = self.line();
        self.expect_word("struct")?;
        let name = match self.advance().tok {
            Tok::Word(w) => w,
            Tok::Var(v) => return self.err(format!("a struct's name is bare — it is a type: `struct {v}`")),
            other => return self.err(format!("expected a struct name, found {other}")),
        };
        if KEYWORDS.contains(&name.as_str()) { return self.err(format!("`{name}` is a keyword and cannot name a struct")); }
        self.expect(Tok::LBrace, "`{`")?;
        let mut fields = Vec::new();
        while *self.peek() != Tok::RBrace {
            if *self.peek() == Tok::Eof { return self.err("unclosed `struct {`"); }
            if self.is_word("let") || self.is_word("var") { return self.err("a struct field has no `let`: a struct always owns its fields — write `'number': int32`"); }
            let mut immut = false;
            if self.is_word("immut") { self.advance(); immut = true; }
            else if self.is_word("mut") { self.advance(); }
            if *self.peek() == Tok::Dot { return self.err("`immut` is a bare word before the field's name: `immut 'number': int32`"); }
            let fname = match self.advance().tok {
                Tok::Var(v) => v,
                Tok::Word(w) => return self.err(format!("a field is `'name': type` — the name first, quoted (found `{w}`)")),
                other => return self.err(format!("expected a quoted field name, found {other}")),
            };
            if *self.peek() != Tok::Colon { return self.err(format!("a field's type follows its name: `'{fname}': int32`")); }
            self.advance();
            let ty = self.ty()?;
            if fields.iter().any(|f: &Field| f.name == fname) { return self.err(format!("field '{fname}' is declared twice")); }
            if *self.peek() == Tok::Semi { return self.err("no `;` after a struct field — fields are not statements"); }
            if *self.peek() == Tok::Comma { return self.err("no `,` between struct fields — one per line"); }
            fields.push(Field { ty, name: fname, immut });
        }
        self.advance();
        if fields.is_empty() { return self.err(format!("struct `{name}` has no fields")); }
        Ok(StructDef { name, is_pub: false, fields, line })
    }

    // fn := 'fn' Word '[' params ']' ('->' type)? block
    fn func(&mut self) -> PResult<Func> {
        let line = self.line();
        self.expect_word("fn")?;
        if *self.peek() == Tok::Dot { return self.err("`fn` takes no chain; the return type goes after the parameters: `fn add [...] -> int32`"); }
        let name = match self.advance().tok {
            Tok::Word(w) => w,
            Tok::Var(v) => return self.err(format!("function names are bare: `fn {v}`, not `fn '{v}'` (quotes mean variable)")),
            other => return self.err(format!("expected a function name, found {other}")),
        };
        if KEYWORDS.contains(&name.as_str()) { return self.err(format!("`{name}` is a keyword and cannot name a function")); }
        if name == "MAIN" { return self.err("MAIN is the program's block, not a function: `MAIN { … }`"); }
        if *self.peek() == Tok::LParen { return self.err("parameters sit in `[ ]`; `( )` is arithmetic grouping"); }
        self.expect(Tok::LBracket, "`[` to open the parameter list")?;
        let mut params = Vec::new();
        if *self.peek() != Tok::RBracket {
            loop {
                params.push(self.param()?);
                if *self.peek() == Tok::Comma { self.advance(); continue; }
                break;
            }
        }
        self.expect(Tok::RBracket, "`]` to close the parameter list")?;
        let ret = match self.peek() {
            Tok::Arrow => { self.advance(); Some(self.ty()?) }
            Tok::Colon => return self.err("a return type is written `-> int32`, not `: int32` (`:` attaches a type to a name)"),
            Tok::Word(_) => return self.err("a return type is written `-> int32` after the parameter list"),
            _ => None,
        };
        let body = self.block()?;
        Ok(Func { name, is_pub: false, ret, params, body, line })
    }

    // param := ('lent' ('.' 'mut')? | 'mut' | 'immut')? Var (':' type)?
    // An owning parameter has no keyword; `lent` / `lent.mut` mirror the caller's `lend` / `lend.mut`.
    fn param(&mut self) -> PResult<Param> {
        let line = self.line();
        let mut kind = ParamKind::Owned;
        let mut mutability = None;
        match self.peek().clone() {
            Tok::Word(w) if w == "lent" => {
                self.advance();
                kind = if *self.peek() == Tok::Dot && matches!(self.peek_at(1), Tok::Word(m) if m == "mut") {
                    self.advance(); self.advance(); ParamKind::LentMut
                } else { ParamKind::Lent };
            }
            Tok::Word(w) if w == "mut" => { self.advance(); mutability = Some(Mutability::Mut); }
            Tok::Word(w) if w == "immut" => { self.advance(); mutability = Some(Mutability::Immut); }
            Tok::Word(w) if w == "var" || w == "let" => return self.err(format!("an owning parameter has no `{w}`: `'a': int32`")),
            Tok::Word(w) if w == "lend" => return self.err("a parameter is `lent` (what it receives); `lend` is what the caller does"),
            _ => {}
        }
        if *self.peek() == Tok::Dot { return self.err("a parameter's type follows its name: `lent 'text': str.utf8`"); }
        let name = match self.advance().tok {
            Tok::Var(v) => v,
            Tok::Word(w) => return self.err(format!("a parameter is `'name': type` — the name first, quoted (found `{w}`)")),
            other => return self.err(format!("expected a quoted parameter name, found {other}")),
        };
        let ty = if *self.peek() == Tok::Colon { self.advance(); Some(self.ty()?) } else { None };
        Ok(Param { kind, mutability, ty, name, line })
    }

    /// A type from already-collected chain segments (used for list/array element types).
    fn ty_from_segs(segs: &[String]) -> Result<Ty, String> {
        let toks: Vec<Token> = segs.iter().enumerate().flat_map(|(i, w)| {
            let mut v = vec![Token { tok: Tok::Word(w.clone()), line: 0, col: 0 }];
            if i + 1 < segs.len() { v.push(Token { tok: Tok::Dot, line: 0, col: 0 }); }
            v
        }).chain(std::iter::once(Token { tok: Tok::Eof, line: 0, col: 0 })).collect();
        let mut p = Parser::new(toks);
        p.ty().map_err(|e| e.msg)
    }

    // type := segments joined by '.'  (a Limb's type, `std::uer`, is not modelled)
    fn ty(&mut self) -> PResult<Ty> {
        let mut segs = Vec::new();
        loop {
            match self.advance().tok {
                Tok::Word(w) => segs.push(w),
                other => return self.err(format!("expected a type segment, found {other}")),
            }
            if *self.peek() == Tok::Dot { self.advance(); continue; }
            break;
        }
        let head = segs[0].as_str();
        let width = |s: &str, prefix: &str| -> Option<u32> { s.strip_prefix(prefix).and_then(|w| w.parse().ok()) };
        let int_w = |n: u32| match n { 8 => Some(IntWidth::W8), 16 => Some(IntWidth::W16), 32 => Some(IntWidth::W32), 64 => Some(IntWidth::W64), 128 => Some(IntWidth::W128), _ => None };
        let bad = |s: &Self, m: String| -> PResult<Ty> { s.err(m) };
        match head {
            "bool" => { if segs.len() > 1 { return bad(self, "`bool` takes no narrowing".into()); } Ok(Ty::Bool) }
            "str" => {
                if segs.len() != 2 { return bad(self, "the typed form is fully explicit: `str.utf8`, `str.utf16`, `str.utf32`, or `str.ascii` — never bare `str`".into()); }
                Ok(Ty::Str(match segs[1].as_str() {
                    "utf8" => Encoding::Utf8, "utf16" => Encoding::Utf16, "utf32" => Encoding::Utf32, "ascii" => Encoding::Ascii,
                    e => return bad(self, format!("unknown text encoding `{e}`")),
                }))
            }
            "char" => {
                if segs.len() != 2 { return bad(self, "the typed form is fully explicit: `char.point`, `char.grapheme`, or `char.byte` — never bare `char`".into()); }
                Ok(Ty::Char(match segs[1].as_str() {
                    "point" => CharUnit::Point, "grapheme" => CharUnit::Grapheme, "byte" => CharUnit::Byte,
                    u => return bad(self, format!("unknown character unit `{u}`")),
                }))
            }
            "list" | "array" => {
                if segs.len() < 2 { return bad(self, format!("`{head}` narrows to its element type: `{head}.int32`, `{head}.str.utf8`, `{head}.list.int32`")); }
                // re-parse the tail as a type
                let tail: Vec<String> = segs[1..].to_vec();
                let inner = Parser::ty_from_segs(&tail).map_err(|m| ParseError { line: self.line(), col: 0, msg: m })?;
                Ok(if head == "list" { Ty::List(Box::new(inner)) } else { Ty::Array(Box::new(inner)) })
            }
            "int" | "uint" | "bin" | "deci" => bad(self, format!("the typed form is fully explicit: `{head}` needs a width (e.g. `{head}32`)")),
            _ if KEYWORDS.contains(&head) => bad(self, format!("`{head}` is a keyword, not a type")),
            _ => {
                if segs.len() > 1 { return bad(self, format!("`{}` takes no narrowing", segs.join("."))); }
                if let Some(n) = width(head, "int") { return int_w(n).map(Ty::Int).ok_or(()).or_else(|_| bad(self, format!("no such width `{head}`; int is 8/16/32/64/128"))); }
                if let Some(n) = width(head, "uint") { return int_w(n).map(Ty::Uint).ok_or(()).or_else(|_| bad(self, format!("no such width `{head}`; uint is 8/16/32/64/128"))); }
                if let Some(n) = width(head, "bin") { return if [16, 32, 64, 128].contains(&n) { Ok(Ty::Bin(n as u8)) } else { bad(self, format!("no such width `{head}`; bin is 16/32/64/128")) }; }
                if let Some(n) = width(head, "deci") { return if [32, 64, 128].contains(&n) { Ok(Ty::Deci(n as u8)) } else { bad(self, format!("no such width `{head}`; deci is 32/64/128 (IEEE 754 defines only these)")) }; }
                // anything else is a struct name; the interpreter checks it was declared
                Ok(Ty::Struct(head.to_string()))
            }
        }
    }

    // block := '{' stmt* '}'
    fn block(&mut self) -> PResult<Vec<Stmt>> {
        self.expect(Tok::LBrace, "`{`")?;
        let mut stmts = Vec::new();
        while *self.peek() != Tok::RBrace {
            if *self.peek() == Tok::Eof { return self.err("unclosed `{`"); }
            stmts.push(self.stmt()?);
        }
        self.advance();
        Ok(stmts)
    }

    fn stmt(&mut self) -> PResult<Stmt> {
        let line = self.line();
        match self.peek().clone() {
            Tok::Word(w) => match w.as_str() {
                "let" => self.decl(),
                "var" => self.err("`var` became `let` (2026-09-19): `let 'x' := 1;` / `let 'x': int32 = 1;`"),
                "if" => self.if_stmt(),
                "loop" => self.loop_stmt(),
                "while" => self.while_stmt(),
                "for" => self.for_stmt(),
                "break" => { self.advance(); self.expect(Tok::Semi, "`;`")?; Ok(Stmt::Break { line }) }
                "continue" => { self.advance(); self.expect(Tok::Semi, "`;`")?; Ok(Stmt::Continue { line }) }
                "return" => {
                    self.advance();
                    let value = if *self.peek() == Tok::Semi { None } else { Some(self.rhs()?) };
                    self.expect(Tok::Semi, "`;` after return")?;
                    Ok(Stmt::Return { value, line })
                }
                "fn" | "func" => self.err("a `fn` cannot be defined inside a body; it goes at the top level"),
                "const" => self.err("a `const` sits at the top level of the Limb; inside a body use `let` (Precog finds the constants itself)"),
                "pub" => self.err("`pub` goes at the top level, before `fn`, `struct`, or `const`"),
                "MAIN" => self.err("MAIN cannot be nested or called"),
                "else" => self.err("`else` without an `if`"),
                "check" | "nocheck" => self.err("the check tiers are decided but their syntax is not; the oracle refuses them for now"),
                "give" | "copy" | "lend" => self.err(format!("`{w}` belongs inside a value: `'s' = {w} 't';`")),
                "list" | "array" => {
                    let (head, op, args) = self.builtin()?;
                    self.expect(Tok::Semi, "`;`")?;
                    Ok(Stmt::BuiltinStmt { head, op, args, line })
                }
                _ => {
                    let call = self.call()?;
                    self.expect(Tok::Semi, "`;` after the call")?;
                    Ok(Stmt::Call { call, line })
                }
            },
            Tok::Var(name) => {
                self.advance();
                if *self.peek() == Tok::LBracket {
                    // 'xs'[pos i] = v;  (decided 2026-09-17)
                    let idx = self.index_bracket(&name)?;
                    self.expect(Tok::Eq, "`=` after the element")?;
                    let value = self.rhs()?;
                    self.expect(Tok::Semi, "`;`")?;
                    return Ok(Stmt::AssignElem { name, index: idx, value, line });
                }
                if *self.peek() == Tok::Dot {
                    // 'mine'.field = v;  (decided 2026-09-18)
                    self.advance();
                    let field = match self.advance().tok { Tok::Word(w) => w, other => return self.err(format!("expected a field name after `'{name}'.`, found {other}")) };
                    self.expect(Tok::Eq, "`=` after the field")?;
                    let value = self.rhs()?;
                    self.expect(Tok::Semi, "`;`")?;
                    return Ok(Stmt::AssignField { name, field, value, line });
                }
                match self.peek() {
                    Tok::Eq => {}
                    Tok::Walrus | Tok::ShadowWalrus | Tok::ShadowEq => return self.err(format!("`:=` declares; to assign to the existing '{name}' write `'{name}' = …;` (to shadow it, `let '{name}' $:= …;`)")),
                    Tok::Colon => return self.err(format!("a declaration starts with `let`: `let '{name}': int32 = …;`")),
                    other => return self.err(format!("expected `=` after '{name}', found {other}")),
                }
                self.advance();
                let value = self.rhs()?;
                self.expect(Tok::Semi, "`;`")?;
                Ok(Stmt::Assign { name, value, line })
            }
            other => self.err(format!("expected a statement, found {other}")),
        }
    }

    // decl := 'let' ('mut' | 'immut')? Var (':' type)? (':=' | '=' | '$:=' | '$=') rhs ';'
    fn decl(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.expect_word("let")?;
        if *self.peek() == Tok::Dot { return self.err("`let` takes no chain: `let mut 'x': int32 = …` — mutability before the name, type after it"); }
        let mut mutability = None;
        if self.is_word("mut") { self.advance(); mutability = Some(Mutability::Mut); }
        else if self.is_word("immut") { self.advance(); mutability = Some(Mutability::Immut); }
        let name = match self.advance().tok {
            Tok::Var(v) => v,
            Tok::Word(w) if w == "mut" || w == "immut" => return self.err("say `mut` or `immut` once, before the name"),
            Tok::Word(w) => return self.err(format!("variable names are quoted: `let '{w}'` (a type goes after the name: `let 'x': {w}`)")),
            other => return self.err(format!("expected a quoted variable name, found {other}")),
        };
        let ty = if *self.peek() == Tok::Colon { self.advance(); Some(self.ty()?) } else { None };
        let shadow;
        match (self.peek().clone(), &ty) {
            (Tok::Walrus, None) => { self.advance(); shadow = false; }
            (Tok::Eq, Some(_)) => { self.advance(); shadow = false; }
            (Tok::ShadowWalrus, None) => { self.advance(); shadow = true; }
            (Tok::ShadowEq, Some(_)) => { self.advance(); shadow = true; }
            (Tok::Walrus | Tok::ShadowWalrus, Some(_)) => return self.err("`:=` means Precog chooses the type; with an explicit type write `=` (or `$=` to shadow)"),
            (Tok::Eq | Tok::ShadowEq, None) => return self.err("`=` means you said everything; with no type write `:=` (or `$:=` to shadow)"),
            (Tok::Semi, _) => return self.err(format!("'{name}' needs a value; there are no implicit zero values")),
            (other, _) => return self.err(format!("expected `:=` or `=`, found {other}")),
        }
        let value = self.rhs()?;
        self.expect(Tok::Semi, "`;`")?;
        Ok(Stmt::Decl { name, mutability, ty, value, shadow, line })
    }

    // if := 'if' expr block ('else' 'if' expr block)* ('else' block)?
    fn if_stmt(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.expect_word("if")?;
        let mut arms = vec![(self.cond()?, self.block()?)];
        let mut otherwise = None;
        while self.is_word("else") {
            self.advance();
            if *self.peek() == Tok::Dot {
                return self.err("`else.if` became `else if` (2026-09-19)");
            } else if self.is_word("if") {
                self.advance();
                arms.push((self.cond()?, self.block()?));
            } else {
                otherwise = Some(self.block()?);
                break;
            }
        }
        Ok(Stmt::If { arms, otherwise, line })
    }

    // A condition is a bare expression followed by `{`; `[ ]` around it is
    // accepted as grouping but builds nothing.
    fn cond(&mut self) -> PResult<Bracket> {
        let b = self.rhs()?;
        if *self.peek() != Tok::LBrace { return self.err(format!("expected `{{` after the condition, found {}", self.peek())); }
        Ok(b)
    }

    // loop := 'loop' block
    fn loop_stmt(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.expect_word("loop")?;
        if *self.peek() == Tok::Dot {
            self.advance();
            return match self.peek().clone() {
                Tok::Word(w) if w == "while" => self.err("`loop.while` became `while` (2026-09-19): `while 'i' < 3 { }`"),
                Tok::Word(w) if w == "temp" || w == "perm" || w == "for" || w == "range" => self.err("`loop.temp.for` / `loop.temp.range` became `for` (2026-09-19): `for 'x' in lend 'xs' { }`, `for 'i' in range[1, 10] { }`"),
                other => self.err(format!("`loop` takes no chain; loops are `loop`, `while`, and `for` (found `.{other}`)")),
            };
        }
        Ok(Stmt::Loop { body: self.block()?, line })
    }

    // while := 'while' expr block
    fn while_stmt(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.expect_word("while")?;
        Ok(Stmt::While { cond: self.cond()?, body: self.block()?, line })
    }

    // for := 'for' ('seq')? ('temp' | 'perm')? Var (':' type)? 'in' ('range' bounds | rhs) block
    // `temp` is the default (decided 2026-09-19); `perm` keeps the variable after the loop;
    // modifiers are bare words before the name, `seq` first.
    fn for_stmt(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.expect_word("for")?;
        if *self.peek() == Tok::Dot { return self.err("`for` takes no chain: `for 'x' in …`, `for seq perm 'i' in …`"); }
        let mut seq = false;
        if self.is_word("seq") { self.advance(); seq = true; }
        let perm = match self.peek().clone() {
            Tok::Word(w) if w == "temp" => { self.advance(); false }
            Tok::Word(w) if w == "perm" => { self.advance(); true }
            Tok::Word(w) if w == "seq" => return self.err("`seq` comes first: `for seq perm 'x' in …`"),
            _ => false,
        };
        let name = match self.advance().tok { Tok::Var(v) => v, other => return self.err(format!("expected a quoted loop variable, found {other}")) };
        let ty = if *self.peek() == Tok::Colon { self.advance(); Some(self.ty()?) } else { None };
        if *self.peek() == Tok::LBracket { return self.err("expected `in` before what the loop runs over"); }
        self.expect_word("in")?;
        if self.is_word("range") {
            self.advance();
            let bounds = self.range_bounds()?;
            let body = self.block()?;
            return Ok(Stmt::Range { name, perm, seq, ty, bounds, body, line });
        }
        if ty.is_some() { return self.err("a `for` over a list takes no type on its variable; the element's type is the list's"); }
        let over = self.rhs()?;
        let body = self.block()?;
        Ok(Stmt::For { name, perm, seq, over, body, line })
    }

    // range bounds := '[' expr ',' expr (',' expr)? ']'   — start, inclusive stop, step
    fn range_bounds(&mut self) -> PResult<Bracket> {
        let line = self.line();
        if *self.peek() != Tok::LBracket { return self.err("`range` builds its bounds in `[ ]`: `range[1, 10]` or `range[1, 10, 2]`"); }
        self.advance();
        let mut values = Vec::new();
        loop {
            if matches!(self.peek(), Tok::Walrus | Tok::Eq) {
                return self.err("range bounds are plain values: `range[0, 9]` — the counter's type goes on its name (`'i': int32`)");
            }
            values.push(Value::Expr(self.expr()?));
            if *self.peek() == Tok::Comma { self.advance(); continue; }
            break;
        }
        self.expect(Tok::RBracket, "`]`")?;
        if values.len() < 2 || values.len() > 3 {
            return Err(ParseError { line, col: 0, msg: format!("a range is `range[start, stop]` or `range[start, stop, step]`, not {} value(s)", values.len()) });
        }
        Ok(Bracket { values, line, one_list: false })
    }

    // rhs := bracket | expr
    // Decided 2026-09-19: `[ ]` is written only where it builds a value (a text
    // join, a list, a call's arguments…). A single expression stands bare.
    fn rhs(&mut self) -> PResult<Bracket> {
        let line = self.line();
        if *self.peek() == Tok::LBracket { return self.bracket(); }
        let e = self.expr()?;
        if self.starts_primary() {
            return self.err(format!("pieces join only inside `[ ]`: `[\"a\" 'b' \\n]` (found {} after the value)", self.peek()));
        }
        Ok(Bracket { values: vec![Value::Expr(e)], line, one_list: false })
    }

    // call := (Word '::')? Word ('.' Word)* bracket
    fn call(&mut self) -> PResult<Call> {
        let line = self.line();
        let first = match self.advance().tok { Tok::Word(w) => w, other => return self.err(format!("expected a name, found {other}")) };
        let (limb, mut path) = if *self.peek() == Tok::ColonColon {
            self.advance();
            let seg = match self.advance().tok { Tok::Word(w) => w, other => return self.err(format!("expected a name after `::`, found {other}")) };
            (Some(first), vec![seg])
        } else {
            (None, vec![first])
        };
        while *self.peek() == Tok::Dot {
            self.advance();
            match self.advance().tok { Tok::Word(w) => path.push(w), other => return self.err(format!("expected a name after `.`, found {other}")) }
        }
        if *self.peek() != Tok::LBracket {
            let shown = match &limb { Some(l) => format!("{l}::{}", path.join(".")), None => path.join(".") };
            return self.err(format!("`{shown}` is followed by {}; a call needs `[…]`", self.peek()));
        }
        let args = self.bracket()?;
        Ok(Call { limb, path, args, line })
    }

    // index := '[' 'pos' expr ']'   — 1-based; `pos` is mandatory (decided 2026-09-17)
    fn index_bracket(&mut self, name: &str) -> PResult<Expr> {
        self.expect(Tok::LBracket, "`[`")?;
        if !self.is_word("pos") { return self.err(format!("an index says it is a position: `'{name}'[pos 2]`")); }
        self.advance();
        let idx = self.expr()?;
        if *self.peek() == Tok::Comma { return self.err(format!("an index is one position: `'{name}'[pos 3]`")); }
        self.expect(Tok::RBracket, "`]` after the position")?;
        Ok(idx)
    }

    // builtin := ('list' | 'array') '.' Word bracket
    fn builtin(&mut self) -> PResult<(String, String, Bracket)> {
        let head = match self.advance().tok { Tok::Word(w) => w, _ => unreachable!() };
        self.expect(Tok::Dot, &format!("`.` after `{head}` (`{head}.length[…]`)"))?;
        let op = match self.advance().tok { Tok::Word(w) => w, other => return self.err(format!("expected an operation after `{head}.`, found {other}")) };
        let ok = match (head.as_str(), op.as_str()) {
            ("list", "length" | "append" | "remove" | "empty") | ("array", "length") => true,
            ("array", "empty") => return self.err("an empty array could never be filled; use `list.empty`"),
            _ => false,
        };
        if !ok { return self.err(format!("`{head}.{op}` does not exist; list has length/append/remove/empty, array has length")); }
        if op == "empty" {
            // `[list.empty]` — a value, not a call (decided 2026-09-17)
            let line = self.line();
            return Ok((head, op, Bracket { values: Vec::new(), line, one_list: false }));
        }
        if *self.peek() != Tok::LBracket { return self.err(format!("`{head}.{op}` is a call and needs `[…]`")); }
        let args = self.bracket()?;
        Ok((head, op, args))
    }

    // bracket := '[' (value (',' value)*)? ']'
    // A `[ ]` builds a value: one value is that value, several are a list,
    // whitespace-separated pieces join into text.
    fn bracket(&mut self) -> PResult<Bracket> {
        let line = self.line();
        self.expect(Tok::LBracket, "`[`")?;
        let mut values = Vec::new();
        let mut trailing_comma = false;
        if *self.peek() != Tok::RBracket {
            loop {
                values.push(self.value()?);
                if *self.peek() == Tok::Comma {
                    self.advance();
                    // `[1,]` — a trailing comma makes a one-element list (decided 2026-09-14)
                    if *self.peek() == Tok::RBracket { trailing_comma = true; break; }
                    continue;
                }
                break;
            }
        }
        self.expect(Tok::RBracket, "`]`")?;
        let one_list = trailing_comma && values.len() == 1;
        Ok(Bracket { values, line, one_list })
    }

    // value := expr+   — one expr is the value; several are pieces joined as text
    fn value(&mut self) -> PResult<Value> {
        let mut exprs = vec![self.expr()?];
        while self.starts_primary() {
            exprs.push(self.expr()?);
        }
        if exprs.len() == 1 {
            return Ok(Value::Expr(exprs.pop().unwrap()));
        }
        // Pieces: quoted text, variables, \n, characters, calls, parenthesised
        // expressions. A bare number is an operand, never a piece; an operator
        // expression next to a piece is ambiguous.
        for e in &exprs {
            match e {
                Expr::Int(..) | Expr::Bin(..) =>
                    return Err(ParseError { line: e.line(), col: 0, msg: "a bare number next to another piece: numbers are operands and need an operator; a number that belongs in text has a name first (`\"count: \" 'n'`)".into() }),
                Expr::Binary(..) =>
                    return Err(ParseError { line: e.line(), col: 0, msg: "an operator expression next to other pieces is ambiguous; put it in a variable or in parentheses".into() }),
                Expr::Unary(..) =>
                    return Err(ParseError { line: e.line(), col: 0, msg: "a unary expression next to other pieces is ambiguous; put it in parentheses".into() }),
                _ => {}
            }
        }
        Ok(Value::Pieces(exprs))
    }

    fn starts_primary(&self) -> bool {
        match self.peek() {
            Tok::Int(_) | Tok::Bin(_) | Tok::Text(_) | Tok::Char(_) | Tok::Newline | Tok::Var(_) | Tok::LParen => true,
            Tok::Word(w) => !OP_WORDS.contains(&w.as_str()) && w != "not",
            _ => false,
        }
    }

    fn binop(&self) -> Option<String> {
        match self.peek() {
            Tok::Op(o) => Some(o.clone()),
            Tok::Word(w) if OP_WORDS.contains(&w.as_str()) => Some(w.clone()),
            Tok::Eq => Some("=".into()),
            _ => None,
        }
    }

    // expr — precedence follows mathematics where mathematics has a settled
    // answer (decided 2026-09-14); where it does not, parentheses are required;
    // parentheses always override.
    //
    //   tightest   xx                     right-associative (a power tower)
    //              unary -                so -2 xx 2 is -(2 xx 2)
    //              x  /  mod              x chains; / and mod must be parenthesised
    //                                     when mixed with x, /, or mod (6 / 2 x 3 has
    //                                     no settled reading; mod has no notation)
    //              +  -                   left-associative
    //              == !== < > <== >==     never chained; never mixed with `not`
    //              not                    applies to one operand or a parenthesis
    //              and
    //   loosest    or                     ∧ before ∨, as in logic
    fn expr(&mut self) -> PResult<Expr> {
        let e = self.climb(0)?;
        self.validate(&e)?;
        Ok(e)
    }

    fn level(op: &str) -> u8 {
        match op {
            "or" => 1,
            "and" => 2,
            "==" | "!==" | "<" | ">" | "<==" | ">==" => 3,
            "+" | "-" => 4,
            "x" | "/" | "mod" => 5,
            "xx" => 7,
            _ => 0,
        }
    }

    fn climb(&mut self, min: u8) -> PResult<Expr> {
        let line = self.line();
        let mut lhs = self.unary()?;
        loop {
            let Some(op) = self.binop() else { break };
            if op == "=" { return self.err("`=` inside a value: assignment is a statement; equality is `==`"); }
            let lv = Self::level(&op);
            if lv < min || lv == 0 { break; }
            self.advance();
            // right-associative for xx, left for everything else
            let rhs = if op == "xx" { self.climb(lv)? } else { self.climb(lv + 1)? };
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs), line);
        }
        Ok(lhs)
    }

    fn validate(&self, e: &Expr) -> PResult<()> {
        let bad = |line: usize, msg: String| Err(ParseError { line, col: 0, msg });
        match e {
            Expr::Binary(op, l, r, line) => {
                let is_cmp = |o: &str| matches!(o, "==" | "!==" | "<" | ">" | "<==" | ">==");
                for child in [l.as_ref(), r.as_ref()] {
                    match child {
                        Expr::Binary(cop, ..) => {
                            if cop == "mod" || op == "mod" {
                                return bad(*line, format!("`{cop}` next to `{op}`: mathematics has no settled reading for mod beside another operator — parenthesise"));
                            }
                            let both_mul = matches!(op.as_str(), "x" | "/" | "mod") && matches!(cop.as_str(), "x" | "/" | "mod");
                            if both_mul && !(op == "x" && cop == "x") {
                                return bad(*line, format!("`{cop}` next to `{op}`: mathematics has no settled reading here (6 / 2 x 3, or anything with mod) — parenthesise one side"));
                            }
                            if is_cmp(op) && is_cmp(cop) {
                                return bad(*line, format!("`{cop}` next to `{op}`: comparisons do not chain — write `[(a {cop} b) and (b {op} c)]`, or parenthesise"));
                            }
                        }
                        Expr::Unary(u, ..) if u == "not" && is_cmp(op) => {
                            return bad(*line, "`not` next to a comparison: no settled reading — write `[not (a == b)]` or `[(not a) == b]`".into());
                        }
                        _ => {}
                    }
                }
                self.validate(l)?;
                self.validate(r)
            }
            Expr::Unary(_, inner, _) => self.validate(inner),
            Expr::Paren(inner, _) => self.validate(inner),
            _ => Ok(()),
        }
    }

    // unary := ('not' | '-') unary | primary
    fn unary(&mut self) -> PResult<Expr> {
        let line = self.line();
        if self.is_word("not") { self.advance(); return Ok(Expr::Unary("not".into(), Box::new(self.unary()?), line)); }
        // unary minus sits below xx: -2 xx 2 is -(2 xx 2), as in mathematics
        if *self.peek() == Tok::Op("-".into()) { self.advance(); return Ok(Expr::Unary("-".into(), Box::new(self.climb(7)?), line)); }
        self.primary()
    }

    fn primary(&mut self) -> PResult<Expr> {
        let line = self.line();
        match self.peek().clone() {
            Tok::Int(i) => { self.advance(); Ok(Expr::Int(i, line)) }
            Tok::Bin(b) => { self.advance(); Ok(Expr::Bin(b, line)) }
            Tok::Text(t) => { self.advance(); Ok(Expr::Text(t, line)) }
            Tok::Char(c) => { self.advance(); Ok(Expr::Char(c, line)) }
            Tok::Newline => { self.advance(); Ok(Expr::Newline(line)) }
            Tok::Var(v) => {
                self.advance();
                if *self.peek() == Tok::LBracket {
                    let idx = self.index_bracket(&v)?;
                    return Ok(Expr::Index(v, Box::new(idx), line));
                }
                if *self.peek() == Tok::Dot {
                    self.advance();
                    let field = match self.advance().tok { Tok::Word(w) => w, other => return self.err(format!("expected a field name after `'{v}'.`, found {other}")) };
                    return Ok(Expr::Field(v, field, line));
                }
                Ok(Expr::Var(v, line))
            }
            Tok::LParen => {
                self.advance();
                let e = self.expr()?;
                self.expect(Tok::RParen, "`)`")?;
                Ok(Expr::Paren(Box::new(e), line))
            }
            Tok::LBracket => Ok(Expr::Nested(self.bracket()?)),
            Tok::Word(w) => match w.as_str() {
                "give" | "copy" | "lend" => {
                    self.advance();
                    let verb = match w.as_str() {
                        "give" => Verb::Give,
                        "copy" => Verb::Copy,
                        _ => {
                            if *self.peek() == Tok::Dot {
                                self.advance();
                                match self.advance().tok { Tok::Word(m) if m == "mut" => Verb::LendMut, other => return self.err(format!("the only narrowing of `lend` is `.mut`, found {other}")) }
                            } else { Verb::Lend }
                        }
                    };
                    match self.advance().tok {
                        Tok::Var(v) => {
                            if *self.peek() == Tok::Dot {
                                self.advance();
                                let field = match self.advance().tok { Tok::Word(f) => f, other => return self.err(format!("expected a field name after `'{v}'.`, found {other}")) };
                                return Ok(Expr::OwnedField(verb, v, field, line));
                            }
                            Ok(Expr::Owned(verb, v, line))
                        }
                        other => self.err(format!("`{w}` takes a variable: `{w} 'name'`, found {other}")),
                    }
                }
                "lent" => self.err("`lent` describes a parameter; a caller writes `lend`"),
                "range" => self.err("`range[…]` is only decided as what a `for` runs over: `for temp 'i' in range[1, 10]`"),
                "list" | "array" => { let (h, o, a) = self.builtin()?; Ok(Expr::Builtin(h, o, a, line)) }
                "true" => { self.advance(); Ok(Expr::Bool(true, line)) }
                "false" => { self.advance(); Ok(Expr::Bool(false, line)) }
                _ if KEYWORDS.contains(&w.as_str()) => self.err(format!("`{w}` is a keyword and cannot appear inside a value")),
                _ if matches!(self.peek_at(1), Tok::LBracket) && matches!(self.peek_at(2), Tok::Var(_)) && matches!(self.peek_at(3), Tok::Eq | Tok::Walrus) => {
                    self.advance(); self.advance();
                    let mut fields = Vec::new();
                    loop {
                        let fname = match self.advance().tok { Tok::Var(v) => v, other => return self.err(format!("expected a quoted field name, found {other}")) };
                        match self.peek() {
                            Tok::Eq => { self.advance(); }
                            Tok::Walrus => return self.err(format!("a field's type is fixed by the struct: `'{fname}' = …`, not `:=`")),
                            Tok::Colon => return self.err(format!("a field is given by name: `'{fname}' = value` (its type is fixed by the struct)")),
                            other => return self.err(format!("expected `=` after '{fname}', found {other}")),
                        }
                        // one value; pieces join, `[ ]` builds a list (or just groups)
                        let vline = self.line();
                        let value = Bracket { values: vec![self.value()?], line: vline, one_list: false };
                        fields.push((fname, value));
                        if *self.peek() == Tok::Comma { self.advance(); continue; }
                        break;
                    }
                    self.expect(Tok::RBracket, "`]` to close the struct value")?;
                    Ok(Expr::Construct(w, fields, line))
                }
                _ => Ok(Expr::Call(self.call()?)),
            },
            Tok::Eof => self.err("unexpected end of file inside a value"),
            other => self.err(format!("expected a value, found {other}")),
        }
    }
}
