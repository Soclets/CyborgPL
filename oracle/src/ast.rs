//! The tree. One node per thing the design names.

#[derive(Debug, Clone, PartialEq)]
pub enum IntWidth { W8, W16, W32, W64, W128 }

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Int(IntWidth),
    Uint(IntWidth),
    Bin(u8),          // 16 32 64 128
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
    /// `fn[lent T, U] -> R or fail[…]` — a function's type (design/functions-as-values.md,
    /// 2026-09-21): its signature with the names removed
    Fn(Vec<(ParamKind, Ty)>, Option<Box<Ty>>, Vec<String>),
}

/// A range type's bound: a numeric literal.
#[derive(Debug, Clone, PartialEq)]
pub enum Bound { Int(i128), Bin(f64) }

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
    /// type variables, in order of first appearance (design/generics.md)
    pub params: Vec<String>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    /// canonical: "shapes.Locker" for a struct in module `shapes`, "Locker" at the root
    pub name: String,
    /// `layout.c struct`: C's layout, C-shaped fields only; may cross to C
    pub layout_c: bool,
    /// type variables, in order of first appearance (design/generics.md)
    pub params: Vec<String>,
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
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Func {
    pub name: String,
    pub module: Vec<String>,
    pub vis: Vis,
    pub ret: Option<Ty>,
    /// `-> T or fail[a, b: type]` (decided 2026-09-19): the ways this function can fail.
    /// Empty = cannot fail.
    pub fails: Vec<FailCase>,
    /// `extern fn …;` — the body is somewhere else (design/ffi.md); a call is a nocheck site
    pub is_extern: bool,
    /// `op NAME ['a': T, 'b': T] -> R { }` — a custom operator (design/operators.md, 2026-09-20):
    /// used between two operands, never called with `[…]`; operands lent unless `give`
    pub is_op: bool,
    /// `<T: Ordered>` — each type variable's required interface (canonical), if any
    pub bounds: Vec<(String, String)>,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub line: usize,
}

/// `pub const 'name' := v;` at the top level (decided 2026-09-19): a value
/// known at compile time, baked in at every use, exportable; never assignable.
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
    pub modules: Vec<ModuleDef>,
    pub main: Option<Vec<Stmt>>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    /// `let 'x' := v;` (ty None) or `let 'x': T = v;` (ty Some);
    /// optionally `let mut` / `let immut` before the name (decided 2026-09-19)
    /// `shadow`: written with `$:=` / `$=`, deliberately hiding an outer name
    Decl { name: String, mutability: Option<Mutability>, ty: Option<Ty>, value: Bracket, shadow: bool, line: usize },
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
    For { name: String, perm: bool, seq: bool, over: Bracket, body: Vec<Stmt>, line: usize },
    /// `for 'i' in range[start, stop, step] { }` — inclusive stop; `'i': T` types the counter
    Range { name: String, perm: bool, seq: bool, ty: Option<Ty>, bounds: Bracket, body: Vec<Stmt>, line: usize },
    Break { line: usize },
    Continue { line: usize },
    Return { value: Option<Bracket>, line: usize },
    /// a call used as a statement: `add[1, 2];` or `std::print.stdout[...];`
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
    /// `'f'[args];` — a function value called as a statement
    CallValue { name: String, args: Bracket, handler: Option<Handler>, line: usize },
}

#[derive(Debug, Clone)]
pub enum AsmBind {
    In(String, String),      // 'v' -> reg
    Out(String, String),     // 'v' <- reg   (declares 'v')
    InOut(String, String),   // 'v' <-> reg
    Clobber(String),         // clobber reg
}

/// The three check tiers (decided 2026-09-14): default is written as nothing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tier { Check, NoCheck }

#[derive(Debug, Clone)]
pub struct Call {
    /// `otherwise …` after the call, if any
    pub handler: Option<Handler>,
    pub limb: Option<String>,   // None = this Limb; Some("std") = std::
    /// std: print.stdout → ["print", "stdout"]; this Limb: the canonical path from
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
    Bin(f64, usize),
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
    Convert(Ty, Bracket, Option<Handler>, usize),
    /// `size.of[T]` / `align.of[T]` — compile-time constants (design/ffi.md)
    SizeOf(Ty, bool, usize),
    /// `average` — a function as a value (canonical path); bare name, no brackets
    FnRef(Vec<String>, Vec<String>, usize),
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
            Expr::Int(_, l) | Expr::Bin(_, l) | Expr::Bool(_, l) | Expr::Text(_, l) | Expr::Char(_, l)
            | Expr::Newline(l) | Expr::Var(_, l) | Expr::Owned(_, _, l)
            | Expr::Unary(_, _, l) | Expr::Binary(_, _, _, l) | Expr::Paren(_, l) => *l,
            Expr::Call(c) => c.line,
            Expr::Nested(b) => b.line,
            Expr::Index(_, _, l) => *l,
            Expr::Builtin(_, _, _, l) => *l,
            Expr::Field(_, _, l) => *l,
            Expr::FieldIndex(_, _, _, l) => *l,
            Expr::Construct(_, _, l) => *l,
            Expr::OwnedField(_, _, _, l) => *l,
            Expr::OwnedElem(_, _, _, l) => *l,
            Expr::ConstPath(_, _, _, l) => *l,
            Expr::Convert(_, _, _, l) => *l,
            Expr::SizeOf(_, _, l) => *l,
            Expr::FnRef(_, _, l) | Expr::Lambda(_, _, l) | Expr::CallValue(_, _, _, l) | Expr::Spread(_, _, l) => *l,
            Expr::EnumValue(_, _, _, l) => *l,
        }
    }
}
