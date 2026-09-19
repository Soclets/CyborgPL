//! Recursive descent over the token list. Grammar is in the comments of each fn.
//!
//! Syntax revised 2026-09-19 (design/syntax-sketch.md): `let`/`fn`, the type
//! after the name (`'x': int32`), `-> T` for a return type, bare `if`/`else if`/
//! `while`/`for`, and `[ ]` only where it builds a value (text join, list, call,
//! construction, index, `range[…]`).

use crate::ast::*;
use crate::lexer::{Tok, Token};
use crate::value::ty_name;

pub const KEYWORDS: &[&str] = &[
    "fn", "let", "return", "if", "else", "loop", "while", "for", "in", "break", "continue",
    "give", "copy", "lend", "lent", "op", "check", "nocheck", "true", "false", "mut", "immut",
    "temp", "perm", "range", "seq", "list", "array", "pos", "struct", "pub", "const", "module", "limb", "fail", "otherwise", "to",
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
    /// the module being parsed: [] for limb.cpls, ["shapes"] for shapes.cpls,
    /// extended by each `module name { … }` block
    module: Vec<String>,
    /// set while parsing the type in `to.<type>[…]`: the bracket belongs to the
    /// conversion, not to the type
    to_target: bool,
}

type PResult<T> = Result<T, ParseError>;

impl Parser {
    pub fn new(toks: Vec<Token>) -> Self { Parser { toks, pos: 0, module: Vec::new(), to_target: false } }
    /// A parser for a file that is the module `module` (its path from the root).
    pub fn new_in(toks: Vec<Token>, module: Vec<String>) -> Self { Parser { toks, pos: 0, module, to_target: false } }

    /// A path written in this module, made canonical (from the root):
    /// `limb.x.f` → x.f; a relative `x.f` → <this module>.x.f  (decided 2026-09-19)
    fn resolve(&self, segs: &[String]) -> Vec<String> {
        if segs.first().map(|s| s == "limb").unwrap_or(false) { return segs[1..].to_vec(); }
        let mut out = self.module.clone();
        out.extend(segs.iter().cloned());
        out
    }

    fn canonical(&self, segs: &[String]) -> String { self.resolve(segs).join(".") }

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

    // program := item* Eof
    // item := ('pub' ('.' 'limb')?)? (fn | struct | const | module) | MAIN block
    pub fn program(&mut self) -> PResult<Program> {
        let mut prog = Program { funcs: Vec::new(), structs: Vec::new(), consts: Vec::new(), modules: Vec::new(), main: None };
        self.items(&mut prog, 0)?;
        Ok(prog)
    }

