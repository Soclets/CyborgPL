//! The generated program as a tree, and how it is written out as CyborgPL.
//! Only what the generator writes is modelled; the spelling follows design/.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ty {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    Bool,
}

use std::sync::atomic::{AtomicBool, Ordering};

/// While set, the generator treats `uint64` and `int64` as if they stopped
/// at 2^63 − 1 (and −(2^63 − 1)), so no value reaches the range CCA is known
/// to mishandle (finding sent 2026-09-23). Only the generator's own view.
pub static NARROW_64: AtomicBool = AtomicBool::new(false);

/// While set, a printed value is passed as `to.int64[…]` / `to.uint64[…]`,
/// so no literal group in it can take the printer parameter's width instead
/// of its neighbour's — a question open with the owner (2026-09-23).
pub static PRINT_THROUGH_TO: AtomicBool = AtomicBool::new(false);

pub const INTS: [Ty; 8] = [Ty::I8, Ty::I16, Ty::I32, Ty::I64, Ty::U8, Ty::U16, Ty::U32, Ty::U64];

impl Ty {
    pub fn name(self) -> &'static str {
        match self {
            Ty::I8 => "int8",
            Ty::I16 => "int16",
            Ty::I32 => "int32",
            Ty::I64 => "int64",
            Ty::U8 => "uint8",
            Ty::U16 => "uint16",
            Ty::U32 => "uint32",
            Ty::U64 => "uint64",
            Ty::Bool => "bool",
        }
    }
    pub fn bits(self) -> u32 {
        match self {
            Ty::I8 | Ty::U8 => 8,
            Ty::I16 | Ty::U16 => 16,
            Ty::I32 | Ty::U32 => 32,
            Ty::I64 | Ty::U64 => 64,
            Ty::Bool => 1,
        }
    }
    pub fn signed(self) -> bool {
        matches!(self, Ty::I8 | Ty::I16 | Ty::I32 | Ty::I64)
    }
    pub fn min(self) -> i128 {
        if self == Ty::I64 && NARROW_64.load(Ordering::Relaxed) {
            return -(1i128 << 63) + 1;
        }
        if self.signed() {
            -(1i128 << (self.bits() - 1))
        } else {
            0
        }
    }
    pub fn max(self) -> i128 {
        if self == Ty::U64 && NARROW_64.load(Ordering::Relaxed) {
            return (1i128 << 63) - 1;
        }
        if self.signed() {
            (1i128 << (self.bits() - 1)) - 1
        } else {
            (1i128 << self.bits()) - 1
        }
    }
    pub fn range(self) -> Iv {
        Iv::new(self.min(), self.max())
    }
}

/// A declared type: a base type, and for a range type its bounds
/// (`int32[0, 100]`, `int32[1, max]`, `int32[min, 100]`; design/input.md).
/// `None` in a bound is the base type's own `min` / `max`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TyR {
    pub base: Ty,
    pub range: Option<(Option<i128>, Option<i128>)>,
}

impl From<Ty> for TyR {
    fn from(base: Ty) -> TyR {
        TyR { base, range: None }
    }
}

impl TyR {
    pub fn text(self) -> String {
        match self.range {
            None => self.base.name().to_string(),
            Some((lo, hi)) => {
                let lo = lo.map(|v| v.to_string()).unwrap_or("min".into());
                let hi = hi.map(|v| v.to_string()).unwrap_or("max".into());
                format!("{}[{lo}, {hi}]", self.base.name())
            }
        }
    }
}

/// A closed interval of integers. The generator's own bookkeeping of what a
/// value can be — never shown to either engine.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Iv {
    pub lo: i128,
    pub hi: i128,
}

