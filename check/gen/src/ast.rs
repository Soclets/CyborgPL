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
            BinOp::Div => "/",
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
            CmpOp::Le => "<==",
            CmpOp::Ge => ">==",
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
    Let { tier: Tier, name: String, mutability: Mutability, ty: Ty, value: Expr },
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Ty,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Func {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Ty,
    pub body: Vec<Stmt>,
}

/// One outside value: `'name'` is the length of environment variable `var`.
#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    pub name: String,
    pub var: String,
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
        Expr::Lit(_) | Expr::Bool(_) | Expr::Var(_) | Expr::To(..) | Expr::Call(..) => Some(0),
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

pub fn render_expr(e: &Expr, loose: bool) -> String {
    let operand = |child: &Expr, left: bool| -> String {
        let s = render_expr(child, loose);
        let bare = level(child) == Some(0) || (loose && bare_ok(e, child, left));
        if bare {
            s
        } else {
            format!("({s})")
        }
    };
    match e {
        Expr::Lit(v) => v.to_string(),
        Expr::Bool(b) => b.to_string(),
        Expr::Var(n) => q(n),
        Expr::Bin(op, a, b) => format!("{} {} {}", operand(a, true), op.text(), operand(b, false)),
        Expr::Neg(a) => format!("-{}", operand(a, false)),
        Expr::NotBit(a) => format!("not.bit {}", operand(a, false)),
        Expr::Not(a) => format!("not {}", operand(a, false)),
        Expr::Cmp(op, a, b) => format!("{} {} {}", operand(a, true), op.text(), operand(b, false)),
        Expr::And(a, b) => format!("{} and {}", operand(a, true), operand(b, false)),
        Expr::Or(a, b) => format!("{} or {}", operand(a, true), operand(b, false)),
        Expr::To(t, a) => format!("to.{}[{}]", t.name(), render_expr(a, loose)),
        Expr::Call(f, args) => {
            let args: Vec<String> = args.iter().map(|a| render_expr(a, loose)).collect();
            format!("{f}[{}]", args.join(", "))
        }
    }
}

fn tier_prefix(t: Tier) -> &'static str {
    match t {
        Tier::Default => "",
        Tier::Check => "check ",
        Tier::Nocheck => "nocheck ",
    }
}

fn render_block(out: &mut String, body: &[Stmt], indent: usize, loose: bool) {
    for s in body {
        render_stmt(out, s, indent, loose);
    }
}

fn render_stmt(out: &mut String, s: &Stmt, indent: usize, loose: bool) {
    let pad = "    ".repeat(indent);
    let ex = |e: &Expr| render_expr(e, loose);
    match s {
        Stmt::Let { tier, name, mutability, ty, value } => {
            let m = match mutability {
                Mutability::Plain => "",
                Mutability::Mut => "mut ",
                Mutability::Immut => "immut ",
            };
            out.push_str(&format!("{pad}{}let {m}{}: {} = {};\n", tier_prefix(*tier), q(name), ty.name(), ex(value)));
        }
        Stmt::Assign { tier, name, value } => {
            out.push_str(&format!("{pad}{}{} = {};\n", tier_prefix(*tier), q(name), ex(value)));
        }
        Stmt::If { arms, otherwise } => {
            for (i, (c, body)) in arms.iter().enumerate() {
                if i == 0 {
                    out.push_str(&format!("{pad}if {} {{\n", ex(c)));
                } else {
                    out.push_str(&format!("{pad}}} else if {} {{\n", ex(c)));
                }
                render_block(out, body, indent + 1, loose);
            }
            if let Some(body) = otherwise {
                out.push_str(&format!("{pad}}} else {{\n"));
                render_block(out, body, indent + 1, loose);
            }
            out.push_str(&format!("{pad}}}\n"));
        }
        Stmt::For { perm, name, ty, start, stop, step, body } => {
            let perm = if *perm { "perm " } else { "" };
            let step = step.map(|s| format!(", {s}")).unwrap_or_default();
            out.push_str(&format!(
                "{pad}for {perm}{}: {} in range[{}, {}{step}] {{\n",
                q(name),
                ty.name(),
                ex(start),
                ex(stop)
            ));
            render_block(out, body, indent + 1, loose);
            out.push_str(&format!("{pad}}}\n"));
        }
        Stmt::While { cond, body } => {
            out.push_str(&format!("{pad}while {} {{\n", ex(cond)));
            render_block(out, body, indent + 1, loose);
            out.push_str(&format!("{pad}}}\n"));
        }
        Stmt::Loop { body } => {
            out.push_str(&format!("{pad}loop {{\n"));
            render_block(out, body, indent + 1, loose);
            out.push_str(&format!("{pad}}}\n"));
        }
        Stmt::Block { tier, body } => {
            out.push_str(&format!("{pad}{}{{\n", tier_prefix(*tier)));
            render_block(out, body, indent + 1, loose);
            out.push_str(&format!("{pad}}}\n"));
        }
        Stmt::Break => out.push_str(&format!("{pad}break;\n")),
        Stmt::Continue => out.push_str(&format!("{pad}continue;\n")),
        Stmt::Print { signed, value } => {
            let f = if *signed { "put_i" } else { "put_u" };
            out.push_str(&format!("{pad}{f}[{}];\n", ex(value)));
        }
        Stmt::Return(e) => out.push_str(&format!("{pad}return {};\n", ex(e))),
    }
}

impl Program {
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("# check claude generator, seed {}\n", self.seed));
        out.push_str("extern fn write ['fd': int32, 'buf': ptr[uint8], 'len': uint64] -> int64;\n");
        if !self.inputs.is_empty() {
            out.push_str("extern fn getenv ['name': ptr[uint8]] -> ptr[uint8];\n");
            out.push_str("extern fn strlen ['s': ptr[uint8]] -> uint64;\n");
        }
        out.push('\n');
        out.push_str(&self.printer);
        for f in &self.funcs {
            let params: Vec<String> = f.params.iter().map(|p| format!("{}: {}", q(&p.name), p.ty.name())).collect();
            out.push_str(&format!("\nfn {} [{}] -> {} {{\n", f.name, params.join(", "), f.ret.name()));
            render_block(&mut out, &f.body, 1, self.loose);
            out.push_str("}\n");
        }
        out.push_str("\nMAIN {\n");
        for i in &self.inputs {
            let key = format!("{}_key", i.name);
            out.push_str(&format!("    let {} := \"{}\";\n", q(&key), i.var));
            out.push_str(&format!("    nocheck let {}: uint64 = strlen[getenv[lend {}]];\n", q(&i.name), q(&key)));
        }
        render_block(&mut out, &self.main, 1, self.loose);
        out.push_str("}\n");
        out
    }
}
