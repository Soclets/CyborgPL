//! The tree. One node per thing the design names.

#[derive(Debug, Clone, PartialEq)]
/// `Other(n)`: a width that is not standard, written `!std_int24` (owner, 2026-10-03)
pub enum IntWidth { W8, W16, W32, W64, W128, Other(u32) }

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
    /// growable; its kind, plain or always or never page-aligned, kept as it grows (owner, 2026-10-25)
    List(Box<Ty>, ListAlign),
    Array(Box<Ty>),   // fixed-size
    Struct(String),   // a declared blueprint, by name
    /// `ptr[T]` / `ptr.mut[T]` (design/ffi.md): only in an extern signature or a layout.C field
    Ptr(Box<Ty>, bool),
    /// `<T>` — a type variable (design/generics.md, 2026-09-20): a blank the compiler fills
    /// per use; every mention wears the `< >`
    Var(String),
    /// `Stack[i32]` — a generic struct or enum with its blanks filled, in order of first appearance
    Inst(String, Vec<Ty>),
    /// `i32[0, 100]` / `f64[0.0, 1.0]` — a range type (design/input.md, 2026-09-20): the base
    /// type, provably within inclusive bounds; assigning into it is a check site
    Ranged(Box<Ty>, Bound, Bound),
    /// `str.utf8[1, 40 graphemes]` — a length bound on text, in a named unit (decided 2026-09-22)
    TextRanged(Encoding, i128, i128, CharUnit),
    /// `channel[T]` — a typed pipe between tasks (decided 2026-09-22); bounded via `Ranged`
    Channel(Box<Ty>),
    /// `std::duration` — a span of time, from a duration literal `5 seconds`
    Duration,
    /// std's writer (owner, 2026-10-23), `std::print.writer`
    Writer,
    /// `C.str` (owner, 2026-10-19): a C string, its bytes and a 0; bounded `C.str[max: 255 bytes, exclude zero]`:
    /// (low, high, whether the closing 0 counts)
    CStr(Option<(i128, i128, bool)>),
    /// `map[K, V]` — a language collection (decided 2026-09-22); the order variant is the prover's
    /// to pick unless written: `map.ordered` keeps insertion order, `map.unordered` cannot be walked
    Map(Box<Ty>, Box<Ty>, MapOrder),
    /// `simd[u8, count: 16]` — N plain values in one register (design/simd.md, 2026-09-22)
    Simd(Box<Ty>, usize),
    /// `fn[lent T, U] -> R or fail[…]` — a function's type (design/functions-as-values.md,
    /// 2026-09-21): its signature with the names removed
    Fn(Vec<(ParamKind, Ty)>, Option<Box<Ty>>, Vec<String>),
    /// `percent[d32]` / `percent[f64]` (owner, 2026-10-17): a percentage holding that number, its own family;
    /// `%[d32]` the symbol spelling; the held number is the fraction (7.5% holds 0.075)
    Percent(Box<Ty>),
    /// `d64[max_places: 2]`, `percent[d32][min_places: 1]` (owner, 2026-10-17; `places` since 2026-10-20): the decimals a
    /// `d` or a percent keeps at most (a literal with more refused, a result that could have more rounded on purpose)
    /// and at least (printing only); around the type, its bounds inside
    Places(Box<Ty>, Option<u32>, Option<u32>),
}

/// A range type's bound: a numeric literal.
#[derive(Debug, Clone, PartialEq)]
pub enum Bound { Int(i128), Bin(f64), Min, Max }   // `i32[1, max]` — the type's own limit (2026-09-23)

#[derive(Debug, Clone, PartialEq)]
pub enum MapOrder { Any, Ordered, Unordered }

/// `std::list[T]`, `std::list.page_aligned[T]`, `std::list.not_page_aligned[T]` (owner, 2026-10-25): plain is Precog's to
/// place; the two narrowings go anywhere a plain list does, and a plain one becomes either only by `to.list.page_aligned(…)`
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ListAlign { Plain, Page, NotPage }

