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
    "temp", "perm", "range", "seq", "list", "array", "pos", "struct", "pub", "const", "module", "limb", "fail", "otherwise", "to", "extern", "layout", "ptr", "size", "align", "inline", "asm", "ASM", "enum", "match", "interface", "satisfies", "xor", "shl", "shr", "bit", "with",
    // retired 2026-09-19, still reserved so the errors can point at the new spelling
    "var", "func",
];

const OP_WORDS: &[&str] = &["x", "xx", "mod", "and", "or"];
/// Bitwise operators (decided 2026-09-21): the boolean words narrowed with `.bit`, plus xor/shl/shr;
/// no precedence — parentheses whenever one meets another operator.
const BIT_OPS: &[&str] = &["and.bit", "or.bit", "xor", "shl", "shr"];

#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub msg: String,
}

/// Counts every `&` / `&mut` lend in the program. The build then says
/// "What are you? Rust programmer or Claude?" once per `&`, on stdout,
/// and carries on (owner, 2026-09-21). You're not supposed to use that.
pub static AMPERSANDS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn note_ampersand() { AMPERSANDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed); }

pub struct Parser {
    toks: Vec<Token>,
    pos: usize,
    /// the module being parsed: [] for limb.cpls, ["shapes"] for shapes.cpls,
    /// extended by each `module name { … }` block
    module: Vec<String>,
    /// set while parsing the type in `to.<type>[…]`: the bracket belongs to the
    /// conversion, not to the type
    to_target: bool,
    /// set while parsing an extern signature or a layout.c struct: `ptr[T]` is allowed
    at_boundary: bool,
    /// set while parsing an interface's entries: signatures end with `;`, no body
    in_interface: bool,
    /// `<T: Ordered>` bounds seen while parsing the current signature
    bounds: Vec<(String, String)>,
    /// captures of the anonymous function just parsed
    pending_captures: Vec<(Verb, String)>,
}

type PResult<T> = Result<T, ParseError>;

