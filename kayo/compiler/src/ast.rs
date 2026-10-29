//! The syntax tree, exactly as parsed: what messages quote. Types and
//! names are resolved later; here a path is still spelling.

use crate::source::Span;

/// A bare word: a keyword, function, type, module, field or case name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

/// A quoted name: `'x'` — a value the writer holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QName {
    pub name: String,
    pub span: Span,
}

/// `a.b.c`, `my.helper`, `std::print.stdout`, `list.length`, `to.i32`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    /// The Kit before `::`, if any.
    pub kit: Option<Ident>,
    pub segs: Vec<Ident>,
    pub span: Span,
}

impl Path {
    pub fn head(&self) -> &str {
        &self.segs[0].name
    }
    pub fn seg(&self, i: usize) -> Option<&str> {
        self.segs.get(i).map(|s| s.name.as_str())
    }
    pub fn last(&self) -> &Ident {
        self.segs.last().unwrap()
    }
    pub fn is(&self, spelled: &[&str]) -> bool {
        self.kit.is_none() && self.segs.len() == spelled.len() && self.segs.iter().zip(spelled).all(|(s, w)| s.name == *w)
    }
    pub fn starts(&self, spelled: &[&str]) -> bool {
        self.kit.is_none() && self.segs.len() >= spelled.len() && self.segs.iter().zip(spelled).all(|(s, w)| s.name == *w)
    }
    pub fn spelled(&self) -> String {
        let mut s = String::new();
        if let Some(l) = &self.kit {
            // `std` inside, `stdkit` as written (owner, 2026-10-07).
            s.push_str(if l.name == "std" { "std" } else { &l.name });
            s.push_str("::");
        }
        s.push_str(&self.segs.iter().map(|x| x.name.as_str()).collect::<Vec<_>>().join("."));
        s
    }
}

// ---------------------------------------------------------------- types

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub kind: TypeKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    /// `i32`, `str.utf8`, `list[T]`, `map.ordered[K, V]`, `Pair[i32, str.utf8]`,
    /// `shapes.circle.Circle`, `std::uenr`, `ptr.mut[T]`, `channel[T]`.
    Path { path: Path, args: Vec<Type> },
    /// `T[lo, hi]`, `str.utf8[1, 40 graphemes]`, `list[T][0, 10]`.
    /// `old`: written `[lo, hi]`, as before 2026-10-12, read so that a
    /// type taking no bound is told that first, then refused.
    /// `places`: `max_places: N` and `min_places: N` in the same bracket
    /// (owner, 2026-10-17; `places` since 2026-10-20), on a `d` or a percent.
    Bound { inner: Box<Type>, lo: Box<Expr>, hi: Box<Expr>, unit: Option<Ident>, span: Span, old: Option<Vec<Span>>, zero: Option<(bool, Span)>, places: Option<(Option<u32>, Option<u32>, Span)> },
    /// `<T>` / `<T: Ordered>`.
    Var { name: Ident, bound: Option<Path> },
    /// `fn[lent T, T] -> R or fail[…]`.
    Fn { params: Vec<FnTypeParam>, ret: Option<Box<Type>>, fails: Vec<FailSpec>, proc: bool },
    /// `count N` where a type parameter goes: the lane count of a
    /// `simd[u8, count 16]` (owner, 2026-10-19). Nothing else takes one.
    Count(u32),
    /// `u8[bits 3]`: a C bit-field's type (owner, 2026-10-19), only a
    /// `layout.C` field's.
    Bits { inner: Box<Type>, bits: u32, span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnTypeParam {
    pub kind: ParamKind,
    pub amp: bool,
    pub ty: Type,
}

/// One entry of `fail[a, b: T]`.
#[derive(Debug, Clone, PartialEq)]
pub struct FailSpec {
    pub name: Ident,
    pub payload: Option<Type>,
}

// ---------------------------------------------------------------- expressions

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    Give,
    Copy,
    Lend,
    LendMut,
}

impl Verb {
    pub fn spelled(self) -> &'static str {
        match self {
            Verb::Give => "give",
            Verb::Copy => "copy",
            Verb::Lend => "lend",
            Verb::LendMut => "lend.mut",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    /// `negate.ieee 'x'` (owner, 2026-10-07: "yea"): IEEE's negation, a `-0`
    /// of a zero, where `-'x'` gives `0`. A bin or brain16 only.
    NegIeee,
    Not,
    NotBit,
}

impl UnOp {
    /// The operator without `.ieee`: the same number but for a zero's sign.
    pub fn plain(self) -> UnOp {
        if self == UnOp::NegIeee { UnOp::Neg } else { self }
    }
}

/// How a division rounds (owner, 2026-10-02): every `div` and `mod` names
/// one. `Bare` is a `div` or `mod` that does not, kept so the checker can
/// refuse it naming the spelling it meant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Round {
    Floor,
    Ceil,
    Trunc,
    Near,
    Bare,
}

