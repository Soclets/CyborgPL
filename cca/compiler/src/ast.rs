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

/// `a.b.c`, `limb.helper`, `std::print.stdout`, `list.length`, `to.int32`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    /// The Limb before `::`, if any.
    pub limb: Option<Ident>,
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
        self.limb.is_none() && self.segs.len() == spelled.len() && self.segs.iter().zip(spelled).all(|(s, w)| s.name == *w)
    }
    pub fn starts(&self, spelled: &[&str]) -> bool {
        self.limb.is_none() && self.segs.len() >= spelled.len() && self.segs.iter().zip(spelled).all(|(s, w)| s.name == *w)
    }
    pub fn spelled(&self) -> String {
        let mut s = String::new();
        if let Some(l) = &self.limb {
            s.push_str(&l.name);
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
    /// `int32`, `str.utf8`, `list[T]`, `map.ordered[K, V]`, `Pair[int32, str.utf8]`,
    /// `shapes.circle.Circle`, `std::uenr`, `ptr.mut[T]`, `channel[T]`.
    Path { path: Path, args: Vec<Type> },
    /// `T[lo, hi]`, `str.utf8[1, 40 graphemes]`, `list[T][0, 10]`.
    Bound { inner: Box<Type>, lo: Box<Expr>, hi: Box<Expr>, unit: Option<Ident>, span: Span },
    /// `<T>` / `<T: Ordered>`.
    Var { name: Ident, bound: Option<Path> },
    /// `fn[lent T, T] -> R or fail[…]`.
    Fn { params: Vec<FnTypeParam>, ret: Option<Box<Type>>, fails: Vec<FailSpec> },
    /// A plain number where a type parameter goes: the lane count of a
    /// `simd[uint8, 16]`. Nothing else takes one.
    Count(u32),
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
    Not,
    NotBit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Pow,
    Div,
    Mod,
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
            BinOp::Add => "+".into(),
            BinOp::Sub => "-".into(),
            BinOp::Mul => "x".into(),
            BinOp::Pow => "xx".into(),
            BinOp::Div => "div".into(),
            BinOp::Mod => "mod".into(),
            BinOp::WrapAdd => "+~".into(),
            BinOp::WrapSub => "-~".into(),
            BinOp::WrapMul => "x~".into(),
            BinOp::WrapPow => "xx~".into(),
            BinOp::Join => "++".into(),
            BinOp::Eq => "==".into(),
            BinOp::Ne => "!==".into(),
            BinOp::Lt => "<".into(),
            BinOp::Gt => ">".into(),
            BinOp::Le => "</==".into(),
            BinOp::Ge => ">/==".into(),
            BinOp::And => "and".into(),
            BinOp::Or => "or".into(),
            BinOp::AndThen => "and.then".into(),
            BinOp::OrElse => "or.else".into(),
            BinOp::AndBit => "and.bit".into(),
            BinOp::OrBit => "or.bit".into(),
            BinOp::Xor => "xor".into(),
            BinOp::Shl => "shl".into(),
            BinOp::Shr => "shr".into(),
            BinOp::Custom(s) => s.clone(),
        }
    }
    pub fn is_comparison(&self) -> bool {
        matches!(self, BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge)
    }
    pub fn is_bitwise(&self) -> bool {
        matches!(self, BinOp::AndBit | BinOp::OrBit | BinOp::Xor | BinOp::Shl | BinOp::Shr)
    }
    pub fn is_multiplicative(&self) -> bool {
        matches!(self, BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::WrapMul)
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
    Str(String),
    Char(String),
    /// The bare `\n`.
    Newline,
    Bool(bool),
    /// `5 seconds`
    Duration { value: String, unit: Ident },
    /// `'x'`
    Name(QName),
    /// `a.'k'`, `shapes.circle.'pi'`, `mathstuff::'pi'`, `limb.'scale'`.
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
    /// `( … )`
    Paren(Box<Expr>),
    /// `[1, 2, 3]`, `[7,]`, `[]`
    List(Vec<Expr>),
    /// `to.int32[…]`, `to.int32[0, 100][…]`, `to.str.utf8[…]`.
    Conv { target: Type, expr: Box<Expr>, otherwise: Option<Box<Otherwise>> },
    /// `channel[T]`, `channel[T][0, N]`
    Channel { elem: Type, bound: Option<Box<(Expr, Expr)>> },
    /// `fn [params] -> T with [captures] { body }`
    FnLit(Box<FnLit>),
    /// `...'xs'`, `...copy 'xs'`
    Spread { verb: Option<Verb>, expr: Box<Expr> },
    /// `pos i` in a `#strict` file: a position, where one goes.
    Pos(Box<Expr>),
    /// `count n`: `list.fill`'s count, labelled in a `#strict` file (owner,
    /// 2026-09-30).
    Count(Box<Expr>),
    /// `size.of[T]` / `align.of[T]`
    SizeOf { align: bool, ty: Type },
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
    Range { start: Expr, stop: Expr, step: Option<Expr> },
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

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    /// A declaration: `let 'x' := …;`, `let mut 'x': T = …;`, `let 'x' $:= …;` (owner,
    /// 2026-09-25: no `let`). `sealed` is `!$` in front: the name may not
    /// be shadowed.
    Let { mutability: Option<Mutability>, name: QName, ty: Option<Type>, shadow: bool, value: Expr, sealed: bool },
    Assign { target: Expr, value: Expr },
    Expr(Expr),
    If { cond: Expr, then: Block, else_: Option<ElseArm> },
    Loop(Block),
    While { cond: Expr, body: Block },
    /// `pos`: `for 'i', 'x' in lend 'xs'` — the position beside the item
    /// (owner, 2026-09-29).
    /// `para`: made parallel by its writer (owner, 2026-09-30); not yet
    /// acted on — every loop runs in order so far.
    For { seq: bool, para: bool, perm: bool, name: QName, pos: Option<QName>, ty: Option<Type>, iter: ForIter, body: Block },
    Return(Option<Expr>),
    Fail { name: Ident, payload: Option<Expr> },
    Break,
    Continue,
    Block(Block),
    Tier { tier: Tier, stmt: Box<Stmt> },
    Concur { detached: bool, captures: Vec<Capture>, body: ConcurBody, otherwise: Option<Otherwise> },
    Whichever { arms: Vec<WhicheverArm>, otherwise: Option<Otherwise> },
    Match { value: Expr, arms: Vec<MatchArm> },
    /// A case's value (owner, 2026-09-29): `{ "free" }` (`bare`), or
    /// `{ gives "free"; }` in a `#strict` file.
    Gives { value: Expr, bare: bool },
    InlineAsm { text: String, bindings: Vec<AsmBinding> },
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumCase {
    pub name: Ident,
    pub payload: Option<Type>,
    /// A `layout.c enum` case's number, as the C header gives it.
    pub number: Option<(i128, Span)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDecl {
    pub name: Ident,
    pub announce: Announce,
    pub cases: Vec<EnumCase>,
    /// `layout.c enum`: C's enum, a number per case (owner, 2026-09-27: a).
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
    Limb,
    Pub,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    Fn(FnDecl),
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
    /// `MAIN { … }`, or `MAIN ['argc': int32, 'argv': ptr[ptr[uint8]]] { … }`
    /// with C's arguments (owner, 2026-09-29).
    Main(Vec<Param>, Block),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub vis: Vis,
    pub kind: ItemKind,
    pub span: Span,
}

/// One parsed `.cpls` file.
#[derive(Debug, Clone, PartialEq)]
pub struct File {
    pub items: Vec<Item>,
    /// `#strict` at its top: which file it is.
    pub strict: Option<crate::source::FileId>,
}