impl Parser {
    pub fn new(toks: Vec<Token>) -> Self { Parser { toks, pos: 0, module: Vec::new(), to_target: false, at_boundary: false, in_interface: false, bounds: Vec::new(), pending_captures: Vec::new() } }
    /// A parser for a file that is the module `module` (its path from the root).
    pub fn new_in(toks: Vec<Token>, module: Vec<String>) -> Self { Parser { toks, pos: 0, module, to_target: false, at_boundary: false, in_interface: false, bounds: Vec::new(), pending_captures: Vec::new() } }

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
        let mut prog = Program { funcs: Vec::new(), structs: Vec::new(), enums: Vec::new(), interfaces: Vec::new(), claims: Vec::new(), consts: Vec::new(), externs: Vec::new(), modules: Vec::new(), main: None };
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
                    Tok::Word(w) if w == "fn" || w == "struct" || w == "const" || w == "module" || w == "extern" || w == "layout" || w == "enum" || w == "op" || w == "interface" => {}
                    Tok::Word(w) if w == "let" => return self.err("a Limb-level value is a `const`, not a `let`: `pub const 'x' := 1;`"),
                    Tok::Word(w) if w == "MAIN" => return self.err("MAIN is the program's entry, never `pub`"),
                    other => return self.err(format!("`pub` goes before `fn`, `struct`, `const`, or `module`, found {other}")),
                }
            }
            match self.peek().clone() {
                Tok::Word(w) if w == "fn" => { let mut f = self.func()?; f.vis = vis; prog.funcs.push(f) }
                Tok::Word(w) if w == "struct" => { let mut d = self.struct_def()?; d.vis = vis; prog.structs.push(d) }
                Tok::Word(w) if w == "enum" => {
                    // enum Name { case  case: type … }   (design/enums.md, 2026-09-20)
                    let line = self.line();
                    self.advance();
                    let name = match self.advance().tok {
                        Tok::Word(n) if KEYWORDS.contains(&n.as_str()) => return self.err(format!("`{n}` is a keyword and cannot name an enum")),
                        Tok::Word(n) => n,
                        Tok::Var(v) => return self.err(format!("an enum's name is bare — it is a type: `enum {v}`")),
                        other => return self.err(format!("expected an enum name, found {other}")),
                    };
                    self.expect(Tok::LBrace, "`{`")?;
                    let mut cases: Vec<FailCase> = Vec::new();
                    while *self.peek() != Tok::RBrace {
                        let cname = match self.advance().tok {
                            Tok::Word(c) if KEYWORDS.contains(&c.as_str()) => return self.err(format!("`{c}` is a keyword and cannot name a case")),
                            Tok::Word(c) => c,
                            Tok::Var(v) => return self.err(format!("a case's name is bare, like a field: `{v}` — it is a path into the type")),
                            Tok::Eof => return self.err("unclosed `enum {`"),
                            other => return self.err(format!("expected a case name, found {other}")),
                        };
                        if cases.iter().any(|c| c.name == cname) { return self.err(format!("case `{cname}` is declared twice")); }
                        let ty = if *self.peek() == Tok::Colon { self.advance(); Some(self.ty()?) } else { None };
                        if *self.peek() == Tok::Semi { return self.err("no `;` after a case"); }
                        if *self.peek() == Tok::Comma { return self.err("no `,` between cases — one per line, like struct fields"); }
                        cases.push(FailCase { name: cname, ty });
                    }
                    self.advance();
                    if cases.is_empty() { return self.err(format!("enum `{name}` has no cases")); }
                    let mut params = Vec::new();
                    for c in &cases { if let Some(t) = &c.ty { t.vars_into(&mut params); } }
                    prog.enums.push(EnumDef { name: self.canonical(&[name]), module: self.module.clone(), vis, cases, params, line });
                }
                Tok::Word(w) if w == "extern" => {
                    // extern fn name [params] -> T;   (design/ffi.md)
                    let line = self.line();
                    self.advance();
                    if self.is_word("let") {
                        // extern let ['mut'] 'name': T;   — a C global (decided 2026-09-21)
                        self.advance();
                        let mutable = if self.is_word("mut") { self.advance(); true } else { false };
                        if self.is_word("immut") { return self.err("an `extern let` is C's variable; it is read-only unless `mut`, never `immut`"); }
                        let name = match self.advance().tok { Tok::Var(v) => v, Tok::Word(w) => return self.err(format!("an extern variable is named like a variable, quoted: `extern let '{w}': …`")), other => return self.err(format!("expected a quoted name after `extern let`, found {other}")) };
                        if *self.peek() != Tok::Colon { return self.err(format!("`extern let '{name}'` needs a type; C has no inference: `extern let '{name}': int32;`")); }
                        self.advance();
                        self.at_boundary = true;
                        let ty = self.ty();
                        self.at_boundary = false;
                        let ty = ty?;
                        if matches!(self.peek(), Tok::Walrus | Tok::Eq) { return self.err(format!("`extern let '{name}'` has no value here; C owns it. A value of your own is a `const`")); }
                        self.expect(Tok::Semi, "`;` after the extern variable's type")?;
                        prog.externs.push(ExternVar { name, module: self.module.clone(), vis, ty, mutable, line });
                        continue;
                    }
                    if !self.is_word("fn") { return self.err("`extern` goes before `fn` or `let`: `extern fn write [...] -> int64;`, `extern let 'errno': int32;`"); }
                    self.at_boundary = true;
                    let f = self.func();
                    self.at_boundary = false;
                    let mut f = f?;
                    f.vis = vis; f.is_extern = true;
                    prog.funcs.push(f);
                }
                Tok::Word(w) if w == "layout" => {
                    // layout.c struct Name { … }   (design/ffi.md)
                    self.advance();
                    self.expect(Tok::Dot, "`.c` after `layout` (`layout.c struct …`)")?;
                    match self.advance().tok { Tok::Word(l) if l == "c" => {}, other => return self.err(format!("the only layout is `layout.c`, found {other}")) }
                    if !self.is_word("struct") { return self.err("`layout.c` goes before `struct`: `layout.c struct Timeval { … }`"); }
                    self.at_boundary = true;
                    let d = self.struct_def();
                    self.at_boundary = false;
                    let mut d = d?;
                    d.vis = vis; d.layout_c = true;
                    prog.structs.push(d);
                }
                Tok::Word(w) if w == "asm" || w == "ASM" || w == "inline" => return self.err("an `inline.ASM` block is a statement; it goes inside a body, under `nocheck`"),
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
                Tok::Word(w) if w == "op" => { let mut f = self.op_decl()?; f.vis = vis; prog.funcs.push(f); }
                Tok::Word(w) if w == "interface" => {
                    // interface Name { fn …;  op …; }   (design/interfaces.md, 2026-09-20)
                    let line = self.line();
                    self.advance();
                    let name = match self.advance().tok {
                        Tok::Word(n) if KEYWORDS.contains(&n.as_str()) => return self.err(format!("`{n}` is a keyword and cannot name an interface")),
                        Tok::Word(n) => n,
                        Tok::Var(v) => return self.err(format!("an interface's name is bare: `interface {v}`")),
                        other => return self.err(format!("expected an interface name, found {other}")),
                    };
                    self.expect(Tok::LBrace, "`{`")?;
                    let mut entries = Vec::new();
                    self.in_interface = true;
                    while *self.peek() != Tok::RBrace {
                        let r = match self.peek().clone() {
                            Tok::Word(w) if w == "fn" => self.func(),
                            Tok::Word(w) if w == "op" => self.op_decl(),
                            Tok::Eof => Err(ParseError { line: self.line(), col: 0, msg: "unclosed `interface {`".into() }),
                            other => Err(ParseError { line: self.line(), col: 0, msg: format!("an interface lists `fn` and `op` signatures; found {other}") }),
                        };
                        match r { Ok(e) => entries.push(e), Err(e) => { self.in_interface = false; return Err(e); } }
                    }
                    self.in_interface = false;
                    self.advance();
                    if entries.is_empty() { return self.err(format!("interface `{name}` promises nothing")); }
                    prog.interfaces.push(InterfaceDef { name: self.canonical(&[name]), module: self.module.clone(), vis, entries, line });
                }
                Tok::Word(_) if matches!(self.peek_at(1), Tok::Word(s) if s == "satisfies") || (matches!(self.peek_at(1), Tok::Dot) && false) => {
                    // Vec satisfies Ordered;   — an optional, verified claim
                    let line = self.line();
                    let ty = self.ty()?;
                    self.expect_word("satisfies")?;
                    let mut segs = Vec::new();
                    loop {
                        match self.advance().tok { Tok::Word(w) => segs.push(w), other => return self.err(format!("expected an interface's name after `satisfies`, found {other}")) }
                        if *self.peek() == Tok::Dot { self.advance(); continue; }
                        break;
                    }
                    self.expect(Tok::Semi, "`;` after the claim")?;
                    prog.claims.push(Claim { ty, iface: self.canonical(&segs), line });
                }
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


    // op NAME ['a': T, 'b': T] -> R { … }   (decided 2026-09-20)
    // The language's own operators may be defined for a user type (2026-09-20), never
    // for the language's types; an interface entry may name them for a `<T>`.
    fn op_decl(&mut self) -> PResult<Func> {
        let line = self.line();
        self.bounds.clear();
        self.expect_word("op")?;
        let name = match self.advance().tok {
            Tok::Word(n) if KEYWORDS.contains(&n.as_str()) => return self.err(format!("`{n}` is a keyword and cannot be an operator")),
            Tok::Word(n) if n == "not" => return self.err("`not` is the language's; it cannot be redefined"),
            Tok::Word(n) => n,
            Tok::Op(o) => o,
            Tok::Var(v) => return self.err(format!("an operator's name is bare — a word or a run of operator characters, not `'{v}'`")),
            other => return self.err(format!("expected an operator name after `op`, found {other}")),
        };
        self.expect(Tok::LBracket, "`[` to open the two operands")?;
        let mut params = Vec::new();
        loop {
            let pline = self.line();
            let kind = match self.peek().clone() {
                Tok::Word(w) if w == "give" => { self.advance(); ParamKind::Owned }
                Tok::Word(w) if w == "lent" || w == "lend" => return self.err("an operator's operands are lent by default; write just `'a': T` (or `give 'a': T` to take it by value)"),
                Tok::Word(w) if w == "mut" || w == "immut" || w == "copy" => return self.err(format!("`{w}` has no meaning on an operand; an operand is lent, or `give`")),
                Tok::Ellipsis => return self.err("an operator takes exactly two operands; no `...`"),
                _ => ParamKind::Lent,
            };
            let pname = match self.advance().tok { Tok::Var(v) => v, other => return self.err(format!("expected a quoted operand name, found {other}")) };
            let ty = if *self.peek() == Tok::Colon { self.advance(); Some(self.ty()?) } else { None };
            params.push(Param { kind, mutability: None, ty, name: pname, variadic: false, line: pline });
            if *self.peek() == Tok::Comma { self.advance(); continue; }
            break;
        }
        self.expect(Tok::RBracket, "`]`")?;
        if params.len() != 2 { return self.err(format!("an operator takes exactly two operands, `op {name}` has {}", params.len())); }
        let builtin = Parser::level(&name) != 0 || OP_WORDS.contains(&name.as_str());
        if builtin && !self.in_interface {
            let user_type = matches!(params[0].ty, Some(Ty::Struct(_)) | Some(Ty::Inst(..)));
            if !user_type { return self.err(format!("`{name}` is the language's operator; it can be defined for a struct or enum of yours, never changed for the language's own types")); }
        }
        let ret = if *self.peek() == Tok::Arrow { self.advance(); Some(self.ty()?) } else { return self.err("an operator produces a value: `-> T` is required"); };
        if self.is_word("or") { return self.err("an operator cannot fail; make it a `fn` if it can"); }
        let body = if self.in_interface {
            if *self.peek() == Tok::LBrace { return self.err("an interface lists promises, not code: end the signature with `;`"); }
            self.expect(Tok::Semi, "`;` to end the promise")?;
            Vec::new()
        } else { self.block()? };
        let bounds = std::mem::take(&mut self.bounds);
        Ok(Func { name, module: self.module.clone(), vis: Vis::Private, ret, fails: Vec::new(), is_extern: false, is_op: true, bounds, params, body, line })
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
        let mut params = Vec::new();
        for f in &fields { f.ty.vars_into(&mut params); }
        Ok(StructDef { name: self.canonical(&[name]), module: self.module.clone(), vis: Vis::Private, layout_c: false, params, fields, line })
    }

    // fn := 'fn' Word '[' params ']' ('->' type)? block
    fn func(&mut self) -> PResult<Func> {
        let line = self.line();
        self.bounds.clear();
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
                    let saved = self.at_boundary; self.at_boundary = true;
                    let r = self.ty();
                    self.at_boundary = saved;
                    ret = Some(r?);
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
        let body = if self.at_boundary {
            if *self.peek() == Tok::LBrace { return self.err("an `extern fn` has no body — it lives somewhere else; end it with `;`"); }
            if !fails.is_empty() { return self.err("an `extern fn` cannot `fail`; C has no failures — a wrapper turns its return value into one"); }
            self.expect(Tok::Semi, "`;` to end the extern declaration")?;
            Vec::new()
        } else if self.in_interface {
            if *self.peek() == Tok::LBrace { return self.err("an interface lists promises, not code: end the signature with `;`"); }
            self.expect(Tok::Semi, "`;` to end the promise")?;
            Vec::new()
        } else { self.block()? };
        let bounds = std::mem::take(&mut self.bounds);
        Ok(Func { name, module: self.module.clone(), vis: Vis::Private, ret, fails, is_extern: false, is_op: false, bounds, params, body, line })
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
            Tok::Op(o) if o == "&" => {
                note_ampersand();
                // `&'text': T` / `&mut 'buf': T` — an alternative to `lent` / `lent.mut` (owner, 2026-09-21)
                self.advance();
                kind = if self.is_word("mut") { self.advance(); ParamKind::LentMut } else { ParamKind::Lent };
            }
            Tok::Word(w) if w == "mut" => { self.advance(); mutability = Some(Mutability::Mut); }
            Tok::Word(w) if w == "immut" => { self.advance(); mutability = Some(Mutability::Immut); }
            Tok::Word(w) if w == "var" || w == "let" => return self.err(format!("an owning parameter has no `{w}`: `'a': int32`")),
            Tok::Word(w) if w == "lend" => return self.err("a parameter is `lent` (what it receives); `lend` is what the caller does"),
            _ => {}
        }
        if *self.peek() == Tok::Dot { return self.err("a parameter's type follows its name: `lent 'text': str.utf8`"); }
        let mut variadic = false;
        if *self.peek() == Tok::Ellipsis { self.advance(); variadic = true; }
        let name = match self.advance().tok {
            Tok::Var(v) => v,
            Tok::Word(w) => return self.err(format!("a parameter is `'name': type` — the name first, quoted (found `{w}`)")),
            Tok::Ellipsis => return self.err("`...` goes right before the name: `...'values'`"),
            other => return self.err(format!("expected a quoted parameter name, found {other}")),
        };
        // a signature may mention `ptr[T]` (a callback C calls receives pointers); a `let` may not
        let saved = self.at_boundary; self.at_boundary = true;
        let ty = if *self.peek() == Tok::Colon { self.advance(); Some(self.ty()) } else { None };
        self.at_boundary = saved;
        let ty = match ty { Some(t) => Some(t?), None => None };
        if variadic && *self.peek() == Tok::Comma { return self.err(format!("`...'{name}'` takes the rest of the arguments, so it must be the last parameter")); }
        Ok(Param { kind, mutability, ty, name, variadic, line })
    }

    // type := Word ('.' Word)* ('[' type (',' type)* ']')?
    // Decided 2026-09-19: `.` picks a fixed variant (`str.utf8`, `char.grapheme`);
    // `[ ]` supplies type parameters (`list[int32]`, `list[list[int32]]`). A struct
    // through modules is a dotted path (`shapes.Locker`, `limb.Locker`).
    fn ty(&mut self) -> PResult<Ty> {
        // `<T>` — a type variable (design/generics.md, 2026-09-20)
        if matches!(self.peek(), Tok::Op(o) if o == "<") {
            self.advance();
            let name = match self.advance().tok {
                Tok::Word(n) if KEYWORDS.contains(&n.as_str()) => return self.err(format!("`{n}` is a keyword and cannot name a type variable")),
                Tok::Word(n) => n,
                Tok::Var(v) => return self.err(format!("a type variable is a bare name: `<{v}>`")),
                other => return self.err(format!("expected a type variable's name after `<`, found {other}")),
            };
            if *self.peek() == Tok::Colon {
                // `<T: Ordered>` — a bound (design/interfaces.md)
                self.advance();
                let mut segs = Vec::new();
                loop {
                    match self.advance().tok { Tok::Word(w) => segs.push(w), other => return self.err(format!("expected an interface's name after `<{name}:`, found {other}")) }
                    if *self.peek() == Tok::Dot { self.advance(); continue; }
                    break;
                }
                let iface = self.canonical(&segs);
                if let Some((_, other)) = self.bounds.iter().find(|(v, i)| *v == name && *i != iface) { return self.err(format!("`<{name}>` is already bound to `{other}`; one blank, one interface")); }
                if !self.bounds.iter().any(|(v, _)| *v == name) { self.bounds.push((name.clone(), iface)); }
            }
            match self.advance().tok { Tok::Op(o) if o == ">" => {}, other => return self.err(format!("expected `>` to close `<{name}`, found {other}")) }
            return Ok(Ty::Var(name));
        }
        if matches!(self.peek(), Tok::Op(o) if o.starts_with('<')) { return self.err("a type variable is `<T>` — with a space before it after `->` (`-> <T>`), since a run of operator characters is one token"); }
        // `fn[lent T, U] -> R or fail[…]` — a function's type (2026-09-21)
        if self.is_word("fn") {
            self.advance();
            self.expect(Tok::LBracket, "`[` after `fn` in a type: `fn[lent Locker, int32] -> bool`")?;
            let mut ps = Vec::new();
            if *self.peek() != Tok::RBracket {
                loop {
                    let kind = match self.peek().clone() {
                        Tok::Word(w) if w == "lent" => { self.advance(); if *self.peek() == Tok::Dot && matches!(self.peek_at(1), Tok::Word(m) if m == "mut") { self.advance(); self.advance(); ParamKind::LentMut } else { ParamKind::Lent } }
                        Tok::Op(o) if o == "&" => { note_ampersand(); self.advance(); if self.is_word("mut") { self.advance(); ParamKind::LentMut } else { ParamKind::Lent } }
                        Tok::Var(v) => return self.err(format!("a function type has no parameter names: `fn[lent Locker]`, not `fn[lent '{v}': Locker]`")),
                        _ => ParamKind::Owned,
                    };
                    let t = self.ty()?;
                    ps.push((kind, t));
                    if *self.peek() == Tok::Comma { self.advance(); continue; }
                    break;
                }
            }
            self.expect(Tok::RBracket, "`]`")?;
            let mut ret = None;
            let mut fails = Vec::new();
            if *self.peek() == Tok::Arrow {
                self.advance();
                if self.is_word("fail") { fails = self.fail_list()?.into_iter().map(|c| c.name).collect(); }
                else {
                    ret = Some(Box::new(self.ty()?));
                    if self.is_word("or") { self.advance(); if !self.is_word("fail") { return self.err("after `or` comes `fail[…]`"); } fails = self.fail_list()?.into_iter().map(|c| c.name).collect(); }
                }
            }
            return Ok(Ty::Fn(ps, ret, fails));
        }
        let mut segs = Vec::new();
        loop {
            match self.advance().tok {
                Tok::Word(w) => segs.push(w),
                other => return self.err(format!("expected a type segment, found {other}")),
            }
            if *self.peek() == Tok::Dot { self.advance(); continue; }
            break;
        }
        // `int32[0, 100]` — a range type (design/input.md): a numeric head with two bounds
        {
            let head = segs[0].as_str();
            let numeric = ["int", "uint", "bin", "deci"].iter().any(|p| head.strip_prefix(p).map(|w| w.parse::<u32>().is_ok()).unwrap_or(false));
            let range_ahead = *self.peek() == Tok::LBracket && (matches!(self.peek_at(1), Tok::Int(_) | Tok::Bin(_)) || matches!(self.peek_at(1), Tok::Op(o) if o == "-"));
            if numeric && range_ahead && (!self.to_target || self.bracket_then_bracket()) {
                self.advance();
                let lo = self.bound()?;
                self.expect(Tok::Comma, "`,` between the bounds")?;
                let hi = self.bound()?;
                self.expect(Tok::RBracket, "`]` after the bounds")?;
                let base = Parser::new_in(vec![Token { tok: Tok::Word(head.to_string()), line: 0, col: 0 }, Token { tok: Tok::Eof, line: 0, col: 0 }], self.module.clone()).ty().map_err(|e| ParseError { line: self.line(), col: 0, msg: e.msg })?;
                match (&base, &lo, &hi) {
                    (Ty::Int(_) | Ty::Uint(_), Bound::Bin(_), _) | (Ty::Int(_) | Ty::Uint(_), _, Bound::Bin(_)) => return self.err("an integer's bounds are integers"),
                    (Ty::Bin(_) | Ty::Deci(_), Bound::Int(_), _) | (Ty::Bin(_) | Ty::Deci(_), _, Bound::Int(_)) => return self.err("a bin's bounds are written as bins: `bin64[0.0, 1.0]`"),
                    _ => {}
                }
                let ok = match (&lo, &hi) { (Bound::Int(a), Bound::Int(b)) => a <= b, (Bound::Bin(a), Bound::Bin(b)) => a <= b, _ => true };
                if !ok { return self.err("a range's low bound comes first: `int32[0, 100]`"); }
                return Ok(Ty::Ranged(Box::new(base), lo, hi));
            }
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
        if head == "ptr" {
            if !self.at_boundary { return bad_ptr(self); }
            let mutable = match segs.len() { 1 => false, 2 if segs[1] == "mut" => true, _ => return self.err("a pointer is `ptr[T]` or `ptr.mut[T]`") };
            if params.len() != 1 { return self.err("a pointer says what it points at: `ptr[uint8]`, `ptr.mut[Timeval]`"); }
            return Ok(Ty::Ptr(Box::new(params.pop().unwrap()), mutable));
        }
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
                let base = if head == "list" { Ty::List(Box::new(inner)) } else { Ty::Array(Box::new(inner)) };
                // `list[int32][0, 1000]` — a length bound, the thing then its bounds like `int32[0, 100]` (decided 2026-09-21)
                if *self.peek() == Tok::LBracket && matches!(self.peek_at(1), Tok::Int(_) | Tok::Bin(_)) {
                    if head == "array" { return bad(self, "an `array` has one fixed length, so a bound on it says nothing; a length bound belongs on a `list`".to_string()); }
                    self.advance();
                    let lo = self.bound()?;
                    self.expect(Tok::Comma, "`,` between the length bounds")?;
                    let hi = self.bound()?;
                    self.expect(Tok::RBracket, "`]` after the length bounds")?;
                    let (Bound::Int(a), Bound::Int(b)) = (&lo, &hi) else { return bad(self, "a length bound is a pair of whole numbers: `list[int32][0, 1000]`".to_string()) };
                    if *a < 0 { return bad(self, "a length is never negative: the low bound is 0 or more".to_string()); }
                    if a > b { return bad(self, "a range's low bound comes first: `list[int32][0, 1000]`".to_string()); }
                    return Ok(Ty::Ranged(Box::new(base), lo, hi));
                }
                Ok(base)
            }
            "int" | "uint" | "bin" | "deci" => bad(self, format!("the typed form is fully explicit: `{head}` needs a width (e.g. `{head}32`)")),
            _ if KEYWORDS.contains(&head) => bad(self, format!("`{head}` is a keyword, not a type")),
            _ => {
                if !params.is_empty() {
                    let builtin = ["int", "uint", "bin", "deci"].iter().any(|p| width(head, p).is_some());
                    if builtin { return bad(self, format!("`{head}` takes no type parameters")); }
                    // `Stack[int32]` — a generic struct or enum with its blanks filled
                    return Ok(Ty::Inst(self.canonical(&segs), params));
                }
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
                "ptr" => {
                    // ptr.write[…]; as a statement
                    let (head, op, args) = self.ptr_op()?;
                    self.expect(Tok::Semi, "`;`")?;
                    Ok(Stmt::BuiltinStmt { head, op, args, line })
                }
                "extern" | "layout" => self.err(format!("`{w}` sits at the top level of a file")),
                "asm" | "ASM" => self.err("inline assembly is `inline.ASM { … }` (decided 2026-09-20)"),
                "match" => {
                    // match v { case ['bind'] { … } … }   (design/enums.md, 2026-09-20)
                    self.advance();
                    let value = self.rhs()?;
                    if *self.peek() != Tok::LBrace { return self.err(format!("expected `{{` after the value to match, found {}", self.peek())); }
                    let arms = self.arms("match")?;
                    Ok(Stmt::Match { value, arms, line })
                }
                "enum" => self.err("an `enum` sits at the top level of a file"),
                "inline" => {
                    // nocheck inline.ASM { … } with [ … ];   (design/ffi.md, 2026-09-20)
                    // The language's name is in capitals, like MAIN: easy to spot (owner).
                    self.advance();
                    self.expect(Tok::Dot, "`.ASM` after `inline` (`inline.ASM { … }`)")?;
                    match self.advance().tok {
                        Tok::Word(l) if l == "ASM" => {}
                        Tok::Word(l) if l == "asm" => return self.err("the inline language is in capitals, like MAIN: `inline.ASM`"),
                        Tok::Word(l) => return self.err(format!("`inline.{l}` is not decided; only `inline.ASM` exists (`inline.C` is possible later — LLVM ships clang)")),
                        other => return self.err(format!("expected `ASM` after `inline.`, found {other}")),
                    }
                    let text = match self.advance().tok {
                        Tok::Raw(t) => t,
                        Tok::LBracket => return self.err("assembly goes in braces, verbatim: `inline.ASM { mov x16, #4 … }` — not a list of strings"),
                        other => return self.err(format!("expected `{{` after `inline.ASM`, found {other}")),
                    };
                    let mut binds = Vec::new();
                    if self.is_word("with") {
                        self.advance();
                        self.expect(Tok::LBracket, "`[` after `with`")?;
                        if *self.peek() != Tok::RBracket {
                            loop {
                                match self.advance().tok {
                                    Tok::Var(v) => {
                                        let dir = self.advance().tok;
                                        let reg = match self.advance().tok { Tok::Word(r) => r, other => return self.err(format!("expected a register name, found {other}")) };
                                        binds.push(match dir {
                                            Tok::Arrow => AsmBind::In(v, reg),
                                            Tok::LeftArrow => AsmBind::Out(v, reg),
                                            Tok::BothArrow => AsmBind::InOut(v, reg),
                                            other => return self.err(format!("after '{v}' comes an arrow: `-> reg` (in), `<- reg` (out), `<-> reg` (both); found {other}")),
                                        });
                                    }
                                    Tok::Word(w) if w == "clobber" => {
                                        let reg = match self.advance().tok { Tok::Word(r) => r, other => return self.err(format!("expected a register after `clobber`, found {other}")) };
                                        binds.push(AsmBind::Clobber(reg));
                                    }
                                    Tok::Word(w) if w == "in" || w == "out" || w == "inout" => return self.err(format!("no `{w}` word: the arrow says the direction — `'v' -> x0`, `'v' <- x0`, `'v' <-> x0`")),
                                    other => return self.err(format!("expected `'variable' -> register`, `'variable' <- register`, or `clobber register`, found {other}")),
                                }
                                if *self.peek() == Tok::Comma { self.advance(); continue; }
                                break;
                            }
                        }
                        self.expect(Tok::RBracket, "`]` to close the `with` list")?;
                    }
                    self.expect(Tok::Semi, "`;` after the asm block")?;
                    Ok(Stmt::Asm { text, binds, line })
                }
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
                if *self.peek() == Tok::LBracket && !matches!(self.peek_at(1), Tok::Word(w) if w == "pos") {
                    // 'f'[args];  — a function value called as a statement (2026-09-21)
                    let args = self.bracket()?;
                    let handler = self.handler()?;
                    self.expect(Tok::Semi, "`;` after the call")?;
                    return Ok(Stmt::CallValue { name, args, handler, line });
                }
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
                Expr::CharFromCode(_, _, h, _) => *h = self.handler()?,
                Expr::CallValue(_, _, h, _) => *h = self.handler()?,
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
                let arms = self.arms("otherwise")?;
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

    /// A range type's bound: a numeric literal, possibly negated.
    fn bound(&mut self) -> PResult<Bound> {
        let neg = if matches!(self.peek(), Tok::Op(o) if o == "-") { self.advance(); true } else { false };
        match self.advance().tok {
            Tok::Int(i) => Ok(Bound::Int(if neg { -i } else { i })),
            Tok::Bin(b) => Ok(Bound::Bin(if neg { -b } else { b })),
            other => self.err(format!("a range's bounds are numeric literals, found {other}")),
        }
    }

    /// `[ … ][` — in `to.int32[0, 100]['text']` the first bracket is the range, the second the value.
    fn bracket_then_bracket(&self) -> bool {
        let mut i = 0; let mut depth = 0;
        loop {
            match self.peek_at(i) {
                Tok::LBracket => depth += 1,
                Tok::RBracket => { depth -= 1; if depth == 0 { return *self.peek_at(i + 1) == Tok::LBracket; } }
                Tok::Eof => return false,
                _ => {}
            }
            i += 1;
        }
    }

    /// A lone Word (not a keyword, no bracket, no path) in value position — a function as a value.
    /// A dotted path without a bracket is an enum case (`is_enum_bare`) unless it ends in a
    /// lowercase segment after an uppercase one… the oracle resolves both at run time, so a
    /// single bare word is treated as a function reference and a dotted one as an enum case.
    fn is_fn_ref(&self) -> bool {
        matches!(self.peek(), Tok::Word(w) if !KEYWORDS.contains(&w.as_str()) && !OP_WORDS.contains(&w.as_str()))
            && !matches!(self.peek_at(1), Tok::LBracket | Tok::ColonColon | Tok::Dot)
    }

    /// The body of an anonymous function: `fn [params] -> T with [captures] { … }`
    fn anon_func(&mut self) -> PResult<Func> {
        let line = self.line();
        self.expect_word("fn")?;
        self.expect(Tok::LBracket, "`[` to open the parameter list")?;
        let mut params = Vec::new();
        if *self.peek() != Tok::RBracket {
            loop { params.push(self.param()?); if *self.peek() == Tok::Comma { self.advance(); continue; } break; }
        }
        self.expect(Tok::RBracket, "`]` to close the parameter list")?;
        let mut ret = None; let mut fails = Vec::new();
        if *self.peek() == Tok::Arrow {
            self.advance();
            if self.is_word("fail") { fails = self.fail_list()?; }
            else { ret = Some(self.ty()?); if self.is_word("or") { self.advance(); if !self.is_word("fail") { return self.err("after `or` comes `fail[…]`"); } fails = self.fail_list()?; } }
        }
        // captures: nothing is taken from the surrounding scope unless listed here, with a verb
        let mut caps = Vec::new();
        if self.is_word("with") {
            self.advance();
            self.expect(Tok::LBracket, "`[` after `with`: `with [lend 'limit', give 'name']`")?;
            if *self.peek() != Tok::RBracket {
                loop {
                    let verb = match self.advance().tok {
                        Tok::Word(w) if w == "lend" => { if *self.peek() == Tok::Dot { self.advance(); match self.advance().tok { Tok::Word(m) if m == "mut" => Verb::LendMut, other => return self.err(format!("the only narrowing of `lend` is `.mut`, found {other}")) } } else { Verb::Lend } }
                        Tok::Word(w) if w == "give" => Verb::Give,
                        Tok::Word(w) if w == "copy" => Verb::Copy,
                        Tok::Var(v) => return self.err(format!("a capture says how it is taken: `lend '{v}'`, `copy '{v}'`, or `give '{v}'` — nothing is captured silently")),
                        other => return self.err(format!("expected `lend`, `copy`, or `give` before a captured variable, found {other}")),
                    };
                    let v = match self.advance().tok { Tok::Var(v) => v, other => return self.err(format!("expected a quoted variable to capture, found {other}")) };
                    caps.push((verb, v));
                    if *self.peek() == Tok::Comma { self.advance(); continue; }
                    break;
                }
            }
            self.expect(Tok::RBracket, "`]` to close the captures")?;
        }
        let body = self.block()?;
        self.pending_captures = caps;
        let bounds = std::mem::take(&mut self.bounds);
        Ok(Func { name: String::new(), module: self.module.clone(), vis: Vis::Private, ret, fails, is_extern: false, is_op: false, bounds, params, body, line })
    }

    /// Word ('.' Word)+ not followed by '[' or '::'  — an enum case with no data
    fn is_enum_bare(&self) -> bool {
        let mut i = 0;
        let mut dots = 0;
        loop {
            if !matches!(self.peek_at(i), Tok::Word(_)) { return false; }
            match self.peek_at(i + 1) {
                Tok::Dot if matches!(self.peek_at(i + 2), Tok::Word(_)) => { i += 2; dots += 1; }
                Tok::Dot | Tok::LBracket | Tok::ColonColon => return false,   // a const path, a call, a Limb
                _ => return dots >= 1,
            }
        }
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

    // arms := '{' (Word Var? block)* '}'   — shared by `otherwise` and `match`
    fn arms(&mut self, what: &str) -> PResult<Vec<Arm>> {
        self.expect(Tok::LBrace, "`{`")?;
        let mut arms = Vec::new();
        while *self.peek() != Tok::RBrace {
            let line = self.line();
            let name = match self.advance().tok {
                Tok::Word(w) => w,
                Tok::Var(v) => return self.err(format!("an arm starts with the case's bare name; the data's name comes after it: `claimed {v} {{ … }}`")),
                Tok::Eof => return self.err(format!("unclosed `{what} {{`")),
                other => return self.err(format!("expected a case name to handle, found {other}")),
            };
            let bind = if let Tok::Var(v) = self.peek().clone() { self.advance(); Some(v) } else { None };
            let body = self.block()?;
            if arms.iter().any(|a: &Arm| a.name == name) { return self.err(format!("`{name}` is handled twice")); }
            arms.push(Arm { name, bind, body, line });
        }
        self.advance();
        Ok(arms)
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
            if limb.is_none() && path.len() >= 2 { return self.err(format!("`{shown}`: a call needs `[…]`; an enum case with no data is written bare (`Status.empty`) — is `{}` an enum?", path[path.len() - 2])); }
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

    // ptr op := 'ptr' '.' ('read' | 'write' | 'at' | 'is_null') bracket   (design/ffi.md)
    fn ptr_op(&mut self) -> PResult<(String, String, Bracket)> {
        self.expect_word("ptr")?;
        self.expect(Tok::Dot, "`.` after `ptr` (`ptr.read[…]`, `ptr.write[…]`, `ptr.at[…]`, `ptr.is_null[…]`)")?;
        let op = match self.advance().tok { Tok::Word(w) => w, other => return self.err(format!("expected an operation after `ptr.`, found {other}")) };
        let want = match op.as_str() { "read" => 1, "write" => 2, "at" => 2, "is_null" => 1, "mut" => return self.err("`ptr.mut[T]` is a type, for an extern signature; here use `ptr.read`, `ptr.write`, `ptr.at`, `ptr.is_null`"), _ => return self.err(format!("`ptr.{op}` does not exist; ptr has read, write, at, is_null")) };
        if *self.peek() != Tok::LBracket { return self.err(format!("`ptr.{op}` is a call and needs `[…]`")); }
        let args = self.bracket()?;
        if args.values.len() != want { return self.err(format!("`ptr.{op}` takes {want} argument(s), given {}", args.values.len())); }
        Ok(("ptr".into(), op, args))
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
            // `and.bit` / `or.bit` — one operator spelled as a chain
            Tok::Word(w) if (w == "and" || w == "or") && *self.peek_at(1) == Tok::Dot && matches!(self.peek_at(2), Tok::Word(b) if b == "bit") => Some(format!("{w}.bit")),
            Tok::Word(w) if OP_WORDS.contains(&w.as_str()) => Some(w.clone()),
            Tok::Word(w) if w == "xor" || w == "shl" || w == "shr" => Some(w.clone()),
            // a custom word operator (decided 2026-09-20): a bare non-keyword word between operands
            Tok::Word(w) if !KEYWORDS.contains(&w.as_str()) && w != "not" && *self.peek_at(1) != Tok::LBracket && *self.peek_at(1) != Tok::ColonColon && *self.peek_at(1) != Tok::Dot => Some(w.clone()),
            Tok::Eq => Some("=".into()),
            _ => None,
        }
    }

    fn is_custom_op(op: &str) -> bool { Parser::level(op) == 0 && op != "=" }

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
            "+" | "-" | "+~" | "-~" => 5,
            "x" | "/" | "mod" | "x~" => 6,
            "xx" | "xx~" => 8,
            _ => 0,
        }
    }

    fn climb(&mut self, min: u8) -> PResult<Expr> {
        let line = self.line();
        let mut lhs = self.unary()?;
        loop {
            let Some(op) = self.binop() else { break };
            if op == "=" { return self.err("`=` inside a value: assignment is a statement; equality is `==`"); }
            if Parser::is_custom_op(&op) {
                // A custom operator has no precedence (decided 2026-09-20): it may not
                // sit next to another operator without parentheses. Bitwise operators
                // follow the same rule (2026-09-21): mathematics has no reading for them.
                if min > 0 { break; }
                let what = if BIT_OPS.contains(&op.as_str()) { "a bitwise operator" } else { "a custom operator" };
                if matches!(lhs, Expr::Binary(..)) { return self.err(format!("`{op}` next to another operator: {what} has no precedence — parenthesise")); }
                self.advance();
                if op.ends_with(".bit") { self.advance(); self.advance(); }
                let rhs = self.unary()?;
                if let Some(next) = self.binop() { if next != "=" { return self.err(format!("`{next}` after `{op}`: {what} has no precedence — parenthesise")); } }
                lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs), line);
                continue;
            }
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
        if self.is_word("not") {
            self.advance();
            if *self.peek() == Tok::Dot && matches!(self.peek_at(1), Tok::Word(b) if b == "bit") {
                self.advance(); self.advance();
                let inner = self.primary()?;
                return Ok(Expr::Unary("not.bit".into(), Box::new(inner), line));
            }
            return Ok(Expr::Unary("not".into(), Box::new(self.unary()?), line));
        }
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
                if *self.peek() == Tok::LBracket && !matches!(self.peek_at(1), Tok::Word(w) if w == "pos") {
                    // 'f'[args] — calling a function value; an index always says `pos` (2026-09-21)
                    let args = self.bracket()?;
                    return Ok(Expr::CallValue(v, args, None, line));
                }
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
            Tok::Ellipsis => {
                // `...'xs'` — unpack a list into a call's variadic part (decided 2026-09-21)
                self.advance();
                // `...copy 'xs'` / `...give 'xs'`: how owned elements are handed over (decided 2026-09-21)
                let verb = match self.peek() {
                    Tok::Word(w) if w == "copy" => { self.advance(); Some(Verb::Copy) }
                    Tok::Word(w) if w == "give" => { self.advance(); Some(Verb::Give) }
                    Tok::Word(w) if w == "lend" => return self.err("`...'xs'` with no verb already lends its elements to a `lent ...` parameter; `copy` or `give` are the verbs that go after `...`"),
                    _ => None,
                };
                match self.advance().tok {
                    Tok::Var(v) => Ok(Expr::Spread(v, verb, line)),
                    other => self.err(format!("`...` unpacks a list variable into a call: `...'xs'`, found {other}")),
                }
            }
            Tok::Op(o) if o == "&" => {
                note_ampersand();
                // `&'s'` / `&mut 's'` — Rust's spelling, an alternative to `lend` / `lend.mut` (owner, 2026-09-21)
                self.advance();
                let verb = if self.is_word("mut") { self.advance(); Verb::LendMut } else { Verb::Lend };
                match self.advance().tok {
                    Tok::Var(v) => {
                        if *self.peek() == Tok::Dot {
                            self.advance();
                            let field = match self.advance().tok { Tok::Word(f) => f, other => return self.err(format!("expected a field name after `'{v}'.`, found {other}")) };
                            return Ok(Expr::OwnedField(verb, v, field, line));
                        }
                        if *self.peek() == Tok::LBracket { let idx = self.index_bracket(&v)?; return Ok(Expr::OwnedElem(verb, v, Box::new(idx), line)); }
                        Ok(Expr::Owned(verb, v, line))
                    }
                    other => self.err(format!("`&` lends a variable: `&'name'` or `&mut 'name'`, found {other}")),
                }
            }
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
                            if *self.peek() == Tok::LBracket {
                                let idx = self.index_bracket(&v)?;
                                return Ok(Expr::OwnedElem(verb, v, Box::new(idx), line));
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
                "ptr" => { let (h, o, a) = self.ptr_op()?; Ok(Expr::Builtin(h, o, a, line)) }
                "str" if matches!(self.peek_at(1), Tok::Dot) && !matches!(self.peek_at(2), Tok::Word(u) if u == "utf8" || u == "utf16" || u == "utf32" || u == "ascii") => {
                    // str.bytes[…] / str.points[…] / str.graphemes[…] / str.each.<unit>[…]  (decided 2026-09-20)
                    self.advance(); self.advance();
                    let op = match self.advance().tok { Tok::Word(w) => w, other => return self.err(format!("expected an operation after `str.`, found {other}")) };
                    let op = match op.as_str() {
                        "bytes" | "points" | "graphemes" => op,
                        "each" => {
                            self.expect(Tok::Dot, "`.byte`, `.point`, or `.grapheme` after `str.each`")?;
                            match self.advance().tok {
                                Tok::Word(u) if u == "byte" || u == "point" || u == "grapheme" => format!("each.{u}"),
                                other => return self.err(format!("`str.each` walks by a unit: `.byte`, `.point`, or `.grapheme`; found {other}")),
                            }
                        }
                        "length" | "len" | "count" => return self.err("a text has no single length: `str.bytes[…]`, `str.points[…]`, or `str.graphemes[…]` — say which (decided 2026-09-20)"),
                        "split" | "find" | "contains" | "trim" | "upper" | "lower" | "replace" | "starts_with" | "ends_with" | "at" | "slice" =>
                            return self.err(format!("`str.{op}` is std's: `std::str.{op}[…]` — the language owns only the units (bytes/points/graphemes) and `each`")),
                        _ => return self.err(format!("`str.{op}` does not exist; the language has str.bytes, str.points, str.graphemes, str.each.<unit>")),
                    };
                    if *self.peek() != Tok::LBracket { return self.err(format!("`str.{op}` is a call and needs `[…]`")); }
                    let args = self.bracket()?;
                    if args.values.len() != 1 { return self.err(format!("`str.{op}` takes one text: `str.{op}[lend 's']`")); }
                    Ok(Expr::Builtin("str".into(), op, args, line))
                }
                "char" if matches!(self.peek_at(1), Tok::Dot) && matches!(self.peek_at(2), Tok::Word(u) if u == "code" || ((u == "point" || u == "byte" || u == "grapheme") && *self.peek_at(3) == Tok::Dot)) => {
                    // char.code[c] — the number under a char.point / char.byte;
                    // char.point.from_code[n] / char.byte.from_code[n] — back, failing not_a_char (decided 2026-09-21)
                    self.advance(); self.advance();
                    let w = match self.advance().tok { Tok::Word(w) => w, _ => unreachable!() };
                    if w == "code" {
                        if *self.peek() != Tok::LBracket { return self.err("`char.code` is a call and needs `[…]`: `char.code[`A`]`"); }
                        let args = self.bracket()?;
                        if args.values.len() != 1 { return self.err("`char.code` takes one character"); }
                        return Ok(Expr::Builtin("char".into(), "code".into(), args, line));
                    }
                    self.advance(); // the dot
                    match self.advance().tok { Tok::Word(f) if f == "from_code" => {}, other => return self.err(format!("`char.{w}.` is followed by `from_code[…]`, found {other}")) }
                    let unit = match w.as_str() { "point" => CharUnit::Point, "byte" => CharUnit::Byte, _ => return self.err("a grapheme has no single code; build it from text: `to.char.grapheme[\"…\"]`") };
                    if *self.peek() != Tok::LBracket { return self.err(format!("`char.{w}.from_code` is a call and needs `[…]`")); }
                    let args = self.bracket()?;
                    if args.values.len() != 1 { return self.err(format!("`char.{w}.from_code` takes one number")); }
                    Ok(Expr::CharFromCode(unit, args, None, line))
                }
                "size" | "align" => {
                    // size.of[T] / align.of[T]   (design/ffi.md)
                    self.advance();
                    self.expect(Tok::Dot, &format!("`.of` after `{w}` (`{w}.of[int32]`)"))?;
                    match self.advance().tok { Tok::Word(o) if o == "of" => {}, other => return self.err(format!("`{w}.of[…]` is the only form, found {other}")) }
                    self.expect(Tok::LBracket, "`[`")?;
                    let saved = self.at_boundary; self.at_boundary = true;
                    let ty = self.ty();
                    self.at_boundary = saved;
                    let ty = ty?;
                    self.expect(Tok::RBracket, "`]`")?;
                    Ok(Expr::SizeOf(ty, w == "size", line))
                }
                "to" => {
                    // to.<type>[value]   (decided 2026-09-19)
                    self.advance();
                    self.expect(Tok::Dot, "`.` after `to` (`to.int32[…]`, `to.str.utf8[…]`)")?;
                    self.to_target = true;
                    let ty = self.ty();
                    self.to_target = false;
                    let ty = ty?;
                    if matches!(ty, Ty::List(_) | Ty::Array(_) | Ty::Struct(_) | Ty::Inst(..) | Ty::Var(_)) { return self.err(format!("no conversion to {}; conversions are between numbers and text", ty_name(&ty))); }
                    if matches!(ty, Ty::Ranged(..)) && *self.peek() != Tok::LBracket { return self.err(format!("`to.{}` converts a value: `to.{}[…]`", ty_name(&ty), ty_name(&ty))); }
                    if *self.peek() != Tok::LBracket { return self.err(format!("`to.{}` converts a value: `to.{}[…]`", ty_name(&ty), ty_name(&ty))); }
                    let b = self.bracket()?;
                    if b.values.len() != 1 { return self.err("`to.…[…]` converts exactly one value"); }
                    Ok(Expr::Convert(ty, b, None, line))
                }
                "true" => { self.advance(); Ok(Expr::Bool(true, line)) }
                "false" => { self.advance(); Ok(Expr::Bool(false, line)) }
                "limb" if !matches!(self.peek_at(1), Tok::Dot) => self.err("`limb` is the root of this Limb and is followed by `.`: `limb.helper[]`, `limb.'pi'`"),
                "fn" => {
                    // fn [params] -> T with [lend 'v', give 'w'] { … } — an anonymous function (2026-09-21)
                    let saved_bounds = std::mem::take(&mut self.bounds);
                    // reuse func(): give it a name it can never collide with
                    let mut f = self.anon_func()?;
                    self.bounds = saved_bounds;
                    f.name = format!("<anonymous at line {line}>");
                    let caps = std::mem::take(&mut self.pending_captures);
                    Ok(Expr::Lambda(std::rc::Rc::new(f), caps, line))
                }
                _ if KEYWORDS.contains(&w.as_str()) && w != "limb" => self.err(format!("`{w}` is a keyword and cannot appear inside a value")),
                _ if self.is_fn_ref() => {
                    // a bare function name with no brackets: the function as a value
                    let mut segs = Vec::new();
                    loop {
                        match self.advance().tok { Tok::Word(w) => segs.push(w), _ => unreachable!() }
                        if *self.peek() == Tok::Dot && matches!(self.peek_at(1), Tok::Word(_)) { self.advance(); continue; }
                        break;
                    }
                    Ok(Expr::FnRef(self.resolve(&segs), self.module.clone(), line))
                }
                _ if self.is_enum_bare() => {
                    // `Status.empty` / `shapes.Status.empty` — an enum case with no data;
                    // `Status.claimed["x"]` goes through call() and resolves at run time
                    let mut segs = Vec::new();
                    loop {
                        match self.advance().tok { Tok::Word(w) => segs.push(w), _ => unreachable!() }
                        if *self.peek() == Tok::Dot && matches!(self.peek_at(1), Tok::Word(_)) { self.advance(); continue; }
                        break;
                    }
                    let case = segs.pop().unwrap();
                    Ok(Expr::EnumValue(self.canonical(&segs), case, None, line))
                }
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

fn bad_ptr<T>(p: &Parser) -> PResult<T> {
    p.err("`ptr[T]` exists only at the boundary: in an `extern fn` signature or a `layout.c struct` field (design/ffi.md); a pointer is held with `:=`, never declared")
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