    /// Parse items until `}` (depth > 0) or end of file (depth 0).
    fn items(&mut self, prog: &mut Program, depth: usize) -> PResult<()> {
        loop {
            if depth > 0 && *self.peek() == Tok::RBrace { self.advance(); return Ok(()); }
            if *self.peek() == Tok::Eof {
                if depth > 0 { return self.err("unclosed `module {`"); }
                return Ok(());
            }
            // visibility (decided 2026-09-19): nothing, `pub.limb`, `pub`
            let mut vis = Vis::Private;
            if self.is_word("pub") {
                self.advance();
                vis = Vis::Pub;
                if *self.peek() == Tok::Dot {
                    self.advance();
                    match self.advance().tok {
                        Tok::Word(w) if w == "limb" => vis = Vis::Limb,
                        other => return self.err(format!("the only narrowing of `pub` is `pub.limb` (visible in this Limb only), found {other}")),
                    }
                }
                match self.peek().clone() {
                    Tok::Word(w) if w == "fn" || w == "struct" || w == "const" || w == "module" => {}
                    Tok::Word(w) if w == "let" => return self.err("a Limb-level value is a `const`, not a `let`: `pub const 'x' := 1;`"),
                    Tok::Word(w) if w == "MAIN" => return self.err("MAIN is the program's entry, never `pub`"),
                    other => return self.err(format!("`pub` goes before `fn`, `struct`, `const`, or `module`, found {other}")),
                }
            }
            match self.peek().clone() {
                Tok::Word(w) if w == "fn" => { let mut f = self.func()?; f.vis = vis; prog.funcs.push(f) }
                Tok::Word(w) if w == "struct" => { let mut d = self.struct_def()?; d.vis = vis; prog.structs.push(d) }
                Tok::Word(w) if w == "const" => { let mut c = self.const_def()?; c.vis = vis; prog.consts.push(c) }
                Tok::Word(w) if w == "module" => {
                    let line = self.line();
                    self.advance();
                    let name = match self.advance().tok {
                        Tok::Word(w) => w,
                        Tok::Var(v) => return self.err(format!("a module's name is bare: `module {v}`")),
                        other => return self.err(format!("expected a module name, found {other}")),
                    };
                    if let Err(m) = module_name_ok(&name) { return self.err(format!("`{name}` cannot name a module: {m}")); }
                    if *self.peek() == Tok::Semi { return self.err(format!("a module is a file (`{name}.cpls`) or a block (`module {name} {{ … }}`); there is no `module {name};` include line")); }
                    self.expect(Tok::LBrace, "`{` to open the module")?;
                    self.module.push(name);
                    prog.modules.push(ModuleDef { path: self.module.clone(), vis, line });
                    self.items(prog, depth + 1)?;
                    self.module.pop();
                }
                Tok::Word(w) if w == "MAIN" => {
                    if !self.module.is_empty() { return self.err("MAIN lives at the root of the Limb (`limb.cpls`, outside any module)"); }
                    if prog.main.is_some() { return self.err("a program has exactly one MAIN"); }
                    self.advance();
                    if *self.peek() == Tok::LBracket { return self.err("MAIN is a block, not a function: `MAIN { … }`, no parameter list"); }
                    prog.main = Some(self.block()?);
                }
                Tok::Word(w) if w == "op" => return self.err("custom operators (`op`) are not decided yet; the oracle refuses them"),
                Tok::Word(w) if w == "func" => return self.err("`func` became `fn` (2026-09-19): `fn add ['a': int32] -> int32 { … }`"),
                Tok::Word(w) if w == "let" || w == "var" => return self.err("a `let` cannot sit at the top level; it goes inside MAIN or a fn — a Limb-level value is a `const`"),
                Tok::Word(w) if w == "mod" => return self.err("`mod` is the remainder operator; a module block is `module name { … }`"),
                Tok::Word(w) if w == "check" || w == "nocheck" => return self.err(format!("`{w}` goes in front of a statement inside a body; there is no file-wide or Limb-wide tier")),
                Tok::RBrace => return self.err("`}` with nothing open"),
                other => return self.err(format!("expected `fn`, `struct`, `const`, `module`, or `MAIN` at the top level, found {other}")),
            }
        }
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
        Ok(ConstDef { name, module: self.module.clone(), vis: Vis::Private, ty, value, line })
    }

