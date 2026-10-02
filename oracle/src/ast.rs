//! The tree. One node per thing the design names.

#[derive(Debug, Clone, PartialEq)]
pub enum IntWidth { W8, W16, W32, W64, W128 }

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Int(IntWidth),
    Uint(IntWidth),
    Bin(u8),          // 16 32 64 128
    /// `brain16` (also `bfloat16`, `bf16`) — the brain float (owner, 2026-10-02): a family of its own
    Brain,
    Deci(u8),         // 32 64 128 — accepted by the parser, refused by the interpreter
    Str(Encoding),
    Bool,
    Char(CharUnit),
    List(Box<Ty>),    // growable
    Array(Box<Ty>),   // fixed-size
    Struct(String),   // a declared blueprint, by name
    /// `ptr[T]` / `ptr.mut[T]` (design/ffi.md): only in an extern signature or a layout.c field
    Ptr(Box<Ty>, bool),
    /// `<T>` — a type variable (design/generics.md, 2026-09-20): a blank the compiler fills
    /// per use; every mention wears the `< >`
    Var(String),
    /// `Stack[int32]` — a generic struct or enum with its blanks filled, in order of first appearance
    Inst(String, Vec<Ty>),
    /// `int32[0, 100]` / `bin64[0.0, 1.0]` — a range type (design/input.md, 2026-09-20): the base
    /// type, provably within inclusive bounds; assigning into it is a check site
    Ranged(Box<Ty>, Bound, Bound),
    /// `str.utf8[1, 40 graphemes]` — a length bound on text, in a named unit (decided 2026-09-22)
    TextRanged(Encoding, i128, i128, CharUnit),
    /// `channel[T]` — a typed pipe between tasks (decided 2026-09-22); bounded via `Ranged`
    Channel(Box<Ty>),
    /// `std::duration` — a span of time, from a duration literal `5 seconds`
    Duration,
    /// `map[K, V]` — a language collection (decided 2026-09-22); the order variant is Precog's
    /// to pick unless written: `map.ordered` keeps insertion order, `map.unordered` cannot be walked
    Map(Box<Ty>, Box<Ty>, MapOrder),
    /// `simd[uint8, 16]` — N plain values in one register (design/simd.md, 2026-09-22)
    Simd(Box<Ty>, usize),
    /// `fn[lent T, U] -> R or fail[…]` — a function's type (design/functions-as-values.md,
    /// 2026-09-21): its signature with the names removed
    Fn(Vec<(ParamKind, Ty)>, Option<Box<Ty>>, Vec<String>),
}

/// A range type's bound: a numeric literal.
#[derive(Debug, Clone, PartialEq)]
pub enum Bound { Int(i128), Bin(f64), Min, Max }   // `int32[1, max]` — the type's own limit (2026-09-23)

#[derive(Debug, Clone, PartialEq)]
pub enum MapOrder { Any, Ordered, Unordered }