impl Ty {
    /// The type variables mentioned, in order of first appearance — a generic
    /// struct's or enum's parameter order (decided 2026-09-20: no bracket, first appearance).
    pub fn vars_into(&self, out: &mut Vec<String>) {
        match self {
            Ty::Var(n) => if !out.contains(n) { out.push(n.clone()); },
            Ty::List(t, _) | Ty::Array(t) | Ty::Ptr(t, _) => t.vars_into(out),
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
/// `pub.kit` = the whole kit, `pub` = other kits too.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Vis { Private, Kit, Pub }

/// A module: a file of the kit (`shapes.cy` → `shapes`, `shapes/circle.cy`
/// → `shapes.circle`) or a `module name { … }` block. The root, `kit.cy`,
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
    /// `layout.C enum` (owner, 2026-09-27): each case's number in C; `None` for a Cyborg enum
    pub c_values: Option<Vec<i128>>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    /// canonical: "shapes.Locker" for a struct in module `shapes`, "Locker" at the root
    pub name: String,
    /// `layout.C struct`: C's layout, C-shaped fields only; may cross to C
    pub layout_c: bool,
    /// a C struct CyborgPL never looks inside (owner, 2026-10-10: `ptr.mut[FILE]`): only pointed at, never
    /// built, sized or read; it comes only from a C header, having no spelling of its own
    pub opaque: bool,
    /// `layout.C union` (owner, 2026-10-19): one field at a time; reading a field is a `no_check` site
    pub is_union: bool,
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
/// the prover infers, and stating it is not standard practice.
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
    /// `lent @x 'a'` — the lifetime group this borrow belongs to (owner, 2026-09-29)
    pub life: Option<String>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Func {
    pub name: String,
    pub module: Vec<String>,
    pub vis: Vis,
    pub ret: Option<Ty>,
    /// `-> lent @x T` — the function hands back a borrow: (mutable, its lifetime group if written)
    pub ret_lent: Option<(bool, Option<String>)>,
    /// `-> T or fail[a, b: type]` (decided 2026-09-19): the ways this function can fail.
    /// Empty = cannot fail.
    pub fails: Vec<FailCase>,
    /// `extern.C fn …;` — the body is somewhere else (design/ffi.md); a call is a nocheck site
    pub is_extern: bool,
    /// `op NAME ['a': T, 'b': T] -> R { }` — a custom operator (design/operators.md, 2026-09-20):
    /// used between two operands, never called with `[…]`; operands lent unless `give`
    pub is_op: bool,
    /// `test fn name [] { … }` (owner, 2026-09-29): run by `kayo test`, never called, left out of builds
    pub is_test: bool,
    /// `fn Counter.next` / `fn cum.quick` (2026-10-09), and every member of a `collection`: the last
    /// segment of `module` is the owner's name, not a module, so a const is found in the file around it
    pub owned: bool,
    /// `<T: Ordered>` — each type variable's required interface (canonical), if any
    pub bounds: Vec<(String, String)>,
    pub params: Vec<Param>,
    /// `fn [$s] = $s at_least 60` (owner, 2026-10-24: "(c)"): a lambda of one value, its types taken from where it goes
    pub short: bool,
    pub body: Vec<Stmt>,
    pub line: usize,
    /// `extern.C fn printf_count = C.printf […]` (owner, 2026-10-19): the C function this name calls
    pub link: Option<String>,
}

/// `pub const 'name' := v;` at the top level (decided 2026-09-19): a value
/// known at compile time, baked in at every use, exportable; never assignable.
/// `extern let 'errno': i32;` / `extern let mut 'x': T;` — a C global (design/ffi.md, decided 2026-09-21)
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
    /// `std::print.written[…] otherwise std::print.useful;` (owner, 2026-10-17): print the whole call that way when a
    /// value has no written form; the form's name (`useful`, `stored`)
    PrintAs(String),
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

/// How other kits write the path to a `pub` thing (owner, 2026-10-06): `bare.kit_prefix.never` (the path is
/// required), `bare.kit_prefix.optional`, `bare.kit_prefix.only` (the path is forbidden). There is no default.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Bare { Never, Optional, Only }

/// What a `bare` mark covers: one item (its kind and canonical name), a block module (everything in
/// it), a file (its module and the block modules written in it), or a kind across the kit.
#[derive(Debug, Clone)]
pub enum BareTarget { Item(String, String), Block(Vec<String>), File(Vec<String>), Kind(String) }

/// `case.any` / `case.exact` (owner, 2026-10-06): on an item, a block module, a file, or
/// kinds across the kit (`fn`, `struct`, `enum`, `interface`, `const`, `case`, `field`, `var`), as the
/// `bare` marks are; names are exact unless marked.
#[derive(Debug, Clone)]
pub struct CaseMark { pub target: BareTarget, pub insensitive: bool, pub line: usize, pub kit_root: Vec<String> }

#[derive(Debug, Clone)]
/// `enum_prefix`: the mark is `bare.enum_prefix.…`, about an enum's values dropping the enum's name (owner, 2026-10-20);
/// otherwise `bare.kit_prefix.…`, about a name dropping its kit's path
pub struct BareMark { pub target: BareTarget, pub mode: Bare, pub line: usize, pub kit_root: Vec<String>, pub enum_prefix: bool }

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
    /// `MAIN ['argc': i32, 'argv': ptr[ptr[u8]]]` — C's own arguments, raw (owner, 2026-09-29)
    pub main_args: Option<(String, String)>,
    /// modules written under `#strict` (owner, 2026-09-27)
    pub strict_modules: Vec<Vec<String>>,
    /// `bare.kit_prefix.never` / `bare.kit_prefix.optional` / `bare.kit_prefix.only` marks (owner, 2026-10-06)
    pub bare_marks: Vec<BareMark>,
    pub case_marks: Vec<CaseMark>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    /// `let 'x' := v;` (ty None) or `let 'x': T = v;` (ty Some);
    /// optionally `let mut` / `let immut` before the name (decided 2026-09-19)
    /// `shadow`: written with `$:=` / `$=`, deliberately hiding an outer name
    Decl { name: String, mutability: Option<Mutability>, ty: Option<Ty>, value: Bracket, shadow: bool, sealed: bool, line: usize },
    /// `const 'retries' := 3;` inside a body (owner, 2026-10-10: "I want it", then "`a`": any body):
    /// known at compile time, owned by nobody, never shadowed, seen only in its block
    LocalConst { name: String, ty: Option<Ty>, value: Bracket, line: usize },
    /// `'x' = v;`
    Assign { name: String, value: Bracket, line: usize },
    /// `'xs'[pos: i] = v;`
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
    Range { name: String, perm: bool, seq: bool, ty: Option<Ty>, bounds: Bracket, /** `before`: the end is left out (owner, 2026-10-12) */ exclusive: bool, body: Vec<Stmt>, line: usize },
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
    /// `no_check inline.MSL threads $n { … } with [lend.mut $xs -> buffer(0), copy $k -> buffer(1)];` (owner, 2026-10-01,
    /// 2026-10-22): the thread count, the Metal code, and each binding's verb, variable and buffer slot
    Msl { threads: Bracket, text: String, binds: Vec<(Verb, String, u32)>, line: usize },
    /// `match v { case ['bind'] { … } … }` — every case covered (decided 2026-09-20)
    Match { value: Bracket, arms: Vec<Arm>, line: usize },
    /// `giveS v;` — a case's value, in a `#strict` file (owner, 2026-09-29)
    Gives { value: Bracket, line: usize },
    /// `must 'x' == 5;` — in a `test fn` only (owner, 2026-09-29)
    Must { cond: Expr, line: usize },
    /// `must.fail f[…];` / `must.fail.not_a_number to.i32["x"];` — it must fail (with that reason)
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
    pub kit: Option<String>,   // None = this kit; Some("std") = std::
    /// std: print.stored.stdout → ["print", "stored", "stdout"]; this kit: the canonical path from
    /// the root, `limb.` stripped, the calling module prefixed to a relative one
    pub path: Vec<String>,
    /// the module the call is written in (for visibility)
    pub from: Vec<String>,
    pub args: Bracket,
    pub line: usize,
    /// written with no path (no `x::`, no `my.`), so another kit's `bare.kit_prefix.optional` / `bare.kit_prefix.only`
    /// name may be meant (owner, 2026-10-06); `written` is the path as written
    pub bare: bool,
    pub written: Vec<String>,
    /// a kit function's labelled arguments (owner, 2026-10-21): the label written before each value, if any
    pub labels: Vec<Option<String>>,
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
    /// a whole-number literal in a kit without ASP: a bit pattern, as the machine's (owner, 2026-10-06)
    RawInt(i128, usize),
    /// a literal past i128: its digits (owner, 2026-10-03)
    BigInt(String, usize),
    Bin(f64, String, usize),   // value, digits as written, line
    /// `7.5%` / `exactly 7.5%` (owner, 2026-10-16/17): a percentage literal, the fraction inside (a `Bin` or an
    /// `Exact`); a `percent` under `:=` or into a `percent` type, the plain number into a plain one
    Percent(Box<Expr>, usize),
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
    /// `'xs'[pos: i]` — 1-based
    Index(String, Box<Expr>, usize),
    /// `list.length[…]`, `list.append[…]`, `list.remove[…]`, `array.length[…]`
    Builtin(String, String, Bracket, usize),
    /// `'mine'.field` — read a field
    Field(String, String, usize),
    /// `'mine'.items[pos: i]` — index a field (read only)
    FieldIndex(String, String, Box<Expr>, usize),
    /// `Locker['name' = v, …]` — construct a struct value by field name
    Construct(String, Vec<(String, Bracket)>, usize),
    /// `give 'mine'.field` / `copy 'mine'.field` / `lend 'mine'.field`
    OwnedField(Verb, String, String, usize),
    /// `lend 'xs'[pos: i]` / `copy 'xs'[pos: i]` — an element handed over with a verb
    OwnedElem(Verb, String, Box<Expr>, usize),
    /// `shapes.'pi'` / `kit.'pi'` — a const reached by module path (canonical), from a module
    ConstPath(Vec<String>, String, Vec<String>, usize),
    /// `to.i32[v]` — an explicit conversion to a type (decided 2026-09-19); text → number can
    /// fail (`not_a_number`), so it may carry an `otherwise` handler
    /// `to.T[v]`; the bool is `.useful` (`to.d64.useful[…]`, owner 2026-09-29)
    /// the form: 0 none written, 1 `to.stored.…`, 2 `to.useful.…` (owner, 2026-10-02: text says which)
    Convert(Ty, Bracket, Option<Handler>, u8, usize),
    /// `size.of[T]` / `align.of[T]` — compile-time constants (design/ffi.md)
    SizeOf(Ty, bool, usize),
    /// `incl.bytes["assets/logo.png"]` / `incl.str.utf8["assets/help.txt"]` (owner, 2026-10-10): a
    /// file's bytes, read while building, from the kit's root; the text type for `incl.str.…`, none
    /// for `incl.bytes`
    Incl(std::rc::Rc<Vec<u8>>, Option<Encoding>, usize),
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
            Expr::Int(_, l) | Expr::RawInt(_, l) | Expr::BigInt(_, l) | Expr::Bin(_, _, l) | Expr::Exact(_, l) | Expr::Percent(_, l) | Expr::Bool(_, l) | Expr::Text(_, l) | Expr::Char(_, l)
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
            Expr::SizeOf(_, _, l) | Expr::Incl(_, _, l) => *l,
            Expr::FnRef(_, _, l) | Expr::OpValue(_, _, l) | Expr::Lambda(_, _, l) | Expr::CallValue(_, _, _, l) | Expr::Spread(_, _, l) => *l,
            Expr::CharFromCode(_, _, _, l) | Expr::MapFail(_, _, _, l) | Expr::MapLit(_, l) | Expr::Duration(_, l) | Expr::ChannelNew(_, _, l) => *l,
            Expr::EnumValue(_, _, _, l) => *l,
        }
    }
}