    // struct := 'struct' Word '{' (('mut' | 'immut')? Word ':' type)* '}'   — no `;`, no `let`
    // Field names are bare (decided 2026-09-19): a field is a path into the struct, not a
    // variable you hold; the quotes go on the whole (`'l'`), the route into it is bare.
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
            if self.is_word("let") || self.is_word("var") { return self.err("a struct field has no `let`: a struct always owns its fields — write `number: int32`"); }
            let mut immut = false;
            if self.is_word("immut") { self.advance(); immut = true; }
            else if self.is_word("mut") { self.advance(); }
            if *self.peek() == Tok::Dot { return self.err("`immut` is a bare word before the field's name: `immut number: int32`"); }
            let fname = match self.advance().tok {
                Tok::Word(w) if KEYWORDS.contains(&w.as_str()) => return self.err(format!("`{w}` is a keyword and cannot name a field")),
                Tok::Word(w) => w,
                Tok::Var(v) => return self.err(format!("a field's name is bare: `{v}: int32` — a field is a path into the struct, not a variable (decided 2026-09-19)")),
                other => return self.err(format!("expected a field name, found {other}")),
            };
            if let Tok::Var(v) = self.peek().clone() { return self.err(format!("a field is `name: type` — the name first, bare: `{v}: {fname}`")); }
            if *self.peek() != Tok::Colon { return self.err(format!("a field's type follows its name: `{fname}: int32`")); }
            self.advance();
            let ty = self.ty()?;
            if fields.iter().any(|f: &Field| f.name == fname) { return self.err(format!("field '{fname}' is declared twice")); }
            if *self.peek() == Tok::Semi { return self.err("no `;` after a struct field — fields are not statements"); }
            if *self.peek() == Tok::Comma { return self.err("no `,` between struct fields — one per line"); }
            fields.push(Field { ty, name: fname, immut });
        }
        self.advance();
        if fields.is_empty() { return self.err(format!("struct `{name}` has no fields")); }
        Ok(StructDef { name: self.canonical(&[name]), module: self.module.clone(), vis: Vis::Private, fields, line })
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
        // A keyword may name a function (decided 2026-09-19) — it is then never called
        // bare, only through a path: `limb.to[…]` inside, `std::to[…]` outside.
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
        // -> T | -> T or fail[…] | -> fail[…]   (decided 2026-09-19)
        let mut ret = None;
        let mut fails = Vec::new();
        match self.peek() {
            Tok::Arrow => {
                self.advance();
                if self.is_word("fail") { fails = self.fail_list()?; }
                else {
                    ret = Some(self.ty()?);
                    if self.is_word("or") {
                        self.advance();
                        if !self.is_word("fail") { return self.err("after `or` comes `fail[…]`: `-> bool or fail[full, bad_item: str.utf8]`"); }
                        fails = self.fail_list()?;
                    }
                }
            }
            Tok::Colon => return self.err("a return type is written `-> int32`, not `: int32` (`:` attaches a type to a name)"),
            Tok::Word(w) if w == "or" || w == "fail" => return self.err("a function that can fail says so after `->`: `-> bool or fail[full]` or `-> fail[full]`"),
            Tok::Word(_) => return self.err("a return type is written `-> int32` after the parameter list"),
            _ => {}
        }
        let body = self.block()?;
        Ok(Func { name, module: self.module.clone(), vis: Vis::Private, ret, fails, params, body, line })
    }

    // fail list := 'fail' '[' Word (':' type)? (',' Word (':' type)?)* ']'
    fn fail_list(&mut self) -> PResult<Vec<FailCase>> {
        self.expect_word("fail")?;
        self.expect(Tok::LBracket, "`[` after `fail` to list the ways this function can fail")?;
        let mut cases = Vec::new();
        loop {
            let name = match self.advance().tok {
                Tok::Word(w) if KEYWORDS.contains(&w.as_str()) => return self.err(format!("`{w}` is a keyword and cannot name a failure")),
                Tok::Word(w) => w,
                Tok::Var(v) => return self.err(format!("a failure's name is bare: `fail[{v}]` — it names a reason, not a variable")),
                other => return self.err(format!("expected a failure name, found {other}")),
            };
            if cases.iter().any(|c: &FailCase| c.name == name) { return self.err(format!("failure `{name}` is listed twice")); }
            let ty = if *self.peek() == Tok::Colon { self.advance(); Some(self.ty()?) } else { None };
            cases.push(FailCase { name, ty });
            if *self.peek() == Tok::Comma { self.advance(); continue; }
            break;
        }
        self.expect(Tok::RBracket, "`]` to close the failure list")?;
        Ok(cases)
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

    // type := Word ('.' Word)* ('[' type (',' type)* ']')?
    // Decided 2026-09-19: `.` picks a fixed variant (`str.utf8`, `char.grapheme`);
    // `[ ]` supplies type parameters (`list[int32]`, `list[list[int32]]`). A struct
    // through modules is a dotted path (`shapes.Locker`, `limb.Locker`).
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
        let mut params = Vec::new();
        if *self.peek() == Tok::LBracket && !self.to_target {
            self.advance();
            loop {
                params.push(self.ty()?);
                if *self.peek() == Tok::Comma { self.advance(); continue; }
                break;
            }
            self.expect(Tok::RBracket, "`]` to close the type parameters")?;
        }
        let head = segs[0].as_str();
        let width = |s: &str, prefix: &str| -> Option<u32> { s.strip_prefix(prefix).and_then(|w| w.parse().ok()) };
        let int_w = |n: u32| match n { 8 => Some(IntWidth::W8), 16 => Some(IntWidth::W16), 32 => Some(IntWidth::W32), 64 => Some(IntWidth::W64), 128 => Some(IntWidth::W128), _ => None };
        let bad = |s: &Self, m: String| -> PResult<Ty> { s.err(m) };
        let no_params = |s: &Self, name: &str| -> PResult<()> { if params.is_empty() { Ok(()) } else { s.err(format!("`{name}` takes no type parameters")) } };
        match head {
            "bool" => { if segs.len() > 1 { return bad(self, "`bool` takes no narrowing".into()); } no_params(self, "bool")?; Ok(Ty::Bool) }
            "str" => {
                if segs.len() != 2 { return bad(self, "the typed form is fully explicit: `str.utf8`, `str.utf16`, `str.utf32`, or `str.ascii` — never bare `str`".into()); }
                no_params(self, "str")?;
                Ok(Ty::Str(match segs[1].as_str() {
                    "utf8" => Encoding::Utf8, "utf16" => Encoding::Utf16, "utf32" => Encoding::Utf32, "ascii" => Encoding::Ascii,
                    e => return bad(self, format!("unknown text encoding `{e}`")),
                }))
            }
            "char" => {
                if segs.len() != 2 { return bad(self, "the typed form is fully explicit: `char.point`, `char.grapheme`, or `char.byte` — never bare `char`".into()); }
                no_params(self, "char")?;
                Ok(Ty::Char(match segs[1].as_str() {
                    "point" => CharUnit::Point, "grapheme" => CharUnit::Grapheme, "byte" => CharUnit::Byte,
                    u => return bad(self, format!("unknown character unit `{u}`")),
                }))
            }
            "list" | "array" => {
                if segs.len() > 1 { return bad(self, format!("`{head}` takes its element type in brackets: `{head}[{}]` (decided 2026-09-19), not `{}`", segs[1..].join("."), segs.join("."))); }
                if params.len() != 1 { return bad(self, format!("`{head}` takes exactly one type parameter: `{head}[int32]`, `{head}[str.utf8]`, `{head}[list[int32]]`")); }
                let inner = params.pop().unwrap();
                Ok(if head == "list" { Ty::List(Box::new(inner)) } else { Ty::Array(Box::new(inner)) })
            }
            "int" | "uint" | "bin" | "deci" => bad(self, format!("the typed form is fully explicit: `{head}` needs a width (e.g. `{head}32`)")),
            _ if KEYWORDS.contains(&head) => bad(self, format!("`{head}` is a keyword, not a type")),
            _ => {
                if !params.is_empty() { return bad(self, format!("`{}` takes no type parameters (generic structs are not designed)", segs.join("."))); }
                if segs.len() > 1 {
                    // a struct reached through modules: `shapes.Locker`, `limb.Locker`
                    if segs[..segs.len() - 1].iter().any(|m| m == "limb" || module_name_ok(m).is_ok()) {
                        return Ok(Ty::Struct(self.canonical(&segs)));
                    }
                    return bad(self, format!("`{}` takes no narrowing", segs.join(".")));
                }
                if let Some(n) = width(head, "int") { return int_w(n).map(Ty::Int).ok_or(()).or_else(|_| bad(self, format!("no such width `{head}`; int is 8/16/32/64/128"))); }
                if let Some(n) = width(head, "uint") { return int_w(n).map(Ty::Uint).ok_or(()).or_else(|_| bad(self, format!("no such width `{head}`; uint is 8/16/32/64/128"))); }
                if let Some(n) = width(head, "bin") { return if [16, 32, 64, 128].contains(&n) { Ok(Ty::Bin(n as u8)) } else { bad(self, format!("no such width `{head}`; bin is 16/32/64/128")) }; }
                if let Some(n) = width(head, "deci") { return if [32, 64, 128].contains(&n) { Ok(Ty::Deci(n as u8)) } else { bad(self, format!("no such width `{head}`; deci is 32/64/128 (IEEE 754 defines only these)")) }; }
                // anything else is a struct name in this module; the interpreter checks it was declared
                Ok(Ty::Struct(self.canonical(&segs)))
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
                "pub" => self.err("`pub` goes at the top level, before `fn`, `struct`, `const`, or `module`"),
                "module" => self.err("a `module` block sits at the top level of a file, not inside a body"),
                "MAIN" => self.err("MAIN cannot be nested or called"),
                "else" => self.err("`else` without an `if`"),
                "check" | "nocheck" => {
                    // decided 2026-09-19: a word before a statement (or a `{ }` block)
                    self.advance();
                    let tier = if w == "check" { Tier::Check } else { Tier::NoCheck };
                    match self.peek().clone() {
                        Tok::Semi => return self.err(format!("`{w}` goes in front of a statement: `{w} 'xs'[pos 'i'] = 5;` or `{w} {{ … }}`")),
                        Tok::Word(k) if k == "fn" || k == "struct" || k == "const" || k == "module" => return self.err(format!("`{w}` covers a statement, not a `{k}` — a function's safety is the sum of its statements")),
                        Tok::Word(k) if k == "check" || k == "nocheck" => return self.err(format!("`{w} {k}`: one tier per statement; the innermost wins when they nest")),
                        _ => {}
                    }
                    let body = self.stmt()?;
                    Ok(Stmt::Tier { tier, body: Box::new(body), line })
                }
                "give" | "copy" | "lend" => self.err(format!("`{w}` belongs inside a value: `'s' = {w} 't';`")),
                "list" | "array" => {
                    let (head, op, args) = self.builtin()?;
                    self.expect(Tok::Semi, "`;`")?;
                    Ok(Stmt::BuiltinStmt { head, op, args, line })
                }
                "fail" => {
                    // fail name; | fail name[payload];   (decided 2026-09-19)
                    self.advance();
                    let name = match self.advance().tok {
                        Tok::Word(w) if KEYWORDS.contains(&w.as_str()) => return self.err(format!("`{w}` is a keyword, not a failure name")),
                        Tok::Word(w) => w,
                        Tok::Var(v) => return self.err(format!("a failure's name is bare: `fail {v};`")),
                        Tok::Semi => return self.err("`fail` names its reason: `fail full;` — the reasons are listed in the signature's `fail[…]`"),
                        other => return self.err(format!("expected a failure name after `fail`, found {other}")),
                    };
                    let payload = if *self.peek() == Tok::LBracket { Some(self.bracket()?) } else { None };
                    self.expect(Tok::Semi, "`;` after fail")?;
                    Ok(Stmt::Fail { name, payload, line })
                }
                "otherwise" => self.err("`otherwise` follows a call that can fail: `stash[…] otherwise false;`"),
                "to" => self.err("`to.…[…]` is a value; use it: `let 'n' := to.int32[\"42\"] otherwise 0;`"),
                _ => {
                    let mut call = self.call()?;
                    call.handler = self.handler()?;
                    self.expect(Tok::Semi, "`;` after the call")?;
                    Ok(Stmt::Call { call, line })
                }
            },
            Tok::LBrace => Ok(Stmt::Block { body: self.block()?, line }),
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
                    if *self.peek() == Tok::LBracket { return self.err(format!("assigning into an element of a field (`'{name}'.{field}[pos …] = …`) is not modelled by the oracle; lend the list to a name first: `let 'xs' := lend.mut '{name}'.{field};`")); }
                    self.expect(Tok::Eq, "`=` after the field")?;
                    let value = self.rhs()?;
                    self.expect(Tok::Semi, "`;`")?;
                    return Ok(Stmt::AssignField { name, field, value, line });
                }
                match self.peek() {
                    Tok::Eq => {}
                    Tok::Op(o) if o == "++" => return self.err(format!("`++` joins text; to add one write `'{name}' = '{name}' + 1;`")),
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
        let mut e = self.expr()?;
        if self.starts_primary() {
            return self.err(format!("two values next to each other (found {} after the value): join text with `++`", self.peek()));
        }
        // `call[…] otherwise …` — only when the call is the whole right-hand side (decided 2026-09-19)
        if self.is_word("otherwise") {
            match &mut e {
                Expr::Call(c) => c.handler = self.handler()?,
                Expr::Convert(_, _, h, _) => *h = self.handler()?,
                _ => return self.err("`otherwise` handles a call that can fail, and the call must be the whole value: put it in its own `let` first"),
            }
        }
        Ok(Bracket { values: vec![Value::Expr(e)], line, one_list: false })
    }

    // handler := 'otherwise' ( 'fail' ';'-ahead | leaving-stmt | '{' arm* '}' | expr )
    fn handler(&mut self) -> PResult<Option<Handler>> {
        if !self.is_word("otherwise") { return Ok(None); }
        self.advance();
        let h = match self.peek().clone() {
            Tok::Word(w) if w == "fail" && *self.peek_at(1) == Tok::Semi => { self.advance(); Handler::Propagate }
            Tok::Word(w) if w == "fail" || w == "return" || w == "break" || w == "continue"
                || (w == "std" && *self.peek_at(1) == Tok::ColonColon && matches!(self.peek_at(2), Tok::Word(e) if e == "exit")) => {
                // a leaving statement, `;` included; the outer statement's `;` is this one
                let s = self.stmt()?;
                self.pos -= 1; // hand the `;` back to the enclosing statement
                Handler::Leave(Box::new(s))
            }
            Tok::LBrace => {
                self.advance();
                let mut arms = Vec::new();
                while *self.peek() != Tok::RBrace {
                    let line = self.line();
                    let name = match self.advance().tok {
                        Tok::Word(w) => w,
                        Tok::Var(v) => return self.err(format!("an arm starts with the failure's bare name; the payload's name comes after it: `bad_item {v} {{ … }}`")),
                        Tok::Eof => return self.err("unclosed `otherwise {`"),
                        other => return self.err(format!("expected a failure name to handle, found {other}")),
                    };
                    let bind = if let Tok::Var(v) = self.peek().clone() { self.advance(); Some(v) } else { None };
                    let body = self.block()?;
                    if arms.iter().any(|a: &Arm| a.name == name) { return self.err(format!("`{name}` is handled twice")); }
                    arms.push(Arm { name, bind, body, line });
                }
                self.advance();
                if arms.is_empty() { return self.err("`otherwise { }` handles nothing; list the failures, or use `otherwise fail;`"); }
                Handler::Arms(arms)
            }
            Tok::Semi => return self.err("`otherwise` needs what to do: a value, `fail`, `return …`, or `{ arms }`"),
            _ => {
                let line = self.line();
                let e = self.expr()?;
                Handler::Value(Bracket { values: vec![Value::Expr(e)], line, one_list: false })
            }
        };
        Ok(Some(h))
    }

    /// Word ('.' Word)* '.' Var  — a const reached by path
    fn is_const_path(&self) -> bool {
        let mut i = 0;
        loop {
            if !matches!(self.peek_at(i), Tok::Word(_)) { return false; }
            if *self.peek_at(i + 1) != Tok::Dot { return false; }
            match self.peek_at(i + 2) { Tok::Var(_) => return true, Tok::Word(_) => i += 2, _ => return false }
        }
    }

    /// Word ('.' Word)* '[' (Word | Var) ('=' | ':=' | ':')  — a struct construction
    /// (a Var here is the pre-2026-09-19 spelling, kept so the error can say so)
    fn is_construct(&self) -> bool {
        let mut i = 0;
        loop {
            if !matches!(self.peek_at(i), Tok::Word(_)) { return false; }
            match self.peek_at(i + 1) {
                Tok::Dot => { i += 2; }
                Tok::LBracket => return matches!(self.peek_at(i + 2), Tok::Var(_) | Tok::Word(_)) && matches!(self.peek_at(i + 3), Tok::Eq | Tok::Walrus | Tok::Colon),
                _ => return false,
            }
        }
    }

    // call := (Word '::')? Word ('.' Word)* bracket
    fn call(&mut self) -> PResult<Call> {
        let line = self.line();
        let first = match self.advance().tok { Tok::Word(w) => w, other => return self.err(format!("expected a name, found {other}")) };
        if KEYWORDS.contains(&first.as_str()) && first != "limb" && *self.peek() != Tok::ColonColon {
            return self.err(format!("`{first}` is a keyword; a function named `{first}` is reached through a path, never bare: `limb.{first}[…]`"));
        }
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
        let path = if limb.is_none() { self.resolve(&path) } else { path };
        Ok(Call { handler: None, limb, path, from: self.module.clone(), args, line })
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

    // value := expr   — one expression per comma-separated slot.
    // Decided 2026-09-19: joining is explicit (`++`); two values side by side are an error.
    fn value(&mut self) -> PResult<Value> {
        let e = self.expr()?;
        if self.starts_primary() {
            return self.err(format!("two values next to each other (found {} after the value): join text with `++`, separate values with `,`", self.peek()));
        }
        Ok(Value::Expr(e))
    }

    fn starts_primary(&self) -> bool {
        match self.peek() {
            Tok::Int(_) | Tok::Bin(_) | Tok::Text(_) | Tok::Char(_) | Tok::Newline | Tok::Var(_) | Tok::LParen => true,
            Tok::Word(w) => !OP_WORDS.contains(&w.as_str()) && w != "not" && w != "otherwise",
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
    //              ++                     text join, left-associative; below arithmetic,
    //                                     above comparisons (decided 2026-09-19)
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
            "++" => 4,
            "+" | "-" => 5,
            "x" | "/" | "mod" => 6,
            "xx" => 8,
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
        if *self.peek() == Tok::Op("-".into()) { self.advance(); return Ok(Expr::Unary("-".into(), Box::new(self.climb(8)?), line)); }
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
                    if *self.peek() == Tok::LBracket {
                        let idx = self.index_bracket(&format!("{v}'.{field}"))?;
                        return Ok(Expr::FieldIndex(v, field, Box::new(idx), line));
                    }
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
                "check" | "nocheck" => self.err(format!("`{w}` goes in front of a statement, not inside a value; if half a statement needs it, split the statement")),
                "list" | "array" => { let (h, o, a) = self.builtin()?; Ok(Expr::Builtin(h, o, a, line)) }
                "to" => {
                    // to.<type>[value]   (decided 2026-09-19)
                    self.advance();
                    self.expect(Tok::Dot, "`.` after `to` (`to.int32[…]`, `to.str.utf8[…]`)")?;
                    self.to_target = true;
                    let ty = self.ty();
                    self.to_target = false;
                    let ty = ty?;
                    if matches!(ty, Ty::List(_) | Ty::Array(_) | Ty::Struct(_)) { return self.err(format!("no conversion to {}; conversions are between numbers and text", ty_name(&ty))); }
                    if *self.peek() != Tok::LBracket { return self.err(format!("`to.{}` converts a value: `to.{}[…]`", ty_name(&ty), ty_name(&ty))); }
                    let b = self.bracket()?;
                    if b.values.len() != 1 { return self.err("`to.…[…]` converts exactly one value"); }
                    Ok(Expr::Convert(ty, b, None, line))
                }
                "true" => { self.advance(); Ok(Expr::Bool(true, line)) }
                "false" => { self.advance(); Ok(Expr::Bool(false, line)) }
                "limb" if !matches!(self.peek_at(1), Tok::Dot) => self.err("`limb` is the root of this Limb and is followed by `.`: `limb.helper[]`, `limb.'pi'`"),
                _ if KEYWORDS.contains(&w.as_str()) && w != "limb" => self.err(format!("`{w}` is a keyword and cannot appear inside a value")),
                _ if self.is_const_path() => {
                    // `shapes.'pi'`, `limb.'pi'`
                    let mut segs = Vec::new();
                    loop {
                        match self.advance().tok { Tok::Word(w) => segs.push(w), _ => unreachable!() }
                        self.expect(Tok::Dot, "`.`")?;
                        if let Tok::Var(_) = self.peek() { break; }
                    }
                    let name = match self.advance().tok { Tok::Var(v) => v, _ => unreachable!() };
                    Ok(Expr::ConstPath(self.resolve(&segs), name, self.module.clone(), line))
                }
                _ if self.is_construct() => {
                    let mut segs = Vec::new();
                    loop {
                        match self.advance().tok { Tok::Word(w) => segs.push(w), _ => unreachable!() }
                        if *self.peek() == Tok::Dot { self.advance(); continue; }
                        break;
                    }
                    let w = self.canonical(&segs);
                    self.advance(); // [
                    let mut fields = Vec::new();
                    loop {
                        let fname = match self.advance().tok {
                            Tok::Word(w) => w,
                            Tok::Var(v) => return self.err(format!("a field's name is bare: `{v} = …` (decided 2026-09-19)")),
                            other => return self.err(format!("expected a field name, found {other}")),
                        };
                        match self.peek() {
                            Tok::Eq => { self.advance(); }
                            Tok::Walrus => return self.err(format!("a field's type is fixed by the struct: `{fname} = …`, not `:=`")),
                            Tok::Colon => return self.err(format!("a field is given by name: `{fname} = value` (its type is fixed by the struct)")),
                            other => return self.err(format!("expected `=` after `{fname}`, found {other}")),
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

/// Module names (decided 2026-09-19, "whatever has most compat"): lowercase
/// ASCII letters, digits, `_`, starting with a letter; not a Windows device
/// name; not a keyword. The same rule for file names and `module` blocks.
pub fn module_name_ok(name: &str) -> Result<(), String> {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        Some(c) if c.is_ascii_uppercase() => return Err("lowercase only — macOS and Windows file systems do not tell `Shapes` from `shapes`".into()),
        _ => return Err("it must start with a lowercase ASCII letter".into()),
    }
    if let Some(c) = name.chars().find(|c| !(c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_')) {
        return Err(format!("`{c}` is not allowed — lowercase ASCII letters, digits, and `_` only"));
    }
    const DEVICES: &[&str] = &["con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8", "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9"];
    if DEVICES.contains(&name) { return Err("it is a reserved device name on Windows".into()); }
    if KEYWORDS.contains(&name) { return Err("it is a keyword".into()); }
    Ok(())
}