impl Ty {
    /// The type variables mentioned, in order of first appearance — a generic
    /// struct's or enum's parameter order (decided 2026-09-20: no bracket, first appearance).
    pub fn vars_into(&self, out: &mut Vec<String>) {
        match self {
            Ty::Var(n) => if !out.contains(n) { out.push(n.clone()); },
            Ty::List(t) | Ty::Array(t) | Ty::Ptr(t, _) => t.vars_into(out),
            Ty::Inst(_, args) => for a in args { a.vars_into(out); },
            Ty::Fn(ps, r, _) => { for (_, t) in ps { t.vars_into(out); } if let Some(r) = r { r.vars_into(out); } }
            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    pub ty: Ty,
    pub name: String,
    pub immut: bool,
}

/// Visibility (decided 2026-09-19): nothing = this module (and its descendants),
/// `pub.limb` = the whole Limb, `pub` = other Limbs too.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Vis { Private, Limb, Pub }

/// A module: a file of the Limb (`shapes.cpls` → `shapes`, `shapes/circle.cpls`
/// → `shapes.circle`) or a `module name { … }` block. The root, `limb.cpls`,
/// is the empty path.
#[derive(Debug, Clone)]
pub struct ModuleDef {
    pub path: Vec<String>,
    pub vis: Vis,
    pub line: usize,
}

/// `enum Status { empty  claimed: str.utf8  locked }` (decided 2026-09-20)
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,        // canonical, like a struct's
    pub module: Vec<String>,
    pub vis: Vis,
    pub cases: Vec<FailCase>, // a case is a name with an optional payload — the failure shape
    /// type variables, in the order announced (design/generics.md, 2026-10-02)
    pub params: Vec<String>,
    pub bounds: Vec<(String, String)>,
    /// `layout.c enum` (owner, 2026-09-27): each case's number in C; `None` for a Cyborg enum
    pub c_values: Option<Vec<i128>>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    /// canonical: "shapes.Locker" for a struct in module `shapes`, "Locker" at the root
    pub name: String,
    /// `layout.c struct`: C's layout, C-shaped fields only; may cross to C
    pub layout_c: bool,
    /// type variables, in the order announced (design/generics.md, 2026-10-02)
    pub params: Vec<String>,
    /// what each accepts: (blank, interface), one pair per interface; `any` adds none
    pub bounds: Vec<(String, String)>,
    pub module: Vec<String>,
    pub vis: Vis,
    pub fields: Vec<Field>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Encoding { Utf8, Utf16, Utf32, Ascii }

#[derive(Debug, Clone, PartialEq)]
pub enum CharUnit { Byte, Point, Grapheme }

/// How a parameter receives its argument (design/ownership-syntax.md).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParamKind { Owned, Lent, LentMut }

/// What a caller does with an owned value (design/ownership-syntax.md).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Verb { Give, Copy, Lend, LendMut }

/// Stated mutability (design/ownership-syntax.md). `None` = not stated;
/// Precog infers, and stating it is not standard practice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mutability { Mut, Immut }

#[derive(Debug, Clone)]
pub struct Param {
    pub kind: ParamKind,
    pub mutability: Option<Mutability>,
    pub ty: Option<Ty>,
    pub name: String,
    /// `...'name'` (decided 2026-09-19): the rest of the arguments, any number, each
    /// keeping its own type unless `: type` says otherwise; last parameter only
    pub variadic: bool,
    /// `lent@x 'a'` — the lifetime group this borrow belongs to (owner, 2026-09-29)
    pub life: Option<String>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Func {
    pub name: String,
    pub module: Vec<String>,
    pub vis: Vis,
    pub ret: Option<Ty>,
    /// `-> lent@x T` — the function hands back a borrow: (mutable, its lifetime group if written)
    pub ret_lent: Option<(bool, Option<String>)>,
    /// `-> T or fail[a, b: type]` (decided 2026-09-19): the ways this function can fail.
    /// Empty = cannot fail.
    pub fails: Vec<FailCase>,
    /// `extern fn …;` — the body is somewhere else (design/ffi.md); a call is a nocheck site
    pub is_extern: bool,
    /// `op NAME ['a': T, 'b': T] -> R { }` — a custom operator (design/operators.md, 2026-09-20):
    /// used between two operands, never called with `[…]`; operands lent unless `give`
    pub is_op: bool,
    /// `test fn name [] { … }` (owner, 2026-09-29): run by `cca test`, never called, left out of builds
    pub is_test: bool,
    /// `<T: Ordered>` — each type variable's required interface (canonical), if any
    pub bounds: Vec<(String, String)>,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub line: usize,
}

/// `pub const 'name' := v;` at the top level (decided 2026-09-19): a value
/// known at compile time, baked in at every use, exportable; never assignable.
/// `extern let 'errno': int32;` / `extern let mut 'x': T;` — a C global (design/ffi.md, decided 2026-09-21)
#[derive(Debug, Clone)]
pub struct ExternVar {
    pub name: String,
    pub module: Vec<String>,
    pub vis: Vis,
    pub ty: Ty,
    pub mutable: bool,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct ConstDef {
    pub name: String,
    pub module: Vec<String>,
    pub vis: Vis,
    pub ty: Option<Ty>,
    pub value: Bracket,
    pub line: usize,
}

/// One way a function can fail: a name, optionally carrying a payload.
#[derive(Debug, Clone)]
pub struct FailCase {
    pub name: String,
    pub ty: Option<Ty>,
}

/// What a caller wrote after `otherwise` (decided 2026-09-19). A call that can
/// fail must carry one; a call that cannot must not.
#[derive(Debug, Clone)]
pub enum Handler {
    /// `otherwise <value>` — use this value instead
    Value(Bracket),
    /// `otherwise return …;` / `break;` / `continue;` / `fail …;` — whatever the reason, leave
    Leave(Box<Stmt>),
    /// `otherwise fail;` — fail the same way (the enclosing fn must list the names)
    Propagate,
    /// `otherwise { name ['bind'] { … } … }` — one arm per reason, all covered
    Arms(Vec<Arm>),
}

#[derive(Debug, Clone)]
pub struct Arm {
    pub name: String,
    pub bind: Option<String>,
    /// `empty { "free" }` — a case that is just a value (owner, 2026-09-29)
    pub value: Option<Expr>,
    pub body: Vec<Stmt>,
    pub line: usize,
}

/// `interface Name { fn …;  op …; }` (design/interfaces.md, 2026-09-20): a named list of
/// promises about some `<T>`; a type keeps it by having the operations.
#[derive(Debug, Clone)]
pub struct InterfaceDef {
    pub name: String,        // canonical
    pub module: Vec<String>,
    pub vis: Vis,
    pub entries: Vec<Func>,  // signatures only; `is_op` tells fn from op
    /// the types it accepts, when it lists them (owner, 2026-10-02); empty: any that keeps the entries
    pub types: Vec<Ty>,
    pub line: usize,
}

/// `Vec satisfies Ordered;` — an optional, verified claim.
#[derive(Debug, Clone)]
pub struct Claim {
    pub ty: Ty,
    pub iface: String,       // canonical
    pub line: usize,
}

pub struct Program {
    pub funcs: Vec<Func>,
    pub structs: Vec<StructDef>,
    pub enums: Vec<EnumDef>,
    pub interfaces: Vec<InterfaceDef>,
    pub claims: Vec<Claim>,
    pub consts: Vec<ConstDef>,
    pub externs: Vec<ExternVar>,
    pub modules: Vec<ModuleDef>,
    pub main: Option<Vec<Stmt>>,
    /// `MAIN ['argc': int32, 'argv': ptr[ptr[uint8]]]` — C's own arguments, raw (owner, 2026-09-29)
    pub main_args: Option<(String, String)>,
    /// modules written under `#strict` (owner, 2026-09-27)
    pub strict_modules: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    /// `let 'x' := v;` (ty None) or `let 'x': T = v;` (ty Some);
    /// optionally `let mut` / `let immut` before the name (decided 2026-09-19)
    /// `shadow`: written with `$:=` / `$=`, deliberately hiding an outer name
    Decl { name: String, mutability: Option<Mutability>, ty: Option<Ty>, value: Bracket, shadow: bool, sealed: bool, line: usize },
    /// `'x' = v;`
    Assign { name: String, value: Bracket, line: usize },
    /// `'xs'[pos i] = v;`
    AssignElem { name: String, index: Expr, value: Bracket, line: usize },
    /// `'mine'.field = v;`
    AssignField { name: String, field: String, value: Bracket, line: usize },
    If { arms: Vec<(Bracket, Vec<Stmt>)>, otherwise: Option<Vec<Stmt>>, line: usize },
    Loop { body: Vec<Stmt>, line: usize },
    While { cond: Bracket, body: Vec<Stmt>, line: usize },
    /// `for 'x' in lend 'xs' { }` / `for perm …`: `perm` keeps the variable after the loop (temp is the default)
    /// `index`: `for 'i', 'x' in lend 'xs'` — the position (1, 2, …) beside the item (owner, 2026-09-29)
    For { name: String, index: Option<String>, perm: bool, seq: bool, over: Bracket, body: Vec<Stmt>, line: usize },
    /// `for 'i' in range[start, stop, step] { }` — inclusive stop; `'i': T` types the counter
    Range { name: String, perm: bool, seq: bool, ty: Option<Ty>, bounds: Bracket, body: Vec<Stmt>, line: usize },
    Break { line: usize },
    Continue { line: usize },
    Return { value: Option<Bracket>, line: usize },
    /// a call used as a statement: `add[1, 2];` or `std::print.stored.stdout[...];`
    Call { call: Call, line: usize },
    /// `list.append[…];` etc. as a statement
    BuiltinStmt { head: String, op: String, args: Bracket, line: usize },
    /// `check <stmt>` / `nocheck <stmt>` (decided 2026-09-19): a bare word before a
    /// statement, covering every check site inside it; a `{ }` block is a statement
    Tier { tier: Tier, body: Box<Stmt>, line: usize },
    /// a bare `{ }` block as a statement (needed so `check { … }` is just the prefix rule)
    Block { body: Vec<Stmt>, line: usize },
    /// `fail name;` / `fail name[payload];` — leave the function with a failure (decided 2026-09-19)
    Fail { name: String, payload: Option<Bracket>, line: usize },
    /// `asm { … } with [ 'v' -> reg, 'v' <- reg, 'v' <-> reg, clobber reg ];` (design/ffi.md, 2026-09-20)
    Asm { text: String, binds: Vec<AsmBind>, line: usize },
    /// `match v { case ['bind'] { … } … }` — every case covered (decided 2026-09-20)
    Match { value: Bracket, arms: Vec<Arm>, line: usize },
    /// `giveS v;` — a case's value, in a `#strict` file (owner, 2026-09-29)
    Gives { value: Bracket, line: usize },
    /// `must 'x' == 5;` — in a `test fn` only (owner, 2026-09-29)
    Must { cond: Expr, line: usize },
    /// `must.fail f[…];` / `must.fail.not_a_number to.int32["x"];` — it must fail (with that reason)
    MustFail { expr: Expr, reason: Option<String>, line: usize },
    /// `must.stop 'xs'[99];` — a check site must fire
    MustStop { expr: Expr, line: usize },
    /// `concur [with […]] { … }` / `concur f[…];` — start a task (design/concurrency.md, 2026-09-22)
    Concur { detached: bool, caps: Vec<(Verb, String)>, body: Vec<Stmt>, handler: Option<Handler>, line: usize },
    /// `whichever { 'v' from lend 'c' { … } … }` — wait on several channels, take from whichever first
    Whichever { arms: Vec<WhicheverArm>, handler: Option<Handler>, line: usize },
    /// `map.remove[…] otherwise …;` — a failing map operation as a statement, value dropped
    MapFailStmt { op: String, args: Bracket, handler: Option<Handler>, line: usize },
    /// `'f'[args];` — a function value called as a statement
    CallValue { name: String, args: Bracket, handler: Option<Handler>, line: usize },
}

#[derive(Debug, Clone)]
pub struct WhicheverArm { pub name: String, pub chan: Bracket, pub body: Vec<Stmt>, pub line: usize }

#[derive(Debug, Clone)]
pub enum AsmBind {
    In(String, String),      // 'v' -> reg
    Out(String, String),     // 'v' <- reg   (declares 'v')
    InOut(String, String),   // 'v' <-> reg
    Clobber(String),         // clobber reg
    Lend(String, String, bool), // lend 'v' -> reg: the address of the lent data (bool = lend.mut; owner, 2026-09-30: a)
}

/// The three check tiers (decided 2026-09-14): default is written as nothing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tier { Check, NoCheck, NoChecksAtAll }   // NO_FUCKING_CHECKS (owner, 2026-09-27)

#[derive(Debug, Clone)]
pub struct Call {
    /// `otherwise …` after the call, if any
    pub handler: Option<Handler>,
    pub limb: Option<String>,   // None = this Limb; Some("std") = std::
    /// std: print.stored.stdout → ["print", "stored", "stdout"]; this Limb: the canonical path from
    /// the root, `limb.` stripped, the calling module prefixed to a relative one
    pub path: Vec<String>,
    /// the module the call is written in (for visibility)
    pub from: Vec<String>,
    pub args: Bracket,
    pub line: usize,
}

/// `[ … ]` — one or more comma-separated values. One value is that value;
/// more than one is a list. Since 2026-09-19 a bare single expression is also
/// carried as a one-value Bracket: `[ ]` is only written where it builds.
#[derive(Debug, Clone)]
pub struct Bracket {
    pub values: Vec<Value>,
    pub line: usize,
    /// `[1,]` — a single value with a trailing comma is a one-element list
    pub one_list: bool,
}

/// One comma-separated slot. (Until 2026-09-19 several whitespace-separated
/// pieces joined into one text here; joining is now explicit with `++`.)
#[derive(Debug, Clone)]
pub enum Value {
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i128, usize),
    Bin(f64, String, usize),   // value, digits as written, line
    /// `*0.1` — "exactly this" (owner, 2026-10-02): a decimal literal that is a `deci`; its digits, line
    Exact(String, usize),
    Bool(bool, usize),
    Text(String, usize),
    Char(String, usize),
    Newline(usize),
    Var(String, usize),
    /// `give 'x'`, `copy 'x'`, `lend 'x'`, `lend.mut 'x'`
    Owned(Verb, String, usize),
    Call(Call),
    /// a `[ ]` used as a value inside another bracket: a list literal, or just grouping
    Nested(Bracket),
    /// `'xs'[pos i]` — 1-based
    Index(String, Box<Expr>, usize),
    /// `list.length[…]`, `list.append[…]`, `list.remove[…]`, `array.length[…]`
    Builtin(String, String, Bracket, usize),
    /// `'mine'.field` — read a field
    Field(String, String, usize),
    /// `'mine'.items[pos i]` — index a field (read only)
    FieldIndex(String, String, Box<Expr>, usize),
    /// `Locker['name' = v, …]` — construct a struct value by field name
    Construct(String, Vec<(String, Bracket)>, usize),
    /// `give 'mine'.field` / `copy 'mine'.field` / `lend 'mine'.field`
    OwnedField(Verb, String, String, usize),
    /// `lend 'xs'[pos i]` / `copy 'xs'[pos i]` — an element handed over with a verb
    OwnedElem(Verb, String, Box<Expr>, usize),
    /// `shapes.'pi'` / `limb.'pi'` — a const reached by module path (canonical), from a module
    ConstPath(Vec<String>, String, Vec<String>, usize),
    /// `to.int32[v]` — an explicit conversion to a type (decided 2026-09-19); text → number can
    /// fail (`not_a_number`), so it may carry an `otherwise` handler
    /// `to.T[v]`; the bool is `.useful` (`to.deci64.useful[…]`, owner 2026-09-29)
    Convert(Ty, Bracket, Option<Handler>, bool, usize),
    /// `size.of[T]` / `align.of[T]` — compile-time constants (design/ffi.md)
    SizeOf(Ty, bool, usize),
    /// `average` — a function as a value (canonical path); bare name, no brackets
    FnRef(Vec<String>, Vec<String>, usize),
    /// `op +` / `op neg` / `op dot` — an operator as a value (owner, 2026-09-29): the built-in
    /// ones as a generic function the parser writes; a custom one (None) is looked up by name
    OpValue(String, Option<std::rc::Rc<Func>>, usize),
    /// `5 seconds` — a duration literal, in nanoseconds (decided 2026-09-22)
    Duration(i128, usize),
    /// `channel[T]` / `channel[T][0, N]` — a new channel (hand-off, or a bounded queue)
    ChannelNew(Ty, Option<usize>, usize),
    /// `map["ann" = 5551, "bob" = 5552]` — a map literal (decided 2026-09-22): key = value pairs
    MapLit(Vec<(Expr, Bracket)>, usize),
    /// `map.get[lend 'm', k]` / `map.remove[lend.mut 'm', k]`: hand a value back or fail `missing` (decided 2026-09-22)
    MapFail(String, Bracket, Option<Handler>, usize),
    /// `ptr.read['p'] otherwise …` — a read of a C enum or range type from C, which can fail
    /// `out_of_range` (owner, 2026-09-27/28)
    PtrReadChecked(Bracket, Option<Handler>, usize),
    /// `ptr.cast[T]['p']` — the same address, seen as pointing at a T (owner, 2026-09-30: a)
    PtrCast(Ty, Bracket, usize),
    /// `'x' := match v { case { value } … }` — a match that gives a value (owner, 2026-09-29)
    Match(Bracket, Vec<Arm>, usize),
    /// `char.point.from_code[n]` / `char.byte.from_code[n]`: a number to a character, fails `not_a_char` (decided 2026-09-21)
    CharFromCode(CharUnit, Bracket, Option<Handler>, usize),
    /// `...'xs'` in a call: unpack the list into the variadic part (decided 2026-09-21)
    Spread(String, Option<Verb>, usize),
    /// `fn [params] -> T with [lend 'v', give 'w'] { … }` — an anonymous function with explicit captures
    Lambda(std::rc::Rc<Func>, Vec<(Verb, String)>, usize),
    /// `'f'[args]` — calling a function value held in a variable
    CallValue(String, Bracket, Option<Handler>, usize),
    /// `Status.empty` / `Status.claimed["x"]` — an enum value (canonical type name, case, data)
    EnumValue(String, String, Option<Bracket>, usize),
    Unary(String, Box<Expr>, usize),            // not, -
    Binary(String, Box<Expr>, Box<Expr>, usize),
    Paren(Box<Expr>, usize),
}

impl Expr {
    pub fn line(&self) -> usize {
        match self {
            Expr::Int(_, l) | Expr::Bin(_, _, l) | Expr::Exact(_, l) | Expr::Bool(_, l) | Expr::Text(_, l) | Expr::Char(_, l)
            | Expr::Newline(l) | Expr::Var(_, l) | Expr::Owned(_, _, l)
            | Expr::Unary(_, _, l) | Expr::Binary(_, _, _, l) | Expr::Paren(_, l) => *l,
            Expr::Call(c) => c.line,
            Expr::Match(_, _, l) => *l,
            Expr::PtrReadChecked(_, _, l) => *l,
            Expr::PtrCast(_, _, l) => *l,
            Expr::Nested(b) => b.line,
            Expr::Index(_, _, l) => *l,
            Expr::Builtin(_, _, _, l) => *l,
            Expr::Field(_, _, l) => *l,
            Expr::FieldIndex(_, _, _, l) => *l,
            Expr::Construct(_, _, l) => *l,
            Expr::OwnedField(_, _, _, l) => *l,
            Expr::OwnedElem(_, _, _, l) => *l,
            Expr::ConstPath(_, _, _, l) => *l,
            Expr::Convert(_, _, _, _, l) => *l,
            Expr::SizeOf(_, _, l) => *l,
            Expr::FnRef(_, _, l) | Expr::OpValue(_, _, l) | Expr::Lambda(_, _, l) | Expr::CallValue(_, _, _, l) | Expr::Spread(_, _, l) => *l,
            Expr::CharFromCode(_, _, _, l) | Expr::MapFail(_, _, _, l) | Expr::MapLit(_, l) | Expr::Duration(_, l) | Expr::ChannelNew(_, _, l) => *l,
            Expr::EnumValue(_, _, _, l) => *l,
        }
    }
}