impl Round {
    pub fn word(&self) -> &'static str {
        match self {
            Round::Floor => "floor",
            Round::Ceil => "ceil",
            Round::Trunc => "trunc",
            Round::Near => "near",
            Round::Bare => "",
        }
    }
    pub fn from_word(w: &str) -> Option<Round> {
        match w {
            "floor" => Some(Round::Floor),
            "ceil" => Some(Round::Ceil),
            "trunc" => Some(Round::Trunc),
            "near" => Some(Round::Near),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Pow,
    /// `div.X` and `mod.X`: divide, rounding as named, and what that
    /// division leaves over.
    Div(Round),
    Mod(Round),
    /// `div.X.wrap` (owner, 2026-10-03): the true quotient wrapped at the
    /// type's width, as the hardware gives the lowest value by -1.
    DivWrap(Round),
    /// `div.X.ieee` (owner, 2026-10-07: "`a`"): a bin's or brain16's
    /// quotient keeping IEEE's `-0`, where the plain one gives `0`.
    DivIeee(Round),
    /// `plus.ieee`, `minus.ieee`, `times.ieee` (owner, 2026-10-07: "yea"):
    /// IEEE's own answer, keeping a `-0`; every other bin or brain16
    /// result is never one ("`a`").
    AddIeee,
    SubIeee,
    MulIeee,
    /// `mod.floor/ceil/trunc.ieee`, `pow.ieee` (owner, 2026-10-07:
    /// "yea."): every operator that can make a -0 has its `.ieee` form.
    ModIeee(Round),
    PowIeee,
    /// `and.bit.wrap`, `or.bit.wrap`, `xor.wrap` (owner, 2026-10-07:
    /// "yea"): operands taken as their bits at the operator's width, where
    /// the plain ones must fit it.
    AndBitWrap,
    OrBitWrap,
    XorWrap,
    /// `shl.masked`, `shr.masked` (owner, 2026-10-07: "both", "Whatever's
    /// accurate"): the count taken modulo the width and the bits pushed
    /// past it cut off, as the chip does; no check site.
    ShlMasked,
    ShrMasked,
    WrapAdd,
    WrapSub,
    WrapMul,
    WrapPow,
    Join,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    /// `and.then` / `or.else` (owner, 2026-09-25): the right side only when
    /// the left has not decided. `and` / `or` always work out both.
    AndThen,
    OrElse,
    AndBit,
    OrBit,
    Xor,
    Shl,
    Shr,
    /// A glyph run or a bare word the language does not own.
    Custom(String),
}

impl BinOp {
    pub fn spelled(&self) -> String {
        match self {
            BinOp::Add => "plus".into(),
            BinOp::Sub => "minus".into(),
            BinOp::Mul => "times".into(),
            BinOp::Pow => "pow".into(),
            BinOp::Div(Round::Bare) => "div".into(),
            BinOp::Mod(Round::Bare) => "mod".into(),
            BinOp::Div(r) => format!("div.{}", r.word()),
            BinOp::DivWrap(r) => format!("div.{}.wrap", r.word()),
            BinOp::DivIeee(r) => format!("div.{}.ieee", r.word()),
            BinOp::AddIeee => "plus.ieee".into(),
            BinOp::SubIeee => "minus.ieee".into(),
            BinOp::MulIeee => "times.ieee".into(),
            BinOp::ModIeee(r) => format!("mod.{}.ieee", r.word()),
            BinOp::PowIeee => "pow.ieee".into(),
            BinOp::Mod(r) => format!("mod.{}", r.word()),
            BinOp::WrapAdd => "plus.wrap".into(),
            BinOp::WrapSub => "minus.wrap".into(),
            BinOp::WrapMul => "times.wrap".into(),
            BinOp::WrapPow => "pow.wrap".into(),
            BinOp::Join => "concat".into(),
            BinOp::Eq => "equal_to".into(),
            BinOp::Ne => "not_equal".into(),
            BinOp::Lt => "<".into(),
            BinOp::Gt => ">".into(),
            BinOp::Le => "at_most".into(),
            BinOp::Ge => "at_least".into(),
            BinOp::And => "and".into(),
            BinOp::Or => "or".into(),
            BinOp::AndThen => "and.then".into(),
            BinOp::OrElse => "or.else".into(),
            BinOp::AndBit => "and.bit".into(),
            BinOp::AndBitWrap => "and.bit.wrap".into(),
            BinOp::OrBitWrap => "or.bit.wrap".into(),
            BinOp::XorWrap => "xor.wrap".into(),
            BinOp::ShlMasked => "shl.masked".into(),
            BinOp::ShrMasked => "shr.masked".into(),
            BinOp::OrBit => "or.bit".into(),
            BinOp::Xor => "xor".into(),
            BinOp::Shl => "shl".into(),
            BinOp::Shr => "shr".into(),
            BinOp::Custom(s) => s.clone(),
        }
    }
    /// The operator without `.ieee`: the same number but for a zero's
    /// sign, so everything but the code that makes the zero reads it so.
    pub fn plain(&self) -> BinOp {
        match self {
            BinOp::AddIeee => BinOp::Add,
            BinOp::SubIeee => BinOp::Sub,
            BinOp::MulIeee => BinOp::Mul,
            BinOp::DivIeee(r) => BinOp::Div(*r),
            BinOp::ModIeee(r) => BinOp::Mod(*r),
            BinOp::PowIeee => BinOp::Pow,
            other => other.clone(),
        }
    }
    /// A form that keeps IEEE's `-0`.
    pub fn is_ieee(&self) -> bool {
        matches!(self, BinOp::AddIeee | BinOp::SubIeee | BinOp::MulIeee | BinOp::DivIeee(_) | BinOp::ModIeee(_) | BinOp::PowIeee)
    }
    pub fn is_comparison(&self) -> bool {
        matches!(self, BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge)
    }
    pub fn is_shift(&self) -> bool {
        matches!(self, BinOp::Shl | BinOp::Shr | BinOp::ShlMasked | BinOp::ShrMasked)
    }
    pub fn is_bitwise(&self) -> bool {
        matches!(self, BinOp::AndBit | BinOp::OrBit | BinOp::Xor | BinOp::AndBitWrap | BinOp::OrBitWrap | BinOp::XorWrap | BinOp::Shl | BinOp::Shr | BinOp::ShlMasked | BinOp::ShrMasked)
    }
    /// A `div.X`: beside another multiplicative operator it needs
    /// parentheses.
    pub fn is_div(&self) -> bool {
        matches!(self, BinOp::Div(_) | BinOp::DivWrap(_) | BinOp::DivIeee(_))
    }
    /// A `mod.X`: beside any operator it needs parentheses.
    pub fn is_mod(&self) -> bool {
        matches!(self, BinOp::Mod(_) | BinOp::ModIeee(_))
    }
    pub fn is_multiplicative(&self) -> bool {
        matches!(self, BinOp::Mul | BinOp::MulIeee | BinOp::Div(_) | BinOp::DivWrap(_) | BinOp::DivIeee(_) | BinOp::Mod(_) | BinOp::ModIeee(_) | BinOp::WrapMul)
    }
}

/// One argument in a call, construction or map literal bracket.
#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    /// `add[10, 20]`
    Positional(Expr),
    /// `Locker[name = "A"]`
    Named(Ident, Expr),
    /// `map["ann" = 5551]`
    Pair(Expr, Expr),
}