impl Iv {
    pub fn new(lo: i128, hi: i128) -> Iv {
        assert!(lo <= hi);
        Iv { lo, hi }
    }
    pub fn point(v: i128) -> Iv {
        Iv { lo: v, hi: v }
    }
    pub fn within(self, o: Iv) -> bool {
        self.lo >= o.lo && self.hi <= o.hi
    }
    pub fn has(self, v: i128) -> bool {
        self.lo <= v && v <= self.hi
    }
    pub fn width(self) -> i128 {
        self.hi - self.lo + 1
    }
    pub fn meet(self, o: Iv) -> Option<Iv> {
        let lo = self.lo.max(o.lo);
        let hi = self.hi.min(o.hi);
        (lo <= hi).then_some(Iv { lo, hi })
    }
    pub fn join(self, o: Iv) -> Iv {
        Iv { lo: self.lo.min(o.lo), hi: self.hi.max(o.hi) }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    AndBit,
    OrBit,
    Xor,
    Shl,
    Shr,
    WAdd,
    WSub,
    WMul,
}

impl BinOp {
    pub fn text(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "x",
            BinOp::Div => "div",
            BinOp::Mod => "mod",
            BinOp::Pow => "xx",
            BinOp::AndBit => "and.bit",
            BinOp::OrBit => "or.bit",
            BinOp::Xor => "xor",
            BinOp::Shl => "shl",
            BinOp::Shr => "shr",
            BinOp::WAdd => "+~",
            BinOp::WSub => "-~",
            BinOp::WMul => "x~",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

impl CmpOp {
    pub fn text(self) -> &'static str {
        match self {
            CmpOp::Eq => "==",
            CmpOp::Ne => "!==",
            CmpOp::Lt => "<",
            CmpOp::Gt => ">",
            CmpOp::Le => "</==",
            CmpOp::Ge => ">/==",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// An integer literal; its type comes from its context.
    Lit(i128),
    Bool(bool),
    Var(String),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    Neg(Box<Expr>),
    NotBit(Box<Expr>),
    Not(Box<Expr>),
    Cmp(CmpOp, Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    To(Ty, Box<Expr>),
    Call(String, Vec<Expr>),
    /// `'xs'[p]`: the item at a 1-based position (a check site).
    At(String, Box<Expr>),
    /// `list.length[lend 'xs']`, a `uint64`.
    Len(String),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tier {
    Default,
    Check,
    Nocheck,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mutability {
    Plain,
    Mut,
    Immut,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Let { tier: Tier, name: String, mutability: Mutability, ty: TyR, value: Expr },
    Assign { tier: Tier, name: String, value: Expr },
    If { arms: Vec<(Expr, Vec<Stmt>)>, otherwise: Option<Vec<Stmt>> },
    For { perm: bool, name: String, ty: Ty, start: Expr, stop: Expr, step: Option<i128>, body: Vec<Stmt> },
    While { cond: Expr, body: Vec<Stmt> },
    Loop { body: Vec<Stmt> },
    Block { tier: Tier, body: Vec<Stmt> },
    Break,
    Continue,
    /// Print one integer through the program's own printer.
    Print { signed: bool, value: Expr },
    Return(Expr),
    /// `mut 'xs': list[T] = [a, b];` (design/lists.md).
    ListLet { name: String, mutable: bool, elem: Ty, items: Vec<Expr> },
    /// `change 'xs'[p] = v;`
    SetAt { tier: Tier, name: String, pos: Expr, value: Expr },
    /// `list.append[lend.mut 'xs', v];`
    Append { name: String, value: Expr },
    /// `for 'x' in lend 'xs'`, or `for 'i', 'x' in lend 'xs'` with the
    /// position from 1 (owner, c7951d4).
    ForEach { pos: Option<String>, item: String, list: String, body: Vec<Stmt> },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: TyR,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Func {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: TyR,
    pub body: Vec<Stmt>,
}

/// One outside value: `'name'` is the length of environment variable `var`.
#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    pub name: String,
    pub var: String,
    /// In exhaustive mode: the input is only ever used as `'in' mod domain`,
    /// so lengths 0..domain-1 are every value the program can see.
    pub domain: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub seed: u64,
    pub printer: String,
    pub inputs: Vec<Input>,
    pub funcs: Vec<Func>,
    pub main: Vec<Stmt>,
    /// Omit parentheses where design/operators.md says they are not needed.
    pub loose: bool,
}

// ---------------------------------------------------------------- rendering

fn q(name: &str) -> String {
    format!("'{name}'")
}

/// How tightly an expression binds, for the parenthesis rules of
/// design/operators.md. Lower is tighter. `None` means "always parenthesise
/// when it is an operand" (mod, bitwise, wrapping: no settled reading).
fn level(e: &Expr) -> Option<u8> {
    match e {
        Expr::Lit(v) if *v < 0 => Some(2),
        Expr::Lit(_) | Expr::Bool(_) | Expr::Var(_) | Expr::To(..) | Expr::Call(..) | Expr::At(..) | Expr::Len(_) => Some(0),
        Expr::Bin(BinOp::Pow, ..) => Some(1),
        Expr::Neg(_) => Some(2),
        Expr::Bin(BinOp::Mul | BinOp::Div, ..) => Some(3),
        Expr::Bin(BinOp::Add | BinOp::Sub, ..) => Some(4),
        Expr::Cmp(..) => Some(5),
        Expr::Not(_) => Some(6),
        Expr::And(..) => Some(7),
        Expr::Or(..) => Some(8),
        _ => None,
    }
}

/// Whether `child`, as the `left` (or right) operand of `parent`, may go
/// without parentheses. Only ever says yes where operators.md settles the
/// reading and the reading is the tree's.
fn bare_ok(parent: &Expr, child: &Expr, left: bool) -> bool {
    let Some(c) = level(child) else { return false };
    if c == 0 {
        return true;
    }
    match parent {
        Expr::Bin(BinOp::Add | BinOp::Sub, ..) => match child {
            Expr::Bin(BinOp::Add | BinOp::Sub, ..) => left,
            Expr::Bin(BinOp::Mul | BinOp::Div | BinOp::Pow, ..) => true,
            _ => false,
        },
        // `x` chains freely, but only the left chain is the tree's grouping;
        // `/` beside any multiplicative operator needs parentheses.
        Expr::Bin(BinOp::Mul, ..) => match child {
            Expr::Bin(BinOp::Mul, ..) => left,
            Expr::Bin(BinOp::Pow, ..) => true,
            _ => false,
        },
        Expr::Bin(BinOp::Div, ..) => matches!(child, Expr::Bin(BinOp::Pow, ..)),
        Expr::Bin(BinOp::Pow, ..) => !left && matches!(child, Expr::Bin(BinOp::Pow, ..)),
        Expr::Neg(_) => matches!(child, Expr::Bin(BinOp::Pow, ..)),
        Expr::Cmp(..) => matches!(child, Expr::Bin(BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Pow, ..) | Expr::Neg(_)),
        Expr::And(..) => matches!(child, Expr::Cmp(..) | Expr::Not(_)) || (left && matches!(child, Expr::And(..))),
        Expr::Or(..) => matches!(child, Expr::Cmp(..) | Expr::Not(_) | Expr::And(..)) || (left && matches!(child, Expr::Or(..))),
        _ => false,
    }
}

fn tier_prefix(t: Tier) -> &'static str {
    match t {
        Tier::Default => "",
        Tier::Check => "check ",
        Tier::Nocheck => "nocheck ",
    }
}

impl Program {
    pub fn render(&self) -> String {
        self.render_spans().0
    }
}

// ---------------------------------------------------------------- spans

/// Where every expression and simple statement sits in the rendered text,
/// keyed by the node's address (stable while the program is not changed):
/// 1-based line, 1-based start column, end column one past the last
/// character — the convention of `cca precog --json`. An expression's span
/// includes the parentheses its parent put around it. A statement has two
/// starts: with its `check`/`nocheck` word and without it.
#[derive(Default, Debug)]
pub struct Spans {
    pub expr: std::collections::HashMap<usize, (u32, u32, u32)>,
    pub stmt: std::collections::HashMap<usize, (u32, u32, u32, u32)>,
}

pub fn addr<T>(x: &T) -> usize {
    x as *const T as usize
}

struct Wr<'a> {
    out: String,
    loose: bool,
    /// byte offsets while writing
    expr: Vec<(usize, usize, usize)>,
    stmt: Vec<(usize, usize, usize, usize)>,
    _p: std::marker::PhantomData<&'a ()>,
}

impl<'a> Wr<'a> {
    fn e(&mut self, e: &Expr) {
        let start = self.out.len();
        self.inner(e);
        self.expr.push((addr(e), start, self.out.len()));
    }
    fn operand(&mut self, parent: &Expr, child: &Expr, left: bool) {
        let bare = level(child) == Some(0) || (self.loose && bare_ok(parent, child, left));
        if bare {
            self.e(child);
        } else {
            let s = self.out.len();
            self.out.push('(');
            self.e(child);
            self.out.push(')');
            // the parentheses belong to the child's span
            self.expr.push((addr(child), s, self.out.len()));
        }
    }
    fn inner(&mut self, e: &Expr) {
        match e {
            Expr::Lit(v) => self.out.push_str(&v.to_string()),
            Expr::Bool(b) => self.out.push_str(&b.to_string()),
            Expr::Var(n) => self.out.push_str(&q(n)),
            Expr::Bin(op, a, b) => {
                self.operand(e, a, true);
                self.out.push_str(&format!(" {} ", op.text()));
                self.operand(e, b, false);
            }
            Expr::Neg(a) => {
                self.out.push('-');
                self.operand(e, a, false);
            }
            Expr::NotBit(a) => {
                self.out.push_str("not.bit ");
                self.operand(e, a, false);
            }
            Expr::Not(a) => {
                self.out.push_str("not ");
                self.operand(e, a, false);
            }
            Expr::Cmp(op, a, b) => {
                self.operand(e, a, true);
                self.out.push_str(&format!(" {} ", op.text()));
                self.operand(e, b, false);
            }
            Expr::And(a, b) => {
                self.operand(e, a, true);
                self.out.push_str(" and ");
                self.operand(e, b, false);
            }
            Expr::Or(a, b) => {
                self.operand(e, a, true);
                self.out.push_str(" or ");
                self.operand(e, b, false);
            }
            Expr::To(t, a) => {
                self.out.push_str(&format!("to.{}[", t.name()));
                self.e(a);
                self.out.push(']');
            }
            Expr::Call(f, args) => {
                self.out.push_str(f);
                self.out.push('[');
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        self.out.push_str(", ");
                    }
                    self.e(a);
                }
                self.out.push(']');
            }
            Expr::At(n, p) => {
                self.out.push_str(&q(n));
                self.out.push('[');
                self.e(p);
                self.out.push(']');
            }
            Expr::Len(n) => self.out.push_str(&format!("list.length[lend {}]", q(n))),
        }
    }
    fn block(&mut self, body: &[Stmt], indent: usize) {
        for s in body {
            self.s(s, indent);
        }
    }
    /// A simple statement: `head`, then the expression, then `tail`.
    fn simple(&mut self, s: &Stmt, pad: &str, tier: Tier, head: &str, e: &Expr, tail: &str) {
        self.out.push_str(pad);
        let s0 = self.out.len();
        self.out.push_str(tier_prefix(tier));
        let s1 = self.out.len();
        self.out.push_str(head);
        self.e(e);
        self.out.push_str(tail);
        self.stmt.push((addr(s), s0, s1, self.out.len()));
        self.out.push('\n');
    }
    fn s(&mut self, s: &Stmt, indent: usize) {
        let pad = "    ".repeat(indent);
        match s {
            Stmt::Let { tier, name, mutability, ty, value } => {
                let m = match mutability {
                    Mutability::Plain => "",
                    Mutability::Mut => "mut ",
                    Mutability::Immut => "immut ",
                };
                let head = format!("{m}{}: {} = ", q(name), ty.text());
                self.simple(s, &pad, *tier, &head, value, ";");
            }
            Stmt::Assign { tier, name, value } => {
                let head = format!("change {} = ", q(name));
                self.simple(s, &pad, *tier, &head, value, ";");
            }
            Stmt::If { arms, otherwise } => {
                for (i, (c, body)) in arms.iter().enumerate() {
                    self.out.push_str(&pad);
                    self.out.push_str(if i == 0 { "if " } else { "} else if " });
                    self.e(c);
                    self.out.push_str(" {\n");
                    self.block(body, indent + 1);
                }
                if let Some(body) = otherwise {
                    self.out.push_str(&format!("{pad}}} else {{\n"));
                    self.block(body, indent + 1);
                }
                self.out.push_str(&format!("{pad}}}\n"));
            }
            Stmt::For { perm, name, ty, start, stop, step, body } => {
                let perm = if *perm { "perm " } else { "" };
                self.out.push_str(&format!("{pad}for {perm}{}: {} in range[", q(name), ty.name()));
                self.e(start);
                self.out.push_str(", ");
                self.e(stop);
                if let Some(st) = step {
                    self.out.push_str(&format!(", {st}"));
                }
                self.out.push_str("] {\n");
                self.block(body, indent + 1);
                self.out.push_str(&format!("{pad}}}\n"));
            }
            Stmt::While { cond, body } => {
                self.out.push_str(&format!("{pad}while "));
                self.e(cond);
                self.out.push_str(" {\n");
                self.block(body, indent + 1);
                self.out.push_str(&format!("{pad}}}\n"));
            }
            Stmt::Loop { body } => {
                self.out.push_str(&format!("{pad}loop {{\n"));
                self.block(body, indent + 1);
                self.out.push_str(&format!("{pad}}}\n"));
            }
            Stmt::Block { tier, body } => {
                self.out.push_str(&format!("{pad}{}{{\n", tier_prefix(*tier)));
                self.block(body, indent + 1);
                self.out.push_str(&format!("{pad}}}\n"));
            }
            Stmt::Break => self.out.push_str(&format!("{pad}break;\n")),
            Stmt::Continue => self.out.push_str(&format!("{pad}continue;\n")),
            Stmt::Print { signed, value } => {
                let (f, to) = if *signed { ("put_i", "int64") } else { ("put_u", "uint64") };
                if PRINT_THROUGH_TO.load(Ordering::Relaxed) {
                    self.simple(s, &pad, Tier::Default, &format!("{f}[to.{to}["), value, "]];");
                } else {
                    self.simple(s, &pad, Tier::Default, &format!("{f}["), value, "];");
                }
            }
            Stmt::Return(e) => self.simple(s, &pad, Tier::Default, "return ", e, ";"),
            Stmt::ListLet { name, mutable, elem, items } => {
                let m = if *mutable { "mut " } else { "" };
                self.out.push_str(&pad);
                let s0 = self.out.len();
                self.out.push_str(&format!("{m}{}: list[{}] = [", q(name), elem.name()));
                for (i, x) in items.iter().enumerate() {
                    if i > 0 {
                        self.out.push_str(", ");
                    }
                    self.e(x);
                }
                // `[7]` is the number seven; a one-item list is `[7,]`.
                self.out.push_str(if items.len() == 1 { ",];" } else { "];" });
                self.stmt.push((addr(s), s0, s0, self.out.len()));
                self.out.push('\n');
            }
            Stmt::SetAt { tier, name, pos, value } => {
                self.out.push_str(&pad);
                let s0 = self.out.len();
                self.out.push_str(tier_prefix(*tier));
                let s1 = self.out.len();
                self.out.push_str(&format!("change {}[", q(name)));
                self.e(pos);
                self.out.push_str("] = ");
                self.e(value);
                self.out.push(';');
                self.stmt.push((addr(s), s0, s1, self.out.len()));
                self.out.push('\n');
            }
            Stmt::Append { name, value } => {
                let head = format!("list.append[lend.mut {}, ", q(name));
                self.simple(s, &pad, Tier::Default, &head, value, "];");
            }
            Stmt::ForEach { pos, item, list, body } => {
                let pos = pos.as_ref().map(|p| format!("{}, ", q(p))).unwrap_or_default();
                self.out.push_str(&format!("{pad}for {pos}{} in lend {} {{\n", q(item), q(list)));
                self.block(body, indent + 1);
                self.out.push_str(&format!("{pad}}}\n"));
            }
        }
    }
}

impl Program {
    /// The same text as `render`, with every node's place in it.
    pub fn render_spans(&self) -> (String, Spans) {
        let mut w = Wr { out: String::new(), loose: self.loose, expr: vec![], stmt: vec![], _p: std::marker::PhantomData };
        w.out.push_str(&format!("# check claude generator, seed {}\n", self.seed));
        w.out.push_str("extern fn write ['fd': int32, 'buf': ptr[uint8], 'len': uint64] -> int64;\n");
        if !self.inputs.is_empty() {
            w.out.push_str("extern fn getenv ['name': ptr[uint8]] -> ptr[uint8];\n");
            w.out.push_str("extern fn strlen ['s': ptr[uint8]] -> uint64;\n");
        }
        w.out.push('\n');
        w.out.push_str(&self.printer);
        for f in &self.funcs {
            let params: Vec<String> = f.params.iter().map(|p| format!("{}: {}", q(&p.name), p.ty.text())).collect();
            w.out.push_str(&format!("\nfn {} [{}] -> {} {{\n", f.name, params.join(", "), f.ret.text()));
            w.block(&f.body, 1);
            w.out.push_str("}\n");
        }
        w.out.push_str("\nMAIN {\n");
        for i in &self.inputs {
            let key = format!("{}_key", i.name);
            w.out.push_str(&format!("    {} := \"{}\";\n", q(&key), i.var));
            w.out.push_str(&format!("    nocheck {}: uint64 = strlen[getenv[lend {}]];\n", q(&i.name), q(&key)));
        }
        w.block(&self.main, 1);
        w.out.push_str("}\n");
        // byte offsets to (line, column)
        let starts: Vec<usize> = std::iter::once(0).chain(w.out.match_indices('\n').map(|(i, _)| i + 1)).collect();
        let at = |off: usize| -> (u32, u32) {
            let line = starts.partition_point(|&s| s <= off);
            (line as u32, (off - starts[line - 1] + 1) as u32)
        };
        let mut spans = Spans::default();
        for (a, s, e) in &w.expr {
            let (l, c) = at(*s);
            let (_, ce) = at(*e);
            spans.expr.insert(*a, (l, c, ce));
        }
        for (a, s0, s1, e) in &w.stmt {
            let (l, c0) = at(*s0);
            let (_, c1) = at(*s1);
            let (_, ce) = at(*e);
            spans.stmt.insert(*a, (l, c0, c1, ce));
        }
        (w.out, spans)
    }
}

/// The exact value of an expression made of bare numbers alone, worked out
/// as design/operators.md says ("bare numbers alone are worked out exactly
/// at compile time"): no width in between, `/` floors, `mod` takes the
/// divisor's sign. None if it is not bare numbers alone, or cannot be done
/// (division by zero, a negative exponent, past i128).
pub fn fold_bare(e: &Expr) -> Option<i128> {
    match e {
        Expr::Lit(v) => Some(*v),
        Expr::Neg(a) => fold_bare(a)?.checked_neg(),
        Expr::Bin(op, a, b) => {
            let (x, y) = (fold_bare(a)?, fold_bare(b)?);
            match op {
                BinOp::Add => x.checked_add(y),
                BinOp::Sub => x.checked_sub(y),
                BinOp::Mul => x.checked_mul(y),
                BinOp::Div | BinOp::Mod => {
                    if y == 0 {
                        return None;
                    }
                    let q = x.checked_div(y)?;
                    let q = if x % y != 0 && ((x < 0) != (y < 0)) { q - 1 } else { q };
                    if *op == BinOp::Div {
                        Some(q)
                    } else {
                        x.checked_sub(q.checked_mul(y)?)
                    }
                }
                BinOp::Pow => {
                    if y < 0 || y > 200 {
                        return None;
                    }
                    x.checked_pow(y as u32)
                }
                _ => None,
            }
        }
        _ => None,
    }
}
