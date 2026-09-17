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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Encoding { Utf8, Utf16, Utf32, Ascii }

#[derive(Debug, Clone, PartialEq)]
pub enum CharUnit { Byte, Point, Grapheme }

/// How a parameter receives its argument (design/ownership-syntax.md).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParamKind { Var, Lent, LentMut }

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
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Func {
    pub name: String,
    pub ret: Option<Ty>,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub funcs: Vec<Func>,
    pub main: Option<Vec<Stmt>>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    /// `var 'x' := [v];` (ty None) or `var.T 'x' = [v];` (ty Some);
    /// optionally `var.mut` / `var.immut` before the type
    /// `shadow`: written with `$:=` / `$=`, deliberately hiding an outer name
    Decl { name: String, mutability: Option<Mutability>, ty: Option<Ty>, value: Bracket, shadow: bool, line: usize },
    /// `'x' = [v];`
    Assign { name: String, value: Bracket, line: usize },
    /// `'xs'[pos i] = [v];`
    AssignElem { name: String, index: Expr, value: Bracket, line: usize },
    If { arms: Vec<(Bracket, Vec<Stmt>)>, otherwise: Option<Vec<Stmt>>, line: usize },
    Loop { body: Vec<Stmt>, line: usize },
    While { cond: Bracket, body: Vec<Stmt>, line: usize },
    /// `loop.temp.for` / `loop.perm.for`: `perm` keeps the variable after the loop
    For { name: String, perm: bool, seq: bool, over: Bracket, body: Vec<Stmt>, line: usize },
    /// `loop.temp.range 'i' [:= start, := stop, := step] { }` — inclusive stop
    Range { name: String, perm: bool, seq: bool, ty: Option<Ty>, bounds: Bracket, body: Vec<Stmt>, line: usize },
    Break { line: usize },
    Continue { line: usize },
    Return { value: Option<Bracket>, line: usize },
    /// a call used as a statement: `add[1, 2];` or `std::print.stdout[...];`
    Call { call: Call, line: usize },
    /// `list.append[…];` etc. as a statement
    BuiltinStmt { head: String, op: String, args: Bracket, line: usize },
}

#[derive(Debug, Clone)]
pub struct Call {
    pub limb: Option<String>,   // None = this Limb; Some("std") = std::
    pub path: Vec<String>,      // print.stdout → ["print", "stdout"]
    pub args: Bracket,
    pub line: usize,
}

/// `[ … ]` — one or more comma-separated values. One value is that value;
/// more than one is a list.
#[derive(Debug, Clone)]
pub struct Bracket {
    pub values: Vec<Value>,
    pub line: usize,
    /// `[1,]` — a single value with a trailing comma is a one-element list
    pub one_list: bool,
}

/// One comma-separated slot: either a single expression, or several
/// whitespace-separated pieces that join into one text.
#[derive(Debug, Clone)]
pub enum Value {
    Expr(Expr),
    Pieces(Vec<Expr>),
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
        }
    }
}
