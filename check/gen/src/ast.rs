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
            Ty::I8 => "i8",
            Ty::I16 => "i16",
            Ty::I32 => "i32",
            Ty::I64 => "i64",
            Ty::U8 => "u8",
            Ty::U16 => "u16",
            Ty::U32 => "u32",
            Ty::U64 => "u64",
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
/// (`int32[min 0, max 100]`, `int32[min 1]`, `int32[max 100]`; owner, 2026-10-12).
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
            // bounds labelled, a side left out the type's own limit (owner, 2026-10-12)
            Some((lo, hi)) => {
                let mut parts = vec![];
                if let Some(v) = lo {
                    parts.push(format!("min: {v}"));
                }
                if let Some(v) = hi {
                    parts.push(format!("max: {v}"));
                }
                if parts.is_empty() {
                    return self.base.name().to_string();
                }
                format!("{}[{}]", self.base.name(), parts.join(", "))
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
    /// Every division says how it rounds (owner, 2026-10-02, 5a12c07): `Div`
    /// and `Mod` are `div.floor` / `mod.floor`; the rest are `.trunc`,
    /// `.ceil` and `.near` (a tie to even), and `mod.X` is what `div.X`
    /// leaves. On a deci, bin or brain16, `DivNear` is ordinary division.
    DivTrunc,
    ModTrunc,
    DivCeil,
    ModCeil,
    DivNear,
    ModNear,
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
    /// A division or remainder: whether it is the remainder.
    pub fn division(self) -> Option<bool> {
        match self {
            BinOp::Div | BinOp::DivTrunc | BinOp::DivCeil | BinOp::DivNear => Some(false),
            BinOp::Mod | BinOp::ModTrunc | BinOp::ModCeil | BinOp::ModNear => Some(true),
            _ => None,
        }
    }

    /// The integer quotient `x` over `y` rounded as this operator says
    /// (its `div` for a `mod`). None for a zero divisor or past i128.
    pub fn quotient(self, x: i128, y: i128) -> Option<i128> {
        let q = x.checked_div(y)?;
        let r = x.checked_rem(y)?;
        if r == 0 {
            return Some(q);
        }
        // q is truncated; `away` is the next whole number away from zero
        let neg = (x < 0) != (y < 0);
        let away = if neg { q - 1 } else { q + 1 };
        Some(match self {
            BinOp::DivTrunc | BinOp::ModTrunc => q,
            BinOp::Div | BinOp::Mod => if neg { away } else { q },
            BinOp::DivCeil | BinOp::ModCeil => if neg { q } else { away },
            _ => {
                let (twice, ay) = (r.checked_abs()?.checked_mul(2)?, y.checked_abs()?);
                if twice > ay || (twice == ay && q % 2 != 0) { away } else { q }
            }
        })
    }

    /// The whole answer of this division or remainder, None as `quotient`.
    pub fn divide(self, x: i128, y: i128) -> Option<i128> {
        let q = self.quotient(x, y)?;
        if self.division()? { x.checked_sub(q.checked_mul(y)?) } else { Some(q) }
    }

    /// Bounds of this remainder for a dividend in `x` and a divisor in `d`
    /// (of one sign, never 0).
    pub fn mod_bounds(self, x: (i128, i128), d: (i128, i128)) -> (i128, i128) {
        let m = d.0.saturating_abs().max(d.1.saturating_abs());
        let pos = d.0 > 0;
        match self {
            BinOp::Mod if pos && x.0 >= 0 && x.1 < d.0 => x,
            BinOp::Mod if !pos && x.1 <= 0 && x.0 > d.1 => x,
            BinOp::Mod => if pos { (0, m - 1) } else { (-(m - 1), 0) },
            BinOp::ModCeil => if pos { (-(m - 1), 0) } else { (0, m - 1) },
            BinOp::ModTrunc => (if x.0 >= 0 { 0 } else { x.0.max(-(m - 1)) }, if x.1 <= 0 { 0 } else { x.1.min(m - 1) }),
            _ => (-(m / 2), m / 2),
        }
    }

    /// Its symbol, for the operators that have one (owner, 2026-10-16).
    pub fn symbol(self) -> Option<&'static str> {
        match self {
            BinOp::Add => Some("+"),
            BinOp::Sub => Some("-"),
            BinOp::Mul => Some("x"),
            BinOp::Pow => Some("xx"),
            _ => None,
        }
    }

    pub fn text(self) -> &'static str {
        match self {
            BinOp::Add => "plus",
            BinOp::Sub => "minus",
            BinOp::Mul => "times",
            BinOp::Div => "div.floor",
            BinOp::Mod => "mod.floor",
            BinOp::DivTrunc => "div.trunc",
            BinOp::ModTrunc => "mod.trunc",
            BinOp::DivCeil => "div.ceil",
            BinOp::ModCeil => "mod.ceil",
            BinOp::DivNear => "div.near",
            BinOp::ModNear => "mod.near",
            BinOp::Pow => "pow",
            BinOp::AndBit => "and.bit",
            BinOp::OrBit => "or.bit",
            BinOp::Xor => "xor",
            BinOp::Shl => "shl",
            BinOp::Shr => "shr",
            BinOp::WAdd => "plus.wrap",
            BinOp::WSub => "minus.wrap",
            BinOp::WMul => "times.wrap",
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
    /// `<` and `>`; the others are words only (owner, 2026-10-16).
    pub fn symbol(self) -> Option<&'static str> {
        match self {
            CmpOp::Lt => Some("<"),
            CmpOp::Gt => Some(">"),
            _ => None,
        }
    }

    pub fn text(self) -> &'static str {
        match self {
            CmpOp::Eq => "equal_to",
            CmpOp::Ne => "not_equal",
            CmpOp::Lt => "less_than",
            CmpOp::Gt => "greater_than",
            CmpOp::Le => "at_most",
            CmpOp::Ge => "at_least",
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
    /// `a and.then b`: `b` only if `a` held (owner, 2026-09-25).
    AndThen(Box<Expr>, Box<Expr>),
    /// `a or.else b`: `b` only if `a` failed.
    OrElse(Box<Expr>, Box<Expr>),
    To(Ty, Box<Expr>),
    Call(String, Vec<Expr>),
    /// `'xs'[p]`: the item at a 1-based position (a check site).
    At(String, Box<Expr>),
    /// `list.length[lend 'xs']`, a `uint64`.
    Len(String),
    /// `lend 'xs'`: an argument to a borrow function, or what one returns.
    Lend(String),
    /// `E1.c2[v]`, or `E1.c1` for a case with no data (design/enums.md).
    Case(String, String, Option<Box<Expr>>),
    /// A deci or bin literal as written (`2.50`, `-0.125`): its digits and
    /// trailing zeros are part of the value (design/types.md, deci).
    Num(String),
    /// A text literal, `"1.250"`.
    Text(String),
    /// `to.T[v]`, or `to.T[v] otherwise fb` (design/conversion.md); `T` as
    /// written (`deci64`, `useful.deci64`, `int32`, `str.utf8`, `deci64[0.0, 1.0]`).
    Conv(String, Box<Expr>, Option<Box<Expr>>),
    /// `str.bytes[lend 's']`
    StrBytes(String),
    /// Source text as written, for leaves nothing else models (a char
    /// literal, a list of bytes, `str.points[lend 't']`).
    Src(String),
    /// `list.remove[lend.mut 'xs', pos p]`, its item handed back (lists.md).
    Remove(String, Box<Expr>),
    /// `to.K[v] otherwise { out_of_range { K.c1 } }`: a number into a
    /// `layout.C enum`, the fallback where no case has it (owner, 2026-09-27).
    ToEnum(String, Box<Expr>, Box<Expr>),
    /// `simd.fill[v]`: every lane `v` (design/simd.md).
    SimdFill(Box<Expr>),
    /// `simd.max[v]`, `simd.min`, `simd.sum` (a check site), `simd.all`,
    /// `simd.any`: a vector folded to one value.
    SimdFold(&'static str, Box<Expr>),
    /// `match 'e' { c1 { v } c2 'd' { return w; } }`: every case covered;
    /// an arm gives a value, or holds statements that leave (23ab20b).
    Match(Box<Expr>, Vec<Arm>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Arm {
    pub case: String,
    /// The name for the case's data, if it has any.
    pub bind: Option<String>,
    /// The value the arm gives (a `match` that gives a value) …
    pub value: Option<Expr>,
    /// … or its statements (a `match` statement, or an arm that leaves).
    pub body: Vec<Stmt>,
}

/// `enum E1 { c1  c2: int32 }`.
#[derive(Clone, Debug, PartialEq)]
pub struct EnumDef {
    pub name: String,
    pub cases: Vec<(String, Option<Ty>)>,
    /// A `layout.C enum`: each case's number (no case has data).
    pub values: Option<Vec<i128>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tier {
    Default,
    Check,
    Nocheck,
    /// `NO_FUCKING_CHECKS`: what `nocheck` turns off, and C's answers
    /// trusted too (owner, 2026-09-27).
    NoChecks,
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
    /// `cheat`: `for !cheat` (owner, 2026-10-01; once `!c`): every round really runs; same
    /// answers as a plain `for`. 0 plain, 1 `!cheat`, 2 `!cheat seq`.
    For { perm: bool, cheat: u8, name: String, ty: Ty, start: Expr, stop: Expr, step: Option<i128>, body: Vec<Stmt> },
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
    /// `mut 'xs': list[T] = list.fill[v, n];` — n copies of v; a count below
    /// 0 is a check site; `count n` in a strict file (owner, 2026-09-30).
    ListFill { name: String, mutable: bool, elem: Ty, value: Expr, count: Expr },
    /// `change 'xs'[p] = v;`
    SetAt { tier: Tier, name: String, pos: Expr, value: Expr },
    /// `list.append[lend.mut 'xs', value v];`
    Append { name: String, value: Expr },
    /// `for 'x' in lend 'xs'`, or `for 'i', 'x' in lend 'xs'` with the
    /// position from 1 (owner, c7951d4).
    ForEach { pos: Option<String>, item: String, list: String, body: Vec<Stmt>, mutable: bool },
    /// `'r' := b1[lend 'xs', …];`: a borrow handed back by a call.
    Borrow { name: String, value: Expr },
    /// `'g' := give 'xs';`: 'xs' is dead after this.
    Give { name: String, from: String },
    /// `mut 'e': E1 = E1.c2[5];`
    EnumLet { name: String, mutable: bool, en: String, value: Expr },
    /// `match 'e' { … }` as a statement: every arm a block.
    MatchStmt { scrut: Expr, arms: Vec<Arm> },
    /// `mut 'd': deci64 = …;` (the type as written, a range type included).
    TypedLet { name: String, mutable: bool, ty: String, value: Expr },
    /// `'z' := …;`
    AutoLet { name: String, value: Expr },
    /// A statement as written (a call with `lend.mut`, for one).
    Src(String),
    /// `list.remove[lend.mut 'xs', pos p];`, the item dropped.
    Remove { name: String, pos: Expr },
    /// A declaration written with `shadow` in `let`'s place: a name already
    /// visible, declared again on purpose (owner, 2026-10-08).
    Shadow(Box<Stmt>),
    /// A declaration written with `const` in `let`'s place, inside a body
    /// (owner, 2026-10-10): known at compile time, never changed or shadowed.
    Const(Box<Stmt>),
    /// `mut 's': simd[int32, count 4] = …;` (count, owner 2026-10-19)
    SimdLet { name: String, mutable: bool, elem: Ty, lanes: usize, value: Expr },
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

/// A function that hands back a borrow of one of its lists
/// (design/ownership-syntax.md, 46faa74, 09044b6):
/// `fn b1 [lent@x 'a': list[int32], lent@y 'b': list[int32]] -> lent@x list[int32]`.
#[derive(Clone, Debug, PartialEq)]
pub struct BFunc {
    pub name: String,
    /// Each parameter's name and lifetime group (`@x`), if it has one.
    pub params: Vec<(String, Option<String>)>,
    pub elem: Ty,
    /// `-> lent@g list[T]` (`Some(group)`), or `-> list[T]` when `unsaid`:
    /// a deliberately wrong signature that hides the borrow.
    pub ret_group: Option<String>,
    pub unsaid: bool,
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
    pub bfuncs: Vec<BFunc>,
    /// Functions as written.
    pub extra_fns: Vec<String>,
    pub enums: Vec<EnumDef>,
    pub main: Vec<Stmt>,
    /// Built with a borrow mistake on purpose: both engines must refuse it.
    pub expect_refused: bool,
    /// A `#strict` file (owner, 2026-09-27): `pos` on positions, labelled
    /// ranges, `mut`/`immut` on every declaration, `temp`/`perm` on every
    /// `for`, `giveS` in a match that gives a value, and every site proven
    /// within its own function.
    pub strict: bool,
    /// Written in the main spellings of 9a9e573 (`!mut`, `!check`, `! 'x'`);
    /// otherwise in MTM's, which a lone file also takes.
    pub new_words: bool,
    /// `to.uint8[5]` written `5: uint8` every other time (owner, 2026-10-17;
    /// oracle e88f86be): the colon covers only the literal before it.
    pub typed_lits: bool,
    /// Every third whole literal of 0 or more written `binary …`, `hex …`
    /// or `octal …` in turn (owner, 2026-10-19): the same number.
    pub based_lits: bool,
    /// Omit parentheses where design/operators.md says they are not needed.
    pub loose: bool,
    /// Holds values my exact evaluator does not model (deci, bin, text):
    /// compared between the engines only.
    pub unevaluable: bool,
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
        Expr::Num(t) if t.starts_with('-') => Some(2),
        Expr::Num(_) | Expr::Text(_) | Expr::Conv(..) | Expr::StrBytes(_) | Expr::Src(_) | Expr::Remove(..) => Some(0),
        Expr::Lit(_) | Expr::Bool(_) | Expr::Var(_) | Expr::To(..) | Expr::Call(..) | Expr::At(..) | Expr::Len(_) | Expr::Lend(_) | Expr::Case(..) | Expr::Match(..) | Expr::SimdFill(_) | Expr::SimdFold(..) | Expr::ToEnum(..) => Some(0),
        Expr::Bin(BinOp::Pow, ..) => Some(1),
        Expr::Neg(_) => Some(2),
        Expr::Bin(BinOp::Mul | BinOp::Div, ..) => Some(3),
        Expr::Bin(BinOp::Add | BinOp::Sub, ..) => Some(4),
        Expr::Cmp(..) => Some(5),
        Expr::Not(_) => Some(6),
        Expr::And(..) | Expr::AndThen(..) => Some(7),
        Expr::Or(..) | Expr::OrElse(..) => Some(8),
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
        // Same precedence as `and` / `or` (operators.md), but no mixing
        // is left bare: only comparisons and `not` go without parentheses.
        Expr::AndThen(..) | Expr::OrElse(..) => matches!(child, Expr::Cmp(..) | Expr::Not(_)),
        _ => false,
    }
}

fn tier_prefix(t: Tier, _new_words: bool) -> &'static str {
    match t {
        Tier::Default => "",
        Tier::Check => "check ",
        Tier::Nocheck => "no_check ",
        Tier::NoChecks => "NO_FUCKING_CHECKS ",
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
    /// `!mut`, `!check` and `! 'x'` in place of `immut`, `nocheck` and `not`.
    new_words: bool,
    /// How many operators with a symbol spelling were written so far:
    /// every other one is written as its symbol (owner, 2026-10-16).
    syms: usize,
    typed_lits: bool,
    /// How many literal conversions could be typed literals so far.
    tlits: usize,
    /// How many plain lets of a call so far, for `no_cheat`.
    cheats: usize,
    based_lits: bool,
    /// How many whole literals of 0 or more so far, for the bases.
    bases: usize,
    /// The declarations something changes (see `changed_decls`), by address.
    changed: std::collections::HashSet<usize>,
    out: String,
    loose: bool,
    /// byte offsets while writing
    expr: Vec<(usize, usize, usize)>,
    stmt: Vec<(usize, usize, usize, usize)>,
    /// The indent of the statement being written, for a `match` inside it.
    indent: usize,
    strict: bool,
    _p: std::marker::PhantomData<&'a ()>,
}

impl<'a> Wr<'a> {
    /// Whether a declaration may say `mut`: only when something changes it
    /// (owner, 2026-10-10: a `mut` nothing changes is refused).
    fn changed(&self, s: &Stmt) -> bool {
        self.changed.contains(&addr(s))
    }

    /// The word for a name that never changes, in a `#strict` file.
    fn im(&self) -> &'static str {
        "immut "
    }

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
    /// The word, or every other time its symbol where it has one: `+ - x xx
    /// < >` read correctly and are good practice, the words name the
    /// variations (owner, 2026-10-16: "Symbols only where they read correctly").
    fn spell(&mut self, word: &'static str, symbol: Option<&'static str>) -> &'static str {
        let Some(sym) = symbol else { return word };
        self.syms += 1;
        if self.syms % 2 == 0 { sym } else { word }
    }
    fn inner(&mut self, e: &Expr) {
        match e {
            Expr::Lit(v) if self.based_lits && *v >= 0 && { self.bases += 1; self.bases % 3 == 0 } => {
                let (word, digits) = match (self.bases / 3) % 3 {
                    0 => ("bin", format!("{v:b}")),
                    1 => ("hex", if (self.bases / 9) % 2 == 0 { format!("{v:X}") } else { format!("{v:x}") }),
                    _ => ("oct", format!("{v:o}")),
                };
                // `_` grouping, as in decimals, in fours from the right when long
                let grouped = if digits.len() > 4 && (self.bases / 3) % 2 == 1 {
                    let d: Vec<char> = digits.chars().collect();
                    let mut out = String::new();
                    for (i, c) in d.iter().enumerate() {
                        if i > 0 && (d.len() - i) % 4 == 0 {
                            out.push('_');
                        }
                        out.push(*c);
                    }
                    out
                } else {
                    digits
                };
                self.out.push_str(&format!("{word} {grouped}"));
            }
            Expr::Lit(v) => self.out.push_str(&v.to_string()),
            Expr::Bool(b) => self.out.push_str(&b.to_string()),
            Expr::Var(n) => self.out.push_str(&q(n)),
            Expr::Bin(op, a, b) => {
                self.operand(e, a, true);
                let t = self.spell(op.text(), op.symbol());
                self.out.push_str(&format!(" {t} "));
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
                // `! 'x'`, spaced, is the main spelling (owner, 9a9e573); `not` is MTM's
                self.out.push_str("not ");
                self.operand(e, a, false);
            }
            Expr::Cmp(op, a, b) => {
                self.operand(e, a, true);
                let t = self.spell(op.text(), op.symbol());
                self.out.push_str(&format!(" {t} "));
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
            Expr::AndThen(a, b) => {
                self.operand(e, a, true);
                self.out.push_str(" and.then ");
                self.operand(e, b, false);
            }
            Expr::OrElse(a, b) => {
                self.operand(e, a, true);
                self.out.push_str(" or.else ");
                self.operand(e, b, false);
            }
            Expr::To(t, a) if self.typed_lits && matches!(&**a, Expr::Lit(v) if t.range().has(*v)) && { self.tlits += 1; self.tlits % 2 == 0 } => {
                let Expr::Lit(v) = &**a else { unreachable!() };
                // the sign is the literal's; a negative one is bracketed whole
                if *v < 0 {
                    self.out.push_str(&format!("({v}: {})", t.name()));
                } else {
                    self.out.push_str(&format!("{v}: {}", t.name()));
                }
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
                self.out.push_str("[pos: ");
                self.e(p);
                self.out.push(']');
            }
            Expr::Len(n) => self.out.push_str(&format!("list.length[lend {}]", q(n))),
            Expr::Lend(n) => self.out.push_str(&format!("lend {}", q(n))),
            Expr::Num(t) | Expr::Src(t) => self.out.push_str(t),
            Expr::Remove(n, p) => {
                self.out.push_str(&format!("list.remove[lend.mut {}, pos: ", q(n)));
                self.e(p);
                self.out.push(']');
            }
            // An empty text is `str.empty`; a written `""` is refused (owner, 2026-10-14).
            Expr::Text(t) if t.is_empty() => self.out.push_str("str.empty"),
            Expr::Text(t) => self.out.push_str(&format!("{t:?}")),
            Expr::StrBytes(n) => self.out.push_str(&format!("str.bytes[lend {}]", q(n))),
            Expr::Conv(t, v, fb) => {
                self.out.push_str(&format!("to.{t}["));
                self.e(v);
                self.out.push(']');
                if let Some(fb) = fb {
                    self.out.push_str(" otherwise ");
                    self.e(fb);
                }
            }
            Expr::ToEnum(en, v, fallback) => {
                self.out.push_str(&format!("to.{en}["));
                self.e(v);
                self.out.push_str("] otherwise { out_of_range { is ");
                self.e(fallback);
                self.out.push_str("; } }");
            }
            Expr::SimdFill(v) => {
                self.out.push_str("simd.fill[value: ");
                self.e(v);
                self.out.push(']');
            }
            Expr::SimdFold(f, v) => {
                self.out.push_str(&format!("simd.{f}["));
                self.e(v);
                self.out.push(']');
            }
            Expr::Case(en, c, data) => {
                self.out.push_str(&format!("{en}.{c}"));
                if let Some(d) = data {
                    self.out.push('[');
                    self.e(d);
                    self.out.push(']');
                }
            }
            Expr::Match(scrut, arms) => {
                self.out.push_str("match ");
                self.e(scrut);
                self.out.push_str(" {\n");
                let indent = self.indent;
                self.arms(arms, indent + 1);
                self.out.push_str(&"    ".repeat(indent));
                self.out.push('}');
            }
        }
    }
    fn arms(&mut self, arms: &[Arm], indent: usize) {
        let pad = "    ".repeat(indent);
        for a in arms {
            self.out.push_str(&pad);
            self.out.push_str(&a.case);
            if let Some(b) = &a.bind {
                self.out.push_str(&format!(" {}", q(b)));
            }
            if let Some(v) = &a.value {
                // a case says its value with `is` (owner, 2026-10-07)
                self.out.push_str(" { is ");
                let saved = std::mem::replace(&mut self.indent, indent);
                self.e(v);
                self.indent = saved;
                self.out.push_str("; }\n");
            } else if a.body.is_empty() {
                self.out.push_str(" { }\n");
            } else {
                self.out.push_str(" {\n");
                self.block(&a.body, indent + 1);
                self.out.push_str(&format!("{pad}}}\n"));
            }
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
        self.out.push_str(tier_prefix(tier, self.new_words));
        let s1 = self.out.len();
        self.out.push_str(head);
        self.e(e);
        self.out.push_str(tail);
        self.stmt.push((addr(s), s0, s1, self.out.len()));
        self.out.push('\n');
    }
    fn s(&mut self, s: &Stmt, indent: usize) {
        let pad = "    ".repeat(indent);
        self.indent = indent;
        match s {
            Stmt::EnumLet { name, mutable, en, value } => {
                let head = format!("let {}{}: {en} = ", if *mutable && self.changed(s) { "mut " } else if self.strict { self.im() } else { "" }, q(name));
                self.simple(s, &pad, Tier::Default, &head, value, ";");
            }
            Stmt::ListFill { name, mutable, elem, value, count } => {
                let m = if *mutable && self.changed(s) { "mut " } else if self.strict { self.im() } else { "" };
                self.out.push_str(&pad);
                let s0 = self.out.len();
                self.out.push_str(&format!("let {m}{}: list[{}] = list.fill[value: ", q(name), elem.name()));
                self.e(value);
                self.out.push_str(", count: ");
                self.e(count);
                self.out.push_str("];");
                self.stmt.push((addr(s), s0, s0, self.out.len()));
                self.out.push('\n');
            }
            Stmt::Src(t) => self.out.push_str(&format!("{pad}{t}\n")),
            Stmt::Remove { name, pos } => {
                self.out.push_str(&format!("{pad}list.remove[lend.mut {}, pos: ", q(name)));
                self.e(pos);
                self.out.push_str("];\n");
            }
            Stmt::Shadow(inner) | Stmt::Const(inner) => {
                // the declaration as it is, `shadow` or `const` in `let`'s place
                let word = if matches!(s, Stmt::Shadow(_)) { "shadow " } else { "const " };
                let start = self.out.len();
                self.s(inner, indent);
                let line = self.out[start..].to_string();
                let fixed = line.replacen("let ", word, 1);
                self.out.truncate(start);
                self.out.push_str(&fixed);
            }
            Stmt::TypedLet { name, mutable, ty, value } => {
                let m = if *mutable && self.changed(s) { "mut " } else if self.strict { self.im() } else { "" };
                let head = format!("let {m}{}: {ty} = ", q(name));
                self.simple(s, &pad, Tier::Default, &head, value, ";");
            }
            Stmt::AutoLet { name, value } => {
                let head = format!("let {}{} := ", if self.strict { self.im() } else { "" }, q(name));
                self.simple(s, &pad, Tier::Default, &head, value, ";");
            }
            Stmt::SimdLet { name, mutable, elem, lanes, value } => {
                let m = if *mutable && self.changed(s) { "mut " } else if self.strict { self.im() } else { "" };
                let head = format!("let {m}{}: simd[{}, count: {lanes}] = ", q(name), elem.name());
                self.simple(s, &pad, Tier::Default, &head, value, ";");
            }
            Stmt::MatchStmt { scrut, arms } => {
                self.out.push_str(&format!("{pad}match "));
                self.e(scrut);
                self.out.push_str(" {\n");
                self.arms(arms, indent + 1);
                self.out.push_str(&format!("{pad}}}\n"));
            }
            Stmt::Let { tier, name, mutability, ty, value } => {
                let m = match mutability {
                    Mutability::Plain if self.strict => self.im(),
                    Mutability::Plain => "",
                    Mutability::Mut if self.changed(s) => "mut ",
                    // `mut` is a promise that something changes it (owner, 2026-10-10)
                    Mutability::Mut => "",
                    Mutability::Immut => self.im(),
                };
                // `no_cheat` in front of a statement, calls in it included (owner,
                // 2026-10-18: "(b)"): every third let of a call, after the check
                // word if there is one (owner, 2026-10-19; oracle 89d1eaf2)
                let cheat = if matches!(value, Expr::Call(..)) && { self.cheats += 1; self.cheats % 3 == 0 } { "no_cheat " } else { "" };
                let head = format!("{cheat}let {m}{}: {} = ", q(name), ty.text());
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
            Stmt::For { perm, cheat, name, ty, start, stop, step, body } => {
                let perm = if *perm { "perm " } else if self.strict { "temp " } else { "" };
                // `no_cheat` only in front of the `for`, `seq` after it (owner,
                // 2026-10-18: "(a)"; `for no_cheat seq perm` before)
                let (cheat, seq) = [("", ""), ("no_cheat ", ""), ("no_cheat ", "seq ")][*cheat as usize];
                // `for 'i' from A to B by C`, `by` always written (owner, 2026-10-12)
                self.out.push_str(&format!("{pad}{cheat}for {seq}{perm}{}: {} [from: ", q(name), ty.name()));
                self.e(start);
                self.out.push_str(", to: ");
                self.e(stop);
                self.out.push_str(&format!(", by: {}]", step.unwrap_or(1)));
                self.out.push_str(" {\n");
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
                self.out.push_str(&format!("{pad}{}{{\n", tier_prefix(*tier, self.new_words)));
                self.block(body, indent + 1);
                self.out.push_str(&format!("{pad}}}\n"));
            }
            Stmt::Break => self.out.push_str(&format!("{pad}break;\n")),
            Stmt::Continue => self.out.push_str(&format!("{pad}continue;\n")),
            Stmt::Print { signed, value } => {
                let (f, to) = if *signed { ("put_i", "i64") } else { ("put_u", "u64") };
                if PRINT_THROUGH_TO.load(Ordering::Relaxed) {
                    self.simple(s, &pad, Tier::Default, &format!("{f}[to.{to}["), value, "]];");
                } else {
                    self.simple(s, &pad, Tier::Default, &format!("{f}["), value, "];");
                }
            }
            Stmt::Return(e) => self.simple(s, &pad, Tier::Default, "return ", e, ";"),
            Stmt::ListLet { name, mutable, elem, items } => {
                let m = if *mutable && self.changed(s) { "mut " } else if self.strict { self.im() } else { "" };
                self.out.push_str(&pad);
                let s0 = self.out.len();
                self.out.push_str(&format!("let {m}{}: list[{}] = [", q(name), elem.name()));
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
                self.out.push_str(tier_prefix(*tier, self.new_words));
                let s1 = self.out.len();
                self.out.push_str(&format!("change {}[pos: ", q(name)));
                self.e(pos);
                self.out.push_str("] = ");
                self.e(value);
                self.out.push(';');
                self.stmt.push((addr(s), s0, s1, self.out.len()));
                self.out.push('\n');
            }
            Stmt::Append { name, value } => {
                let head = format!("list.append[lend.mut {}, value: ", q(name));
                self.simple(s, &pad, Tier::Default, &head, value, "];");
            }
            Stmt::Borrow { name, value } => {
                let head = format!("let {}{} := ", if self.strict { self.im() } else { "" }, q(name));
                self.simple(s, &pad, Tier::Default, &head, value, ";");
            }
            Stmt::Give { name, from } => {
                self.out.push_str(&pad);
                let s0 = self.out.len();
                self.out.push_str(&format!("let {}{} := give {};", if self.strict { self.im() } else { "" }, q(name), q(from)));
                self.stmt.push((addr(s), s0, s0, self.out.len()));
                self.out.push('\n');
            }
            Stmt::ForEach { pos, item, list, body, mutable } => {
                let pos = pos.as_ref().map(|p| format!("{}, ", q(p))).unwrap_or_default();
                let temp = if self.strict { "temp " } else { "" };
                let lend = if *mutable { "lend.mut" } else { "lend" };
                self.out.push_str(&format!("{pad}for {temp}{pos}{} in {lend} {} {{\n", q(item), q(list)));
                self.block(body, indent + 1);
                self.out.push_str(&format!("{pad}}}\n"));
            }
        }
    }
}

impl Program {
    /// The same text as `render`, with every node's place in it.
    pub fn render_spans(&self) -> (String, Spans) {
        let changed = changed_decls(self);
        let mut w = Wr { new_words: self.new_words, syms: 0, typed_lits: self.typed_lits, tlits: 0, cheats: 0, based_lits: self.based_lits, bases: 0, changed, out: String::new(), loose: self.loose, expr: vec![], stmt: vec![], indent: 0, strict: self.strict, _p: std::marker::PhantomData };
        w.out.push_str(&format!("# check claude generator, seed {}\n", self.seed));
        // `extern.C`, one line each, or as a block (owner, 2026-10-02: "both");
        // the block where there are three, so both forms are written
        let write = "fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;\n";
        if self.inputs.is_empty() {
            w.out.push_str(&format!("extern.C {write}"));
        } else {
            w.out.push_str(&format!("extern.C {{\n    {write}    fn getenv ['name': ptr[u8]] -> ptr[u8];\n    fn strlen ['s': ptr[u8]] -> u64;\n}}\n"));
        }
        w.out.push('\n');
        w.out.push_str(&self.printer);
        for e in &self.enums {
            let layout = if e.values.is_some() { "layout.C " } else { "" };
            w.out.push_str(&format!("\n{layout}enum {} {{\n", e.name));
            // `,` between cases, and after the last when there is an even
            // number of them, so both spellings are run (owner, 2026-10-14)
            let n = e.cases.len();
            for (i, (c, t)) in e.cases.iter().enumerate() {
                let comma = if i + 1 < n || n % 2 == 0 { "," } else { "" };
                match (t, &e.values) {
                    (_, Some(vs)) => w.out.push_str(&format!("    {c} = {}{comma}\n", vs[i])),
                    (Some(t), None) => w.out.push_str(&format!("    {c}: {}{comma}\n", t.name())),
                    (None, None) => w.out.push_str(&format!("    {c}{comma}\n")),
                }
            }
            w.out.push_str("}\n");
        }
        for f in &self.bfuncs {
            let t = f.elem.name();
            let group = |g: &Option<String>| g.as_ref().map(|g| format!(" @{g}")).unwrap_or_default();
            let params: Vec<String> = f.params.iter().map(|(n, g)| format!("lent{} {}: list[{t}]", group(g), q(n))).collect();
            let ret = if f.unsaid { format!("list[{t}]") } else { format!("lent{} list[{t}]", group(&f.ret_group)) };
            // every lifetime group is announced after the name (owner, 2026-10-02)
            let mut groups: Vec<String> = vec![];
            for g in f.params.iter().map(|p| &p.1).chain(std::iter::once(&f.ret_group)).flatten() {
                if !groups.contains(g) {
                    groups.push(g.clone());
                }
            }
            let announce = if groups.is_empty() { String::new() } else { format!(" [{}]", groups.iter().map(|g| format!("@{g}")).collect::<Vec<_>>().join(", ")) };
            w.out.push_str(&format!("\nfn {}{announce} [{}] -> {ret} {{\n", f.name, params.join(", ")));
            w.block(&f.body, 1);
            w.out.push_str("}\n");
        }
        for f in &self.extra_fns {
            w.out.push('\n');
            w.out.push_str(f);
        }
        for f in &self.funcs {
            let params: Vec<String> = f.params.iter().map(|p| format!("{}: {}", q(&p.name), p.ty.text())).collect();
            w.out.push_str(&format!("\nfn {} [{}] -> {} {{\n", f.name, params.join(", "), f.ret.text()));
            w.block(&f.body, 1);
            w.out.push_str("}\n");
        }
        w.out.push_str("\nMAIN {\n");
        for i in &self.inputs {
            let key = format!("{}_key", i.name);
            let im = if self.strict { w.im() } else { "" };
            w.out.push_str(&format!("    let {im}{} := \"{}\";\n", q(&key), i.var));
            let vouch = "no_check";
            w.out.push_str(&format!("    {vouch} let {im}{}: u64 = strlen[getenv[lend {}]];\n", q(&i.name), q(&key)));
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
    // Exact however wide it gets in between (crate::big); an answer past
    // i128 keeps its sign at the edge, where any landing refuses it.
    let v = fold_big(e)?;
    Some(v.to_i128().unwrap_or(if v.is_neg() { i128::MIN } else { i128::MAX }))
}

fn fold_big(e: &Expr) -> Option<crate::big::Big> {
    use crate::big::Big;
    match e {
        Expr::Lit(v) => Some(Big::from_i128(*v)),
        Expr::Neg(a) => Some(fold_big(a)?.neg()),
        Expr::Bin(op, a, b) => {
            let (x, y) = (fold_big(a)?, fold_big(b)?);
            match op {
                BinOp::Add => Some(x.add(&y)),
                BinOp::Sub => Some(x.sub(&y)),
                BinOp::Mul => Some(x.mul(&y)),
                _ if op.division().is_some() => x.divide(&y, *op),
                BinOp::Pow => {
                    let n = y.to_i128()?;
                    if n < 0 {
                        return None;
                    }
                    // 0, 1 and -1 to any power; anything else up to 200
                    let small = x.to_i128().filter(|v| (-1..=1).contains(v));
                    if n > 200 {
                        let v = small?;
                        return Some(Big::from_i128(if v == -1 && n % 2 == 1 { -1 } else if v == 0 { 0 } else { 1 }));
                    }
                    Some(x.pow(n as u32))
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Whether `e` is an exact answer (plain arithmetic or bare numbers), not a
/// value that already holds a type.
pub fn is_exact(e: &Expr) -> bool {
    match e {
        Expr::Lit(_) => true,
        Expr::Neg(a) => is_exact(a),
        Expr::Bin(op, ..) => !matches!(op, BinOp::AndBit | BinOp::OrBit | BinOp::Xor | BinOp::Shl | BinOp::Shr | BinOp::WAdd | BinOp::WSub | BinOp::WMul),
        _ => false,
    }
}

/// The declarations something changes, by the address of their statement:
/// each `change`, element write, append, remove or `lend.mut` marks the
/// declaration of that name in scope where it is written, so a change of a
/// shadow is not the shadowed one's (owner, 2026-10-10: a `mut` nothing
/// changes is refused; Synapse judges by declaration, seed 48033).
pub fn changed_decls(p: &Program) -> std::collections::HashSet<usize> {
    struct W {
        scopes: Vec<Vec<(String, usize)>>,
        out: std::collections::HashSet<usize>,
    }
    impl W {
        fn decl(&mut self, n: &str, at: usize) {
            self.scopes.last_mut().unwrap().push((n.to_string(), at));
        }
        fn mark(&mut self, n: &str) {
            if let Some(&(_, at)) = self.scopes.iter().flatten().rev().find(|(m, _)| m == n) {
                self.out.insert(at);
            }
        }
        fn text(&mut self, t: &str) {
            for key in ["change '", "lend.mut '"] {
                for (i, _) in t.match_indices(key) {
                    let rest = &t[i + key.len()..];
                    if let Some(end) = rest.find('\'') {
                        let n = rest[..end].to_string();
                        self.mark(&n);
                    }
                }
            }
        }
        fn expr(&mut self, e: &Expr) {
            match e {
                Expr::Remove(n, p) => {
                    self.mark(n);
                    self.expr(p);
                }
                Expr::Bin(_, a, b) | Expr::Cmp(_, a, b) | Expr::And(a, b) | Expr::Or(a, b) | Expr::AndThen(a, b) | Expr::OrElse(a, b) => {
                    self.expr(a);
                    self.expr(b);
                }
                Expr::Neg(a) | Expr::NotBit(a) | Expr::Not(a) | Expr::To(_, a) | Expr::At(_, a) | Expr::SimdFill(a) | Expr::SimdFold(_, a) => self.expr(a),
                Expr::Call(_, args) => args.iter().for_each(|a| self.expr(a)),
                Expr::Match(s, arms) => {
                    self.expr(s);
                    self.arms(arms);
                }
                _ => {}
            }
        }
        fn arms(&mut self, arms: &[Arm]) {
            for a in arms {
                self.scopes.push(vec![]);
                if let Some(v) = &a.value {
                    self.expr(v);
                }
                self.block(&a.body);
                self.scopes.pop();
            }
        }
        fn block(&mut self, b: &[Stmt]) {
            self.scopes.push(vec![]);
            for s in b {
                self.stmt(s);
            }
            self.scopes.pop();
        }
        fn stmt(&mut self, s: &Stmt) {
            let at = addr(s);
            match s {
                Stmt::Let { name, value, .. } | Stmt::EnumLet { name, value, .. } | Stmt::SimdLet { name, value, .. } | Stmt::TypedLet { name, value, .. } | Stmt::AutoLet { name, value } | Stmt::Borrow { name, value } => {
                    self.expr(value);
                    self.decl(name, at);
                }
                Stmt::ListLet { name, items, .. } => {
                    items.iter().for_each(|e| self.expr(e));
                    self.decl(name, at);
                }
                Stmt::ListFill { name, value, count, .. } => {
                    self.expr(value);
                    self.expr(count);
                    self.decl(name, at);
                }
                Stmt::Give { name, .. } => self.decl(name, at),
                Stmt::Shadow(inner) | Stmt::Const(inner) => self.stmt(inner),
                Stmt::Assign { name, value, .. } => {
                    self.expr(value);
                    self.mark(name);
                }
                Stmt::SetAt { name, pos, value, .. } => {
                    self.expr(pos);
                    self.expr(value);
                    self.mark(name);
                }
                Stmt::Append { name, value } => {
                    self.expr(value);
                    self.mark(name);
                }
                Stmt::Remove { name, pos } => {
                    self.expr(pos);
                    self.mark(name);
                }
                Stmt::Print { value, .. } | Stmt::Return(value) => self.expr(value),
                Stmt::Src(t) => self.text(t),
                Stmt::If { arms, otherwise } => {
                    for (c, b) in arms {
                        self.expr(c);
                        self.block(b);
                    }
                    if let Some(b) = otherwise {
                        self.block(b);
                    }
                }
                Stmt::For { start, stop, name, body, .. } => {
                    self.expr(start);
                    self.expr(stop);
                    self.scopes.push(vec![(name.clone(), 0)]);
                    self.block(body);
                    self.scopes.pop();
                }
                Stmt::ForEach { list, body, mutable, .. } => {
                    if *mutable {
                        self.mark(list);
                    }
                    self.block(body);
                }
                Stmt::While { cond, body } => {
                    self.expr(cond);
                    self.block(body);
                }
                Stmt::Loop { body } | Stmt::Block { body, .. } => self.block(body),
                Stmt::MatchStmt { scrut, arms } => {
                    self.expr(scrut);
                    self.arms(arms);
                }
                _ => {}
            }
        }
    }
    let mut w = W { scopes: vec![vec![]], out: Default::default() };
    for f in &p.funcs {
        w.scopes.push(f.params.iter().map(|x| (x.name.clone(), 0)).collect());
        w.block(&f.body);
        w.scopes.pop();
    }
    for f in &p.bfuncs {
        w.block(&f.body);
    }
    w.block(&p.main);
    w.out
}
