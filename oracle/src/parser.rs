//! Recursive descent over the token list. Grammar is in the comments of each fn.

use crate::ast::*;
use crate::lexer::{Tok, Token};

pub const KEYWORDS: &[&str] = &[
    "func", "var", "return", "if", "else", "loop", "in", "break", "continue",
    "give", "copy", "lend", "lent", "op", "check", "nocheck", "true", "false", "mut", "immut",
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

    // program := (func | MAIN block)* Eof
    pub fn program(&mut self) -> PResult<Program> {
        let mut funcs = Vec::new();
        let mut main = None;
        loop {
            match self.peek().clone() {
                Tok::Eof => break,
                Tok::Word(w) if w == "func" => funcs.push(self.func()?),
                Tok::Word(w) if w == "MAIN" => {
                    if main.is_some() { return self.err("a program has exactly one MAIN"); }
                    self.advance();
                    if *self.peek() == Tok::LBracket { return self.err("MAIN is a block, not a function: `MAIN { … }`, no parameter list"); }
                    main = Some(self.block()?);
                }
                Tok::Word(w) if w == "op" => return self.err("custom operators (`op`) are not decided yet; the oracle refuses them"),
                Tok::Word(w) if w == "var" => return self.err("a `var` cannot sit at the top level; it goes inside MAIN or a func"),
                other => return self.err(format!("expected `func` or `MAIN` at the top level, found {other}")),
            }
        }
        Ok(Program { funcs, main })
    }

    // func := 'func' ('.' type)? Word '[' params ']' block
    fn func(&mut self) -> PResult<Func> {
        let line = self.line();
        self.expect_word("func")?;
        let ret = if *self.peek() == Tok::Dot { self.advance(); Some(self.ty()?) } else { None };
        let name = match self.advance().tok {
            Tok::Word(w) => w,
            Tok::Var(v) => return self.err(format!("function names are bare: `func {v}`, not `func '{v}'` (quotes mean variable)")),
            other => return self.err(format!("expected a function name, found {other}")),
        };
        if KEYWORDS.contains(&name.as_str()) { return self.err(format!("`{name}` is a keyword and cannot name a function")); }
        if name == "MAIN" { return self.err("MAIN is the program's block, not a function: `MAIN { … }`"); }
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
        let body = self.block()?;
        Ok(Func { name, ret, params, body, line })
    }

    // param := ('var' | 'lent' | 'lent' '.' 'mut') ('.' type)? Var
    fn param(&mut self) -> PResult<Param> {
        let line = self.line();
        let kind = match self.peek().clone() {
            Tok::Word(w) if w == "var" => { self.advance(); ParamKind::Var }
            Tok::Word(w) if w == "lent" => {
                self.advance();
                if *self.peek() == Tok::Dot && matches!(self.peek_at(1), Tok::Word(m) if m == "mut") {
                    self.advance(); self.advance(); ParamKind::LentMut
                } else { ParamKind::Lent }
            }
            Tok::Word(w) if w == "lend" => return self.err("a parameter is `lent` (what it receives); `lend` is what the caller does"),
            Tok::Var(v) => return self.err(format!("a parameter says how it is received: `var.T '{v}'`, `lent.T '{v}'`, or `lent.mut.T '{v}'`")),
            other => return self.err(format!("expected a parameter, found {other}")),
        };
        let (mutability, ty) = if kind == ParamKind::Var { self.mutability_then_type()? } else {
            (None, if *self.peek() == Tok::Dot { self.advance(); Some(self.ty()?) } else { None })
        };
        let name = match self.advance().tok {
            Tok::Var(v) => v,
            other => return self.err(format!("expected a quoted parameter name, found {other}")),
        };
        Ok(Param { kind, mutability, ty, name, line })
    }

    // ('.' ('mut' | 'immut'))? ('.' type)?   — after `var` or `lent`
    fn mutability_then_type(&mut self) -> PResult<(Option<Mutability>, Option<Ty>)> {
        let mut m = None;
        if *self.peek() == Tok::Dot {
            match self.peek_at(1) {
                Tok::Word(w) if w == "mut" => { self.advance(); self.advance(); m = Some(Mutability::Mut); }
                Tok::Word(w) if w == "immut" => { self.advance(); self.advance(); m = Some(Mutability::Immut); }
                _ => {}
            }
        }
        let ty = if *self.peek() == Tok::Dot { self.advance(); Some(self.ty()?) } else { None };
        Ok((m, ty))
    }

    // type := segments joined by '.', already positioned after the leading '.'
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
            "int" | "uint" | "bin" | "deci" => bad(self, format!("the typed form is fully explicit: `{head}` needs a width (e.g. `{head}32`)")),
            _ => {
                if segs.len() > 1 { return bad(self, format!("`{}` takes no narrowing", segs.join("."))); }
                if let Some(n) = width(head, "int") { return int_w(n).map(Ty::Int).ok_or(()).or_else(|_| bad(self, format!("no such width `{head}`; int is 8/16/32/64/128"))); }
                if let Some(n) = width(head, "uint") { return int_w(n).map(Ty::Uint).ok_or(()).or_else(|_| bad(self, format!("no such width `{head}`; uint is 8/16/32/64/128"))); }
                if let Some(n) = width(head, "bin") { return if [16, 32, 64, 128].contains(&n) { Ok(Ty::Bin(n as u8)) } else { bad(self, format!("no such width `{head}`; bin is 16/32/64/128")) }; }
                if let Some(n) = width(head, "deci") { return if [32, 64, 128].contains(&n) { Ok(Ty::Deci(n as u8)) } else { bad(self, format!("no such width `{head}`; deci is 32/64/128 (IEEE 754 defines only these)")) }; }
                bad(self, format!("unknown type `{head}`"))
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
                "var" => self.decl(),
                "if" => self.if_stmt(),
                "loop" => self.loop_stmt(),
                "break" => { self.advance(); self.expect(Tok::Semi, "`;`")?; Ok(Stmt::Break { line }) }
                "continue" => { self.advance(); self.expect(Tok::Semi, "`;`")?; Ok(Stmt::Continue { line }) }
                "return" => {
                    self.advance();
                    let value = if *self.peek() == Tok::LBracket { Some(self.bracket()?) } else { None };
                    self.expect(Tok::Semi, "`;` after return")?;
                    Ok(Stmt::Return { value, line })
                }
                "func" => self.err("a `func` cannot be defined inside a body; it goes at the top level"),
                "MAIN" => self.err("MAIN cannot be nested or called"),
                "else" => self.err("`else` without an `if`"),
                "check" | "nocheck" => self.err("the check tiers are decided but their syntax is not; the oracle refuses them for now"),
                "give" | "copy" | "lend" => self.err(format!("`{w}` belongs inside a value: `'s' = [{w} 't'];`")),
                "while" | "for" if matches!(self.peek_at(1), Tok::LBracket | Tok::Var(_)) => self.err(format!("loops are narrowed from `loop`: `loop.{w}`")),
                _ => {
                    let call = self.call()?;
                    self.expect(Tok::Semi, "`;` after the call")?;
                    Ok(Stmt::Call { call, line })
                }
            },
            Tok::Var(name) => {
                self.advance();
                match self.peek() {
                    Tok::Eq => {}
                    Tok::Walrus | Tok::ShadowWalrus | Tok::ShadowEq => return self.err(format!("`:=` declares; to assign to the existing '{name}' write `'{name}' = […];` (to shadow it, `var '{name}' $:= […];`)")),
                    Tok::LBracket => return self.err(format!("'{name}'[…] — indexing is not designed yet, and a call is a bare name (`{name}[…]`)")),
                    other => return self.err(format!("expected `=` after '{name}', found {other}")),
                }
                self.advance();
                let value = self.bracket()?;
                self.expect(Tok::Semi, "`;`")?;
                Ok(Stmt::Assign { name, value, line })
            }
            other => self.err(format!("expected a statement, found {other}")),
        }
    }

    // decl := 'var' ('.' type)? Var (':=' | '=') bracket ';'
    fn decl(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.expect_word("var")?;
        let (mutability, ty) = self.mutability_then_type()?;
        let name = match self.advance().tok {
            Tok::Var(v) => v,
            Tok::Word(w) => return self.err(format!("variable names are quoted: `var '{w}'`")),
            other => return self.err(format!("expected a quoted variable name, found {other}")),
        };
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
        let value = self.bracket()?;
        self.expect(Tok::Semi, "`;`")?;
        Ok(Stmt::Decl { name, mutability, ty, value, shadow, line })
    }

    // if := 'if' bracket block ('else' '.' 'if' bracket block)* ('else' block)?
    fn if_stmt(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.expect_word("if")?;
        let mut arms = vec![(self.bracket()?, self.block()?)];
        let mut otherwise = None;
        while self.is_word("else") {
            self.advance();
            if *self.peek() == Tok::Dot {
                self.advance();
                self.expect_word("if")?;
                arms.push((self.bracket()?, self.block()?));
            } else if self.is_word("if") {
                return self.err("`else if` is written as a chain: `else.if`");
            } else {
                otherwise = Some(self.block()?);
                break;
            }
        }
        Ok(Stmt::If { arms, otherwise, line })
    }

    // loop := 'loop' block | 'loop' '.' 'while' bracket block
    //       | 'loop' '.' 'for' ('.' 'seq')? Var 'in' bracket block
    fn loop_stmt(&mut self) -> PResult<Stmt> {
        let line = self.line();
        self.expect_word("loop")?;
        if *self.peek() != Tok::Dot {
            return Ok(Stmt::Loop { body: self.block()?, line });
        }
        self.advance();
        let kind = match self.advance().tok { Tok::Word(w) => w, other => return self.err(format!("expected `while` or `for` after `loop.`, found {other}")) };
        match kind.as_str() {
            "while" => Ok(Stmt::While { cond: self.bracket()?, body: self.block()?, line }),
            "for" => {
                let mut seq = false;
                if *self.peek() == Tok::Dot {
                    self.advance();
                    match self.advance().tok {
                        Tok::Word(w) if w == "seq" => seq = true,
                        other => return self.err(format!("the only narrowing of `loop.for` is `.seq`, found {other}")),
                    }
                }
                let name = match self.advance().tok { Tok::Var(v) => v, other => return self.err(format!("expected a quoted loop variable, found {other}")) };
                self.expect_word("in")?;
                let over = self.bracket()?;
                let body = self.block()?;
                Ok(Stmt::For { name, seq, over, body, line })
            }
            other => self.err(format!("`loop.{other}` does not exist; loops are `loop`, `loop.while`, `loop.for`")),
        }
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

    // bracket := '[' (value (',' value)*)? ']'
    // Decided 2026-09-14: every value position is `[ ]`, a call included —
    // `var 'n' := [count[lend 'words']];`, never `:= count[...]`.
    fn bracket(&mut self) -> PResult<Bracket> {
        let line = self.line();
        if let Tok::Word(w) = self.peek().clone() {
            if !KEYWORDS.contains(&w.as_str()) && !OP_WORDS.contains(&w.as_str()) {
                return self.err(format!("a value position is always `[ ]`, a call included: `[{w}[…]]`"));
            }
        }
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
            Tok::Var(v) => { self.advance(); Ok(Expr::Var(v, line)) }
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
                        Tok::Var(v) => Ok(Expr::Owned(verb, v, line)),
                        other => self.err(format!("`{w}` takes a variable: `{w} 'name'`, found {other}")),
                    }
                }
                "lent" => self.err("`lent` describes a parameter; a caller writes `lend`"),
                "true" => { self.advance(); Ok(Expr::Bool(true, line)) }
                "false" => { self.advance(); Ok(Expr::Bool(false, line)) }
                _ if KEYWORDS.contains(&w.as_str()) => self.err(format!("`{w}` is a keyword and cannot appear inside a value")),
                _ => Ok(Expr::Call(self.call()?)),
            },
            Tok::Eof => self.err("unexpected end of file inside a value"),
            other => self.err(format!("expected a value, found {other}")),
        }
    }
}