impl Arg {
    pub fn span(&self) -> Span {
        match self {
            Arg::Positional(e) => e.span,
            Arg::Named(n, e) => n.span.to(e.span),
            Arg::Pair(k, v) => k.span.to(v.span),
        }
    }
}

/// What follows `otherwise`.
#[derive(Debug, Clone, PartialEq)]
pub enum Otherwise {
    /// A fallback value (or a leaving call such as `std::exit[1]`).
    Value(Expr),
    /// `return …`, `fail name…`, `break`, `continue`.
    Leave(Box<Stmt>),
    /// Bare `fail`: pass the failure up unchanged.
    PassUp(Span),
    /// One arm per reason.
    Arms(Vec<FailArm>, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailArm {
    pub name: Ident,
    pub bind: Option<QName>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Capture {
    pub verb: Verb,
    pub amp: bool,
    pub name: QName,
    pub span: Span,
}

/// What an `incl` makes of its file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InclForm {
    Bytes,
    Utf8,
    Ascii,
    /// UTF-16 or UTF-32 in the byte order it says (owner, 2026-10-10):
    /// `low_first` for `.low_first`.
    Utf16 { low_first: bool },
    Utf32 { low_first: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    /// `op +`: an operator as a value, where a function goes (owner,
    /// 2026-09-29); its type comes from where it goes.
    OpValue { op: BinOp, spelled: String, unary: Option<UnOp> },
    /// `match 's' { empty { "free" } … }` used as a value (owner, 2026-09-29).
    Match { value: Box<Expr>, arms: Vec<MatchArm> },
    Int(String),
    Bin(String),
    /// `*0.1`: "exactly this", a `deci` (owner, 2026-10-02).
    Exact(String),
    /// `80%`, `exactly 7%` (owner, 2026-10-17): a `Bin` or `Exact` literal,
    /// already over 100, that is a `percent` unless its place says a plain
    /// number.
    Percent(Box<Expr>),
    /// `5: u8`, `-128: i8`, `exactly 0.1: d64` (owner, 2026-10-17):
    /// a literal of the type after the `:`, which it must fit. The checker
    /// refuses anything but a literal here, naming `to.T[…]`.
    Typed { value: Box<Expr>, ty: Type },
    Str(String),
    Char(String),
    /// The bare `\n`.
    Newline,
    Bool(bool),
    /// `5 secs`
    Duration { value: String, unit: Ident },
    /// `'x'`
    Name(QName),
    /// `a.'k'`, `shapes.circle.'pi'`, `mathstuff::'pi'`, `kit.'scale'`.
    ConstPath { path: Path, name: QName },
    /// A bare path with no bracket: a function as a value, `Status.empty`,
    /// `list.empty`, `map.empty`.
    Path(Path),
    /// `add[…]`, `list.length[…]`, `Status.claimed["x"]`, `Locker[name = …]`,
    /// `map["a" = 1]`, `std::exit[1]` — the bracket is `args`; what the path
    /// names is decided by the resolver.
    Call { path: Path, args: Vec<Arg>, otherwise: Option<Box<Otherwise>> },
    /// `'f'[…]` — calling a held function.
    CallValue { callee: Box<Expr>, args: Vec<Arg>, otherwise: Option<Box<Otherwise>> },
    /// `'x'.field`
    Field { base: Box<Expr>, field: Ident },
    /// `'xs'[i]`
    Index { base: Box<Expr>, index: Box<Expr> },
    /// `give 'x'`, `lend 'xs'[1]`, `&'s'`.
    Verb { verb: Verb, amp: bool, expr: Box<Expr> },
    Unary { op: UnOp, expr: Box<Expr> },
    Binary { op: BinOp, op_span: Span, lhs: Box<Expr>, rhs: Box<Expr> },
    /// `( … )`, or `[x]` (`true`), one value bracketed: grouping, not a
    /// list of one, which is `[x,]` (2026-09-14).
    Paren(Box<Expr>, bool),
    /// `[1, 2, 3]`, `[7,]`, `[]`
    List(Vec<Expr>),
    /// `to.i32[…]`, `to.i32[0, 100][…]`, `to.str.utf8[…]`.
    Conv { target: Type, expr: Box<Expr>, otherwise: Option<Box<Otherwise>> },
    /// `channel[T]`, `channel[T][0, N]`
    Channel { elem: Type, bound: Option<Box<(Expr, Expr)>> },
    /// `fn [params] -> T with [captures] { body }`
    FnLit(Box<FnLit>),
    /// `...'xs'`, `...copy 'xs'`
    Spread { verb: Option<Verb>, expr: Box<Expr> },
    /// `pos i`: a position, where one goes (owner, 2026-10-07: on every
    /// index and position).
    Pos(Box<Expr>),
    /// `count n`: `list.fill`'s count (owner, 2026-10-07).
    Count(Box<Expr>),
    /// `value v`: what `list.fill` and `simd.fill` fill with (owner,
    /// 2026-10-07: "`b`").
    Value(Box<Expr>),
    /// `value: "hi"` in a call to a function whose collection gives its
    /// parameters labels (owner, 2026-10-21): the label, then the value.
    Label(Ident, Box<Expr>),
    /// `size.of[T]` / `align.of[T]`
    SizeOf { align: bool, ty: Type },
    /// `incl.bytes["assets/logo.png"]`, `incl.str.utf8[…]`, `incl.str.ascii[…]`
    /// (owner, 2026-10-10): a file read while building, from the Neuron's
    /// root; `path_span` is the quoted path's.
    Incl { form: InclForm, path: String, path_span: Span },
    /// `ptr.cast[T]['p']`: the same address, seen as pointing at a T
    /// (owner, 2026-09-30: a).
    PtrCast { ty: Type, value: Box<Expr> },
    /// `pos i` where an argument goes: a position, not a value
    /// (`simd.load[lend 'xs', 'i']`).
    /// `range[start, stop, step]` — only what a `for` runs over.
    Range { start: Box<Expr>, stop: Box<Expr>, step: Option<Box<Expr>> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnLit {
    pub params: Vec<Param>,
    pub ret: Option<Type>,
    pub fails: Vec<FailSpec>,
    pub captures: Vec<Capture>,
    pub body: Block,
    /// `fn [$s] = $s at_least 60` (owner, 2026-10-24: "(c)"): one value,
    /// its types from where it goes.
    pub short: bool,
    /// `proc [ … ] = …`: a lambda with effects (owner, 2026-10-27).
    pub proc: bool,
}

// ---------------------------------------------------------------- statements

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Mut,
    Immut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    Check,
    Nocheck,
    /// `NO_FUCKING_CHECKS` (owner, 2026-09-27): everything `nocheck` turns
    /// off, and what C hands back is trusted too — no `out_of_range` on a
    /// C enum or range from C, and a bad one is undefined behaviour.
    NoChecks,
}

impl Tier {
    /// The writer vouches: `nocheck` or `NO_FUCKING_CHECKS`.
    pub fn trusts(self) -> bool {
        matches!(self, Tier::Nocheck | Tier::NoChecks)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForIter {
    /// `from 1 to 10 by 1`; `exclusive` for `before`, the end left out
    /// (owner, 2026-10-12).
    Range { start: Expr, stop: Expr, step: Option<Expr>, exclusive: bool },
    /// `lend 'xs'`, `give 'xs'`, `str.each.grapheme[…]`, `map.keys[…]`.
    Over(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ElseArm {
    If(Box<Stmt>),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub case: Ident,
    pub bind: Option<QName>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhicheverArm {
    pub name: QName,
    pub channel: Expr,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AsmDir {
    In,
    Out,
    InOut,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AsmBinding {
    Reg { name: QName, dir: AsmDir, reg: Ident },
    /// `lend 'text' -> x1`: the address of the lent data goes into the
    /// register (owner, 2026-09-30: a); `lend.mut` lets the assembly write
    /// through it.
    Lend { name: QName, mutable: bool, reg: Ident },
    Clobber(Ident),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConcurBody {
    Block(Block),
    Call(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

/// One value an `inline.MSL` block binds to a Metal buffer slot.
#[derive(Debug, Clone, PartialEq)]
pub struct MslBinding {
    /// `lend`, `lend.mut` or `copy`.
    pub verb: Verb,
    pub name: QName,
    /// `buffer(N)`'s N.
    pub slot: u32,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    /// A declaration: `let 'x' := …;`, `let mut 'x': T = …;`, `let 'x' $:= …;` (owner,
    /// 2026-09-25: no `let`). `sealed` is `!$` in front: the name may not
    /// be shadowed.
    Let { mutability: Option<Mutability>, name: QName, ty: Option<Type>, shadow: bool, value: Expr, sealed: bool },
    Assign { target: Expr, value: Expr },
    /// A `const` in a body (owner, 2026-10-10: "`a`"): seen only in its
    /// block, never assigned or shadowed.
    Const(ConstDecl),
    Expr(Expr),
    If { cond: Expr, then: Block, else_: Option<ElseArm> },
    Loop(Block),
    While { cond: Expr, body: Block },
    /// `pos`: `for 'i', 'x' in lend 'xs'` — the position beside the item
    /// (owner, 2026-09-29).
    /// `para`: made parallel by its writer (owner, 2026-09-30); not yet
    /// acted on — every loop runs in order so far.
    /// `nocheat`: `for !cheat` (owner, 2026-10-01), every round really runs.
    For { nocheat: bool, seq: bool, para: bool, perm: bool, name: QName, pos: Option<QName>, ty: Option<Type>, iter: ForIter, body: Block },
    Return(Option<Expr>),
    Fail { name: Ident, payload: Option<Expr> },
    Break,
    Continue,
    Block(Block),
    Tier { tier: Tier, stmt: Box<Stmt> },
    /// `no_cheat` in front of a statement or block (owner, 2026-10-18:
    /// "(b)"): the work written really runs, calls included.
    NoCheat { stmt: Box<Stmt> },
    Concur { detached: bool, captures: Vec<Capture>, body: ConcurBody, otherwise: Option<Otherwise> },
    Whichever { arms: Vec<WhicheverArm>, otherwise: Option<Otherwise> },
    Match { value: Expr, arms: Vec<MatchArm> },
    /// A case's value (owner, 2026-09-29): `{ "free" }` (`bare`), or
    /// `{ gives "free"; }` in a `#strict` file.
    Gives { value: Expr, bare: bool },
    InlineAsm { text: String, bindings: Vec<AsmBinding> },
    /// `inline.MSL threads <count> { … } with [lend.mut $xs -> buffer(0),
    /// copy $k -> buffer(1)];` (owner, 2026-10-01, 2026-10-22): Metal's code,
    /// handed to Apple's compiler unchanged; `text_span` is where it is.
    /// `precise` and `wrap`: `inline.MSL.precise`, `.wrap`, `.precise.wrap`
    /// (owner, 2026-10-25), Apple's compiler told safe math with no
    /// contraction, and wrapping signed arithmetic.
    InlineMsl { text: String, text_span: Span, threads: Option<Expr>, bindings: Vec<MslBinding>, precise: bool, wrap: bool },
    /// `must …;` in a `test fn` (owner, 2026-09-29): what must hold, that
    /// something fails (with a reason), or that a check site fires.
    Must { kind: MustKind, expr: Expr },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MustKind {
    /// `must 'n' == 42;`
    Hold,
    /// `must.fail …;`, `must.fail.not_a_number …;`
    Fail(Option<Ident>),
    /// `must.stop …;`
    Stop,
}

// ---------------------------------------------------------------- items

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    Owned,
    Lent,
    LentMut,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub kind: ParamKind,
    /// `lent@x`: the lifetime group this borrow belongs to (owner,
    /// 2026-09-29).
    pub group: Option<Ident>,
    pub amp: bool,
    pub mutability: Option<Mutability>,
    pub variadic: bool,
    pub name: QName,
    pub ty: Option<Type>,
    pub span: Span,
}

/// A returned borrow: `-> lent T`, `-> lent.mut@x T`.
#[derive(Debug, Clone, PartialEq)]
pub struct RetLent {
    pub mutable: bool,
    pub group: Option<Ident>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub name: Ident,
    /// `fn Counter.next […]` (owner, 2026-10-09: "`b`"): the type the
    /// function belongs to, `Counter`; `name` is then `next`.
    pub owner: Option<Ident>,
    /// `<T>: Number`, `| @x`: announced after the name (owner, 2026-10-02).
    pub announce: Announce,
    pub params: Vec<Param>,
    pub ret: Option<Type>,
    /// `-> lent@x T`: the function hands back a borrow (owner, 2026-09-29).
    pub ret_lent: Option<RetLent>,
    pub fails: Vec<FailSpec>,
    /// `None` for `extern fn` and interface entries.
    pub body: Option<Block>,
    /// `test fn` (owner, 2026-09-29): run by `cca test`, never called,
    /// left out of a normal build.
    pub test: bool,
    /// `table_bytes N fn …` (owner, 2026-10-19): the largest table Precog
    /// may make of this function, over the target's; 0 is never.
    pub table_bytes: Option<u64>,
    /// `extern.C fn length = C.strlen […]` (owner, 2026-10-19): the C
    /// function this one is, under another name.
    pub c_name: Option<String>,
    /// Written inside a `coll.fn`, where several versions of one function
    /// may stand (owner, 2026-10-20).
    pub in_coll: bool,
    /// A `proc`, which may have effects; a `fn` has none and calls only
    /// `fn`s (owner, 2026-10-27).
    pub proc: bool,
    /// `labels ['text' as value];` in its `coll.fn` (owner, 2026-10-21):
    /// the own function's parameters written `word:` in a call.
    pub labels: Option<Labels>,
    /// `defaults ['times' = 1];` in its `coll.fn` (owner, 2026-10-21): what
    /// a left-out argument is.
    pub defaults: Vec<(QName, Expr)>,
}

/// Whether a function's labelled arguments keep the labels' order (owner,
/// 2026-10-21): `labels.in_order`, `labels.any_order`, or a private
/// function's plain `labels`, which keeps it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LabelOrder {
    Plain,
    InOrder,
    AnyOrder,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Labels {
    pub order: LabelOrder,
    /// Each parameter and the word it is written with: `'text' as value`.
    pub entries: Vec<(QName, Ident)>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub immut: bool,
    pub name: Ident,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructDecl {
    pub name: Ident,
    pub announce: Announce,
    pub layout_c: bool,
    pub fields: Vec<Field>,
    /// A C struct Nerve never looks inside (owner, 2026-10-10): only
    /// pointed at, never built or sized.
    pub opaque: bool,
    /// `layout.C union` (owner, 2026-10-19): one field at a time.
    pub union: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumCase {
    pub name: Ident,
    pub payload: Option<Type>,
    /// A `layout.C enum` case's number, as the C header gives it.
    pub number: Option<(i128, Span)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDecl {
    pub name: Ident,
    pub announce: Announce,
    pub cases: Vec<EnumCase>,
    /// `layout.C enum`: C's enum, a number per case (owner, 2026-09-27: a).
    pub layout_c: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IfaceEntryName {
    Fn(Ident),
    Op(Ident),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfaceEntry {
    pub name: IfaceEntryName,
    pub params: Vec<Param>,
    pub ret: Option<Type>,
    pub fails: Vec<FailSpec>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfaceDecl {
    pub name: Ident,
    /// `interface Ordered <T> { … }`: the type the operations are about,
    /// with no bound of its own.
    pub own: Option<Ident>,
    pub entries: Vec<IfaceEntry>,
    /// The types it accepts, one per line (owner, 2026-10-02): a closed
    /// list, beside or instead of operations.
    pub types: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpDecl {
    /// The operator's spelling: `<+>`, `dot`, `<`, `+`.
    pub op: Ident,
    pub announce: Announce,
    pub params: Vec<Param>,
    pub ret: Option<Type>,
    pub body: Block,
}

/// A generic, announced right after its declaration's name with what it
/// accepts (owner, 2026-10-02): `<T>: Number and Hashable`, `<A>: any`.
#[derive(Debug, Clone, PartialEq)]
pub struct Generic {
    pub name: Ident,
    /// The interfaces it must satisfy, every one; none for `any`.
    pub bounds: Vec<Path>,
}

/// What a declaration announces after its name: its generics, one bracket
/// each, then its lifetime groups (`| @x @y`, the bar only between kinds).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Announce {
    pub generics: Vec<Generic>,
    pub groups: Vec<Ident>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConstDecl {
    pub name: QName,
    pub ty: Option<Type>,
    pub value: Expr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vis {
    Private,
    Kit,
    Pub,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    /// `extern.C { … }` (owner, 2026-10-02): its lines, each an `ExternFn`
    /// or `ExternLet` with the visibility it says; flattened into the
    /// file's items as it is read.
    ExternBlock(Vec<(Vis, ItemKind)>),
    Fn(FnDecl),
    /// A collection's member written elsewhere (owner, 2026-10-19):
    /// `area.square = shapes.square.area;`, that function under the
    /// member's name. `owner` is the collection when it has its own
    /// function; with none, the member sits in the collection's module.
    FnPointer { owner: Option<Ident>, name: Ident, target: Path },
    ExternFn(FnDecl),
    ExternLet { mutable: bool, name: QName, ty: Type },
    Struct(StructDecl),
    Enum(EnumDecl),
    Interface(IfaceDecl),
    Op(OpDecl),
    Const(ConstDecl),
    Module { name: Ident, items: Vec<Item> },
    /// `Vec satisfies Ordered;`
    Satisfies { ty: Type, iface: Path },
    /// `MAIN { … }`, or `MAIN ['argc': i32, 'argv': ptr[ptr[u8]]] { … }`
    /// with C's arguments (owner, 2026-09-29).
    Main(Vec<Param>, Block),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub vis: Vis,
    pub kind: ItemKind,
    pub span: Span,
    /// `pub bare.kit_prefix.never fn …`: how other Kits write its path.
    pub bare: Option<(Bare, Span)>,
    /// `bare.enum_prefix.optional enum …` (owner, 2026-10-20): whether its
    /// values may drop its name where the type is known.
    pub enum_bare: Option<(Bare, Span)>,
    /// `pub case.any fn …` (owner, 2026-10-06): `true` for
    /// insensitive. It covers the item's own name.
    pub case: Option<(bool, Span)>,
}

/// The kinds a `case.any [fn, var];` line in `limb.cpls` covers
/// (owner, 2026-10-06, design claude's list).
pub const CASE_KINDS: [&str; 8] = ["fn", "struct", "enum", "interface", "const", "case", "field", "var"];

/// Unicode's full case folding, as the oracle approximates it: upper case,
/// then lower (`ß` → `SS` → `ss`).
pub fn fold_case(name: &str) -> String {
    name.to_uppercase().to_lowercase()
}

/// How other Kits write the path to a `pub` thing (owner, 2026-10-06):
/// `bare.kit_prefix.never` (required), `bare.kit_prefix.optional`, `bare.kit_prefix.only` (forbidden).
/// There is no default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bare {
    Never,
    Optional,
    Only,
}

impl Bare {
    /// The kit-prefix mark (owner, 2026-10-20: `bare.` names the prefix it
    /// drops).
    pub fn spelled(self) -> &'static str {
        match self {
            Bare::Never => "bare.kit_prefix.never",
            Bare::Optional => "bare.kit_prefix.optional",
            Bare::Only => "bare.kit_prefix.only",
        }
    }
    /// The enum-prefix mark: whether an enum's values may drop its name.
    pub fn enum_spelled(self) -> &'static str {
        match self {
            Bare::Never => "bare.enum_prefix.never",
            Bare::Optional => "bare.enum_prefix.optional",
            Bare::Only => "bare.enum_prefix.only",
        }
    }
    pub fn word(self) -> &'static str {
        match self {
            Bare::Never => "never",
            Bare::Optional => "optional",
            Bare::Only => "only",
        }
    }
}

/// Which prefix a `bare.` mark drops (owner, 2026-10-20): a kit's path
/// (`bare.kit_prefix.…`) or an enum's name before its values
/// (`bare.enum_prefix.…`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prefix {
    Kit,
    Enum,
}

/// The kinds a `bare.kit_prefix.never [fn, struct];` line in `limb.cpls` covers. A
/// const has none (owner, 2026-10-06: "`b`"): `'pi'` alone is a variable,
/// so another Kit's const always has its path.
pub const BARE_KINDS: [&str; 5] = ["fn", "op", "struct", "enum", "interface"];

/// Code for one target (owner, 2026-10-06), design claude's lists, not
/// ruled: the chips and OSes `on.target` knows, and what
/// `on.machine.chip` can ask the machine.
pub const ON_CHIPS: [&str; 13] = ["aarch64", "x86_64", "x86", "arm", "riscv64", "riscv32", "wasm32", "wasm64", "powerpc64", "powerpc64le", "s390x", "mips64", "loongarch64"];
pub const ON_OSES: [&str; 10] = ["macos", "ios", "linux", "android", "windows", "freebsd", "netbsd", "openbsd", "wasi", "none"];
pub const ON_FEATURES: [&str; 14] = ["aarch64", "x86_64", "neon", "sve", "sve2", "aes", "sha2", "sha3", "crc", "lse", "dotprod", "fp16", "bf16", "i8mm"];

/// The official lobes, their registry IDs and the one version of each
/// Synapse knows (`design/cores.md`; owner, 2026-10-06, regrouped by
/// expertise 2026-10-12), the ones `lobes` may list; another lobe is a
/// limb, which Synapse cannot load yet.
pub const OFFICIAL_CORES: [(&str, &str, &str); 10] = [
    ("MCPS", "7d0ccfe6e41eed3a4d8ffe00ccaada9b89a4b9a3ea553a9d869cb6902e2ca75d", "2026-10-06T18:04:29+07:00"),
    ("AutoGPU", "67b2514aab0ffa5ca9c3ae48144498880466d68aa3b8b8f8e1082e6032c24c4d", "2026-10-06T18:04:29+07:00"),
    ("Metal", "3e7b81fec072c86d58538d10978aff1d42814c9e907f1259766851795cded9b7", "2026-10-06T18:04:29+07:00"),
    ("Nvidia", "beefcce9ed7cccf0d62f5a6e004c4bd53c6ae9e3a45c48883d3b1dc4be913361", "2026-10-06T18:04:29+07:00"),
    ("Vulkan", "63ff7ed8da5e0eead0164488e89553b240501f16d4be863ba6be1e46b51ec3ec", "2026-10-06T18:04:29+07:00"),
    ("AI", "8c723ccae25a719cd66eb44e95aeeef0557acec1467624ab91092c4ada766f21", "2026-10-07T20:22:24+07:00"),
    ("WSMW", "0fb49902b57b8bc0b72caf424d80e17d035038e2cac58292d92f18b792d205a0", "2026-10-07T20:22:24+07:00"),
    ("Money", "d7b62f4448a5b23d792a25bcdba89220e02691fb5e27cf18640ee491d049df0f", "2026-10-07T20:22:24+07:00"),
    // Everything about C (owner, 2026-10-10: "CNC Not C").
    ("CNC", "04b99a7cbb141b84b9f2f38c085c124e2badc37d663f71c6deebdd6ac24e79b1", "2026-10-10T16:42:00+07:00"),
    // Text and the world's languages: WSTN, UniTect and CAPS (owner,
    // 2026-10-12).
    ("unicode", "55a2e2de2eca2e06e8a4b3aa2c5ef7245f3f55d4dfbfd872a3ddd9500f74dece", "2026-10-12T10:19:44+07:00"),
];

/// Lobes merged into Broca on 2026-10-12: their IDs name no lobe now.
pub const BROCA_CORES: [(&str, &str); 3] = [
    ("WSTN", "e86b0df56e35b02b2ee91df6d89e63011564364d404ed7f5ea25d768d6d854d4"),
    ("UniTect", "137a9a35afc69ec6ee796bc39eeefc75680db17ce8a04443126a492812a0ae96"),
    ("CAPS", "90d0813dc72a0c7d54a4ded7a2c9577e77f0036958b245ab01808f9a8568bd01"),
];

/// Lobes folded into the Brainstem, Nerve's base, and when (owner:
/// 2026-10-08, "`a`"; IEG and TAS 2026-10-12): their IDs name no lobe now.
pub const RETIRED_CORES: [(&str, &str, &str); 4] = [
    ("ASP", "4882d1f3bf7b11d59879e4921672f15e91369e25d55a10e3dcd24e9b3fbe676f", "2026-10-08"),
    ("Genny", "c3e299d99a13567b2039c13fc17d0ca562be9d6f657f8c40a4774081983b20c9", "2026-10-08"),
    ("IEG", "5d0e846b2a0f57f6fae20216e8d856d41b6257e0a6104aaf0bc55b57b44b4085", "2026-10-12"),
    ("TAS", "63baa4cf89d8b6df6adceb812df005b30ea2771fd56f2117d0246ef911de21aa", "2026-10-12"),
];

/// MTM's registry ID, retired (owner, 2026-10-07: "MTM is gone.").
pub const MTM_ID: &str = "9c6c773027ae873a736d2e9b2dc398fd4807a7df6cf43b7e233c3447af3d6a56";

/// The line that lists an official core, for a message to show.
pub fn core_line(name: &str) -> String {
    let (id, version) = OFFICIAL_CORES.iter().find(|(n, _, _)| *n == name).map(|(_, i, v)| (*i, *v)).unwrap_or(("", ""));
    format!("{} {{` / `id = {}` / `ver = {}` / `}}", name.to_lowercase(), id, version)
}

pub const CONST_TAKES_NO_MARK: &str = "a const takes no `bare` mark: `$pi` alone is a variable, so another kit's const always has its path (owner, 2026-10-06)";

/// One parsed `.cpls` file.
#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub items: Vec<Item>,
    /// The collections' templates (owner, 2026-10-13), for `synapse
    /// generate` to write and every build to compare.
    pub templates: Vec<Template>,
    /// `#strict` at its top: which file it is.
    pub strict: Option<crate::source::FileId>,
    /// `bare.kit_prefix.optional;` as its first line: covers its module and the block
    /// modules written in it.
    pub bare_file: Option<(Bare, Span)>,
    /// `bare.kit_prefix.never [fn, struct];` lines, which cover the whole Kit and
    /// belong in `limb.cpls`.
    pub bare_kinds: Vec<(Bare, Vec<String>, Span)>,
    /// `bare.enum_prefix.optional;` as a file's line, and `bare.enum_prefix.
    /// optional [enum];` in `kit.cy` (owner, 2026-10-20).
    pub enum_bare_file: Option<(Bare, Span)>,
    pub enum_bare_kinds: Vec<(Bare, Span)>,
    /// `case.any;` at its top: covers its module and the block
    /// modules written in it (owner, 2026-10-06). `true` for insensitive.
    pub case_file: Option<(bool, Span)>,
    /// `case.any [fn, var];` lines, which cover the whole Kit.
    pub case_kinds: Vec<(bool, Vec<String>, Span)>,
}

/// A template in a collection (owner, 2026-10-13): `for 'n' in [2, 4]` (or
/// `from … to … by …`, or a list of types), `named "times_" ++ …;`, then a
/// member with no name, kept as its text, since `'T'` may stand where a
/// type goes until it is filled in.
#[derive(Debug, Clone, PartialEq)]
pub struct Template {
    /// The `for` keyword: where the template starts, and its indent.
    pub at: Span,
    pub var: QName,
    pub values: TemplateValues,
    pub named: Expr,
    /// From the member's `[` to its body's closing `}`.
    pub member: Span,
    /// The template's own closing `}`.
    pub close: Span,
    /// `#generated_N_lines_below` under it, if there is one.
    pub marker: Option<(u32, Span)>,
    /// The collection it is in: each member it makes is written
    /// `coll.member` (owner, 2026-10-19).
    pub coll: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplateValues {
    /// `in [2, 10, 'big']` or `in [i16, i64]`, each as written.
    List(Vec<Expr>),
    /// `from 1 to 3 by 1`, `before` leaving the end out.
    Count { start: Expr, stop: Expr, step: Expr, exclusive: bool },
}
