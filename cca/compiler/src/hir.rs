//! The checked tree: names resolved, every expression typed, every call
//! bound to what it calls. What rustc calls HIR; MIR is built from it.

use crate::ast::{BinOp, Mutability, Tier, UnOp};
use crate::source::Span;
use crate::types::{ParamKind, StructId, EnumId, Ty};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalId(pub u32);

/// A checked function instance: one source function, one shape
/// (generic blanks filled, variadic bundle typed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FnInst(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExternId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstId(pub u32);

#[derive(Debug, Clone)]
pub struct Local {
    pub name: String,
    pub ty: Ty,
    pub mutability: Option<Mutability>,
    /// What this local holds: its own value, or a view of someone else's.
    pub kind: LocalKind,
    /// Declared with a generic blank: verbs on a plain value are allowed and do nothing.
    pub from_blank: bool,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalKind {
    Owned,
    Lent,
    LentMut,
    /// A loop counter or element: cannot be assigned.
    LoopVar,
    /// A captured lend inside a function literal.
    Captured,
}

/// One instance's body, ready for MIR.
#[derive(Debug, Clone)]
pub struct Body {
    pub locals: Vec<Local>,
    pub params: Vec<LocalId>,
    pub ret: Ty,
    pub fails: Vec<(String, Option<Ty>)>,
    pub block: Block,
    pub span: Span,
    /// The source function's name, for messages.
    pub name: String,
    pub is_main: bool,
    /// A `test fn` (owner, 2026-09-29): `cca test` runs it; a normal build
    /// leaves it out.
    pub is_test: bool,
    /// A returned borrow (owner, 2026-09-29): whether it may be changed,
    /// and the parameters (by position) the caller's result borrows —
    /// its group's, or, with no group written, the ones it hands back.
    pub ret_lent: Option<(bool, Vec<usize>)>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
    /// The check tier covering this statement (innermost wins).
    pub tier: Option<Tier>,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    /// `'x' … = value;` — a store into a fresh local; a typed `=` is a check site.
    Let { local: LocalId, value: Expr, typed: bool },
    /// `place = value;`
    Assign { place: Place, value: Expr },
    Expr(Expr),
    If { cond: Expr, then: Block, else_: Option<Block> },
    Loop(Block),
    While { cond: Expr, body: Block },
    ForRange { counter: LocalId, start: Expr, stop: Expr, step: Option<Expr>, body: Block, perm: bool, seq: bool, para: bool, nocheat: bool },
    /// Over a list/array/bundle/text-walk/map-walk: `elem` is the element local.
    ForEach { elem: LocalId, pos: Option<LocalId>, iter: Expr, body: Block, perm: bool, seq: bool, para: bool, nocheat: bool },
    Return(Option<Expr>),
    Fail { name: String, index: usize, payload: Option<Expr> },
    Break,
    Continue,
    Block(Block),
    Match { value: Expr, arms: Vec<MatchArm> },
    Concur { detached: bool, captures: Vec<CaptureBinding>, body: Block, otherwise: Option<Otherwise>, fails: Vec<(String, Option<Ty>)> },
    Whichever { arms: Vec<WhicheverArm>, otherwise: Option<Otherwise> },
    InlineAsm { text: String, ins: Vec<(LocalId, String)>, outs: Vec<(LocalId, String)>, lends: Vec<(LocalId, bool, String)>, clobbers: Vec<String> },
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub case: usize,
    pub bind: Option<LocalId>,
    pub body: Block,
    /// The case's value, in a `match` used as one (owner, 2026-09-29).
    pub gives: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct WhicheverArm {
    pub value: LocalId,
    pub channel: Expr,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct CaptureBinding {
    pub verb: crate::ast::Verb,
    pub outer: LocalId,
    pub inner: LocalId,
}

/// A place that can be read, written, lent or given.
#[derive(Debug, Clone)]
pub struct Place {
    pub base: LocalId,
    pub proj: Vec<Proj>,
    pub ty: Ty,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Proj {
    Field(usize),
    /// `[i]` — 1-based; a check site.
    Index(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Otherwise {
    Value(Expr),
    Leave(Box<Stmt>),
    PassUp,
    Arms(Vec<FailArm>),
}

#[derive(Debug, Clone)]
pub struct FailArm {
    pub index: usize,
    pub bind: Option<LocalId>,
    pub body: Block,
    /// The arm's value, when the call's value is used (owner, 2026-09-29).
    pub gives: Option<Expr>,
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: Ty,
    pub span: Span,
}

/// What a call reaches.
#[derive(Debug, Clone)]
pub enum Callee {
    Fn(FnInst),
    Extern(ExternId),
    /// `'f'[…]`: a held function value.
    Value(Box<Expr>),
    /// A language operation: `list.length`, `map.get`, `str.bytes`, `char.code`, `ptr.read`…
    Builtin(Builtin),
}

/// The language's own operations, spelled as calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Builtin {
    /// `list.fill[value, count]`: a list of that many copies (owner,
    /// 2026-09-30).
    ListFill,
    /// A `must` that does not hold, in a `test fn` (owner, 2026-09-29):
    /// the message, reported at the statement's line; the test ends.
    MustReport,
    /// `must.stop`: 1 in the process that runs the expression, 0 in the
    /// one that waits when a check fired there, 2 when none did.
    MustStopFork,
    /// `must.stop`'s expression ran and no check fired: that process ends.
    MustStopDone,
    ListLength,
    ListAppend,
    ListRemove,
    /// `list.slice[lend 'xs', a, b]`: a new list owning copies of
    /// elements a to b (`lists.md`, owner 2026-09-23).
    ListSlice,
    /// `array.slice`: a `list[T]` too (owner; `lists.md`).
    ArraySlice,
    /// `to.list[give 'a']` / `to.array[copy 'l']` (owner, 2026-09-23).
    ToList,
    ToArray,
    ArrayLength,
    MapLength,
    MapHas,
    MapGet,
    MapSet,
    MapRemove,
    MapKeys,
    MapValues,
    MapPairs,
    StrBytes,
    StrPoints,
    StrGraphemes,
    StrEach(crate::types::Unit),
    CharCode,
    /// `on.machine.chip sve` (owner, 2026-10-06): whether the machine
    /// running has it, by its place in `ast::ON_FEATURES`.
    MachineHas(u32),
    CharFromCode(crate::types::Unit),
    PtrRead,
    PtrWrite,
    PtrAt,
    PtrIsNull,
    ChannelSend,
    ChannelTake,
    SimdFill,
    SimdLoad,
    SimdStore,
    SimdAll,
    SimdAny,
    SimdMax,
    SimdMin,
    /// `simd.sum`: worked out in full, checked where it lands (owner,
    /// 2026-10-07: "b"). `.lane` checks it at the lane's type, `.wrap`
    /// wraps it there ("yea").
    SimdSum,
    SimdSumLane,
    SimdSumWrap,
    /// `simd.shuffle['v', [4, 3, 2, 1]]`: the new order, 0-based here.
    SimdShuffle(Vec<u32>),
    /// `simd.load.some` / `simd.store.some`: only the first k lanes.
    SimdLoadSome,
    SimdStoreSome,
    /// `std::…` — refused for now; kept so the checker names it.
    Std(String),
}

#[derive(Debug, Clone)]
pub enum Arg {
    /// A plain value (copies by itself) or an owned value with its verb.
    Value(Expr),
    /// `lend`/`lend.mut` of a place.
    Lend { place: Place, mutable: bool },
    /// `give` of a place: moves out; the place is dead after.
    Give(Place),
    /// `copy` of a place.
    Copy(Place),
    /// `...` of a list or bundle: unpacked at the call.
    Spread { expr: Box<Arg>, count: usize },
}

impl Arg {
    pub fn span(&self) -> Span {
        match self {
            Arg::Value(e) => e.span,
            Arg::Lend { place, .. } | Arg::Give(place) | Arg::Copy(place) => place.span,
            Arg::Spread { expr, .. } => expr.span(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    /// A `match` used as a value: each arm gives it or leaves.
    Match { value: Box<Expr>, arms: Vec<MatchArm> },
    Int(i128),
    /// A whole number past `i128`, as its decimal digits (owner,
    /// 2026-10-03: integers of any width).
    BigInt(String),
    Bin(String),
    /// A `deci` literal in its layout (BID), worked out at its width.
    Deci { bits: u16, value: u128 },
    Str(String),
    Char(String),
    Bool(bool),
    Duration(i128),
    /// A read of a local holding a plain value.
    Local(LocalId),
    /// A read through a place (field of a struct, element of a list): plain result.
    Read(Place),
    Const(ConstId),
    ExternRead(ExternId),
    /// `lend`/`copy`/`give` as a value: the place with its verb.
    Lend { place: Place, mutable: bool },
    Copy(Place),
    Give(Place),
    Call { callee: Callee, args: Vec<Arg>, otherwise: Option<Box<Otherwise>>, fails: Vec<(String, Option<Ty>)> },
    Construct { id: StructId, fields: Vec<Arg> },
    EnumCase { id: EnumId, case: usize, payload: Option<Box<Arg>> },
    List(Vec<Arg>),
    MapLit(Vec<(Arg, Arg)>),
    ListEmpty,
    MapEmpty,
    Channel { elem: Ty, capacity: Option<(u64, u64)> },
    Unary { op: UnOp, expr: Box<Expr> },
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr>, op_span: Span },
    /// A user `op`: a call under an operator's spelling.
    UserOp { inst: FnInst, lhs: Box<Arg>, rhs: Box<Arg> },
    /// `to.T[e]`: `fails` non-empty when the conversion can fail.
    /// `useful`: `to.useful.deci64[…]` or `to.useful.str.utf8[…]`, the useful form.
    Conv { target: Ty, expr: Box<Expr>, otherwise: Option<Box<Otherwise>>, fails: Vec<String>, useful: bool, bits: bool },
    /// A named function as a value.
    FnRef(FnInst),
    /// An anonymous function with its captures.
    FnLit { inst: FnInst, captures: Vec<CaptureBinding> },
    SizeOf { align: bool, ty: Ty },
    /// The same address, at the expression's own pointer type.
    PtrCast(Box<Expr>),
    /// Element `index` of a mixed variadic bundle (an unrolled walk).
    BundleElem { place: Place, index: usize },
    /// A range as a `for` iterable is a statement form; this never reaches MIR.
    Nothing,
}

/// Where a parameter kind is needed for a call: the instance's signature.
#[derive(Debug, Clone)]
pub struct Signature {
    pub params: Vec<(ParamKind, Ty)>,
    pub ret: Ty,
    pub fails: Vec<(String, Option<Ty>)>,
}
