//! MIR — the control-flow graph Precog reads and LLVM is lowered from.
//!
//! `design/precog.md` (owner, 2026-09-22, three IRs): basic blocks,
//! explicit jumps, one operation per instruction on named temporaries,
//! **every check site, drop, lend and failure edge an explicit
//! instruction**, a source span on each. A temporary's type carries its
//! **proven domain**, and a proof made in Precog rides the instruction to
//! LLVM. **Precog is the last MIR pass**: after it, MIR is frozen and
//! lowered as is, so every MIR-level optimisation runs *before* Precog and
//! Precog proves exactly what is emitted.
//!
//! Spans live in a side table (`Function::spans`) and instructions carry a
//! `SpanId`, which is a position, not a byte offset. So a comment shifts
//! the table's contents and leaves the instruction stream identical —
//! which is what makes the reuse key in `precog.md` hash canonical MIR
//! without a comment invalidating every proof.

pub mod borrows;
pub mod build;
pub mod channels;
pub mod domain;
pub mod ownership;
pub mod print;
pub mod verify;

use crate::ast::{BinOp, Tier, UnOp};
use crate::hir::{ConstId, ExternId, FnInst};
use crate::source::Span;
use crate::types::{EnumId, StructId, Ty, Unit};
use domain::Domain;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LocalId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SiteId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SpanId(pub u32);
/// A function in the program's arena: one instance, or one task body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FnIdx(pub u32);

/// What a local is for. A temporary is the compiler's; a named local is
/// the writer's and appears in messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalKind {
    /// Holds its own value and is dropped when its scope ends.
    Owned,
    /// A view of someone else's value: never dropped.
    Lent { mutable: bool },
    /// A plain value: nothing behind it to free.
    Plain,
    /// The result of one operation.
    Temp,
    /// The return slot.
    Return,
}

#[derive(Debug, Clone)]
pub struct LocalDecl {
    /// The writer's name, when it has one.
    pub name: Option<String>,
    pub ty: Ty,
    /// What Precog knows about it. Starts as the type's own domain.
    pub domain: Domain,
    pub kind: LocalKind,
    pub span: SpanId,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Proj {
    Field(usize),
    /// `[i]` — 1-based; guarded by a `Bounds` site.
    Index(Operand),
    /// One element of a mixed variadic bundle.
    BundleElem(usize),
    /// Through a `ptr[T]` from C.
    Deref,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    pub local: LocalId,
    pub proj: Vec<Proj>,
}

impl Place {
    pub fn local(l: LocalId) -> Place {
        Place { local: l, proj: Vec::new() }
    }
    pub fn is_local(&self) -> bool {
        self.proj.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Const {
    Int(i128),
    /// Kept as written: compile-time arithmetic rounds at the target's
    /// width, never the host's (`reproducible-builds.md`).
    Bin(String),
    /// A `deci` constant in its layout (BID), and its width.
    Deci { bits: u16, value: u128 },
    Str(String),
    Char(String),
    Bool(bool),
    /// Nanoseconds.
    Duration(i128),
    /// A Limb-level value, folded in at every use.
    Const(ConstId),
    SizeOf { align: bool, ty: Ty },
    Nothing,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Const(Const),
    /// A plain value read out of a place: a register copy, nothing owned.
    Read(Place),
    /// `give` — ownership transfers; the place is dead after this.
    Move(Place),
}

impl Operand {
    pub fn place(&self) -> Option<&Place> {
        match self {
            Operand::Read(p) | Operand::Move(p) => Some(p),
            Operand::Const(_) => None,
        }
    }
}

/// The language's own operations, as MIR sees them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    /// `must` in a `test fn` (owner, 2026-09-29): see `hir::Builtin`.
    MustReport,
    MustStopFork,
    MustStopDone,
    ListLength,
    ListAppend,
    /// `list.fill[value, count]`: a new list of `count` copies.
    ListFill,
    ListRemove,
    ListSlice,
    /// The same elements, as the other kind of sequence: a list's length
    /// frozen, or an array allowed to grow.
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
    StrEach(Unit),
    CharCode,
    CharFromCode(Unit),
    PtrRead,
    PtrWrite,
    PtrAt,
    PtrIsNull,
    ChannelSend,
    ChannelTake,
    /// The task that made this channel will never name it again: it stops
    /// counting as one that might send (`mir::channels`).
    ChannelRelease,
    SimdFill,
    SimdLoad,
    SimdStore,
    SimdAll,
    SimdAny,
    SimdMax,
    SimdMin,
    SimdSum,
    /// The lanes in a new order, fixed when compiling (0-based).
    SimdShuffle(Vec<u32>),
    SimdLoadSome,
    SimdStoreSome,
    /// A std name, until the std Limb exists (owner, 2026-09-22).
    Std(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Rvalue {
    Use(Operand),
    /// `copy` — a second, independent value. An allocation, unlike `Read`.
    Duplicate(Place),
    /// `lend` / `lend.mut`. The borrow's life is an explicit `EndLend`.
    Lend { place: Place, mutable: bool },
    Unary { op: UnOp, operand: Operand },
    Binary { op: BinOp, lhs: Operand, rhs: Operand },
    /// `to.T[…]` that cannot fail (the failing ones are a `Call`).
    /// `useful`: a bin into a `deci` as its shortest decimal
    /// (`to.useful.deci64[…]`), not its exact value; into text, the useful form.
    Convert { to: Ty, operand: Operand, useful: bool },
    MakeList(Vec<Operand>),
    MakeMap(Vec<(Operand, Operand)>),
    MakeStruct { id: StructId, fields: Vec<Operand> },
    MakeEnum { id: EnumId, case: usize, payload: Option<Operand> },
    MakeChannel { elem: Ty, capacity: Option<(u64, u64)> },
    /// A named function as a value.
    FnRef(FnIdx),
    /// An anonymous function and what it took with it.
    Closure { body: FnIdx, captures: Vec<(crate::ast::Verb, Place)> },
    /// C's variable.
    ExternRead(ExternId),
}

/// Why a check site exists. Every one of these is a place the final build
/// refuses unless Precog proves it (`core-features.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SiteKind {
    /// A plain `+ - x xx` that must not leave its width. `+~` is never one.
    Overflow(BinOp),
    DivZero,
    ModZero,
    /// `'xs'[i]` — 1-based, must be within the length.
    Bounds,
    /// Into a narrower type, or into a range type by assignment.
    Narrow { to: Ty },
    /// A bounded list or map, after every change.
    LengthBound { to: Ty },
    /// `shl` past the width.
    ShiftWidth,
    /// `shr`: the count must be 0 to width - 1 (owner, 2026-09-23). A
    /// right shift cannot overflow, so the count is all there is to ask.
    ShiftCount,
    /// `xx` (and `xx~`): a negative exponent makes a fraction, which no
    /// integer holds.
    NegativeExponent,
    /// `list.fill`'s count: a count is 0 or more (owner, 2026-09-30).
    FillCount,
    /// `list.slice[…, a, b]`: `1 <= a <= b + 1 <= length + 1` —
    /// inside the list, and not backwards; `pos n, n - 1` is empty.
    SliceRange,
    /// `simd.load` / `simd.store`: is `i + N - 1` still inside?
    SimdRange { lanes: u32 },
    /// `simd.load.some` / `simd.store.some`: k is at most the lanes, and
    /// the k items from the position are in the list.
    SimdSome { lanes: u32 },
    /// Equal keys must hash the same, for a type with its own `hash`.
    KeyHashAgrees,
    /// A lend that must not outlive its owner.
    LendOutlives,
    /// A `concur` capture lent into a task that must end first.
    TaskLendOutlives,
    /// A `send` nobody can ever take; two tasks waiting on each other.
    ChannelReachable,
    Deadlock,
}

impl SiteKind {
    /// What the message calls it, in the writer's words (`diagnostics.md`
    /// rule 8: say what would happen, no borrowed furniture).
    pub fn sentence(&self, spell: &dyn Fn(&Ty) -> String) -> String {
        match self {
            SiteKind::Overflow(op) => format!("`{}` can leave what the type holds", op.spelled()),
            SiteKind::DivZero => "the divisor can be zero, and nothing can be divided by zero".into(),
            SiteKind::ModZero => "the divisor can be zero, and there is no remainder by zero".into(),
            SiteKind::Bounds => "the position can be past the end".into(),
            SiteKind::Narrow { to } if *to == Ty::Duration => "a duration is a length of time, and this can take it below zero".to_string(),
            SiteKind::Narrow { to } => format!("the value can be outside what a `{}` holds", spell(to)),
            SiteKind::LengthBound { to } => format!("this can take it outside the bound its type gives it (`{}`)", spell(to)),
            SiteKind::ShiftWidth => "the shift can move bits out of the type: too far, or a value too big to shift".into(),
            SiteKind::ShiftCount => "the shift count can be outside 0 to the width - 1".into(),
            SiteKind::SliceRange => "the slice can run outside the list, or backwards".into(),
            SiteKind::NegativeExponent => "a negative exponent on integers would produce a fraction".into(),
            SiteKind::FillCount => "`list.fill` of fewer than 0 copies: a count is 0 or more".into(),
            SiteKind::SimdRange { lanes } => format!("the {} lanes can run past the end", lanes),
            SiteKind::SimdSome { lanes } => format!("the items for {} lanes can run past the end, or be more than the lanes", lanes),
            SiteKind::KeyHashAgrees => "two keys this says are equal can hash differently, and the map would hold both".into(),
            SiteKind::LendOutlives => "the borrow can outlive what it borrows".into(),
            SiteKind::TaskLendOutlives => "the task can outlive what it borrowed".into(),
            SiteKind::ChannelReachable => "nothing can ever take what is sent here".into(),
            SiteKind::Deadlock => "both sides can end up waiting for the other".into(),
        }
    }
}

/// What Precog concluded. Everything starts `Unproven`; Precog is the last
/// MIR pass, so the verdict on the frozen MIR is what the build emits.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    /// Not looked at yet.
    Unproven,
    /// No value reaching here can fail: emit the plain instruction, and
    /// hand the fact to LLVM (`nsw`/`nuw`/`!range`).
    Proven { how: &'static str },
    /// A counterexample: the working build keeps a check, the final build
    /// refuses, `check` refuses in both.
    Fails { counterexample: String },
    /// `nocheck`: removed on the writer's word, and listed.
    Trusted,
}

#[derive(Debug, Clone)]
pub struct Site {
    pub kind: SiteKind,
    /// `None` is the default tier: "I have no guardrails yet, help me".
    pub tier: Option<Tier>,
    /// The type the value has to fit — the *result's*, which is not the
    /// same as either operand's: `'total' x 'n'` with `'total'` an
    /// `int32[0, 1000]` is computed as an `int32`, and the store into the
    /// narrower type is its own site.
    pub result_ty: Option<Ty>,
    pub span: SpanId,
    /// The values the check is about, for the counterexample.
    pub operands: Vec<Operand>,
    pub verdict: Verdict,
}

impl Site {
    /// What the message calls it. Plain math on integers is exact (owner,
    /// 2026-09-23), so its one question in between is the 64-bit window —
    /// the oracle's words, "goes past 64 bits in between".
    pub fn sentence(&self, spell: &dyn Fn(&Ty) -> String) -> String {
        match (&self.kind, &self.result_ty) {
            (SiteKind::Overflow(op), Some(t)) if t.is_exact() => format!("`{}` goes past 64 bits in between", op.spelled()),
            // A bin never holds an infinity or a NaN (owner, 2026-09-29).
            (SiteKind::Overflow(BinOp::Pow), Some(t)) if matches!(t.base(), Ty::Bin(_)) => "`xx` can go past the largest bin (it would be infinity) or have no number (it would be NaN)".into(),
            (SiteKind::Overflow(op), Some(t)) if matches!(t.base(), Ty::Bin(_)) => format!("`{}` can go past the largest bin (it would be infinity)", op.spelled()),
            _ => self.kind.sentence(spell),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Assign { place: Place, rvalue: Rvalue },
    /// A language operation with no failure edge.
    Op { dest: Option<Place>, op: Op, args: Vec<Operand> },
    /// A check site, immediately before the instruction it guards.
    Site(SiteId),
    /// The scope that owns this value has ended: free it.
    Drop(Place),
    /// A borrow ends here. Between `Lend` and this, the owner cannot move
    /// or (for a mutable lend) read it — what the borrow checker walks.
    EndLend(Place),
    /// A task: its body is its own MIR function, with what it captured.
    /// `concur`: start `body` with these captures. A structured task joins
    /// `group` — one per block that starts tasks — and `id` names it for
    /// its outcome check; a detached one belongs to no group.
    Task { detached: bool, body: FnIdx, captures: Vec<(crate::ast::Verb, Place)>, group: u32, id: u32 },
    /// `lends`: the address of each lent place, into its register; `true`
    /// when the assembly may write through it.
    Asm { text: String, ins: Vec<(Place, String)>, outs: Vec<(Place, String)>, lends: Vec<(Place, bool, String)>, clobbers: Vec<String> },
    Nop,
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub kind: StmtKind,
    pub span: SpanId,
}

/// One arm of a call's failure edge.
#[derive(Debug, Clone)]
pub struct FailArm {
    /// Which reason of the callee's list.
    pub index: usize,
    /// Where the payload lands, when the reason carries one.
    pub payload: Option<Place>,
    pub block: BlockId,
}

#[derive(Debug, Clone)]
pub enum Callee {
    Fn(FnIdx),
    Extern(ExternId),
    /// A held function value.
    Value(Operand),
    /// A language operation that can fail (`map.get`, `channel.take`, a
    /// failing `to.…`).
    Op(Op),
    /// `to.T[…]` that can fail.
    Convert(Ty),
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Goto(BlockId),
    If { cond: Operand, then: BlockId, else_: BlockId },
    /// One arm per enum case; `match` is exhaustive, so there is no default.
    Switch { value: Operand, cases: Vec<(usize, BlockId)> },
    /// Every call is a terminator: a failure is an edge, not a value to test.
    Call { callee: Callee, args: Vec<Arg>, dest: Option<Place>, ok: BlockId, fail: Vec<FailArm> },
    /// `whichever`: whichever channel has a value first.
    Whichever { arms: Vec<WhicheverArm>, closed: Option<BlockId> },
    /// The end of a block that started tasks: it waits for them here, and a
    /// task's failure is an edge out of the wait (`concurrency.md`: the
    /// handler runs when the enclosing block ends).
    Join { group: u32, ok: BlockId },
    /// After the wait: did task `id` (whose body is `body`) fail, and how?
    /// One check per task that can fail, chained, because two tasks in one
    /// block can both fail and each has its own handler (`concurrency.md`:
    /// the handler runs when the enclosing block ends). `ok` is the next
    /// check, or the rest of the block.
    TaskOutcome { id: u32, body: FnIdx, ok: BlockId, fail: Vec<FailArm> },
    Return,
    Fail { index: usize, payload: Option<Operand> },
    /// A check that fired. The working build traps here with the site
    /// named; the final build never emits one, because it refuses first.
    Trap(SiteId),
    Unreachable,
}

#[derive(Debug, Clone)]
pub struct WhicheverArm {
    pub channel: Operand,
    pub dest: Place,
    pub block: BlockId,
}

/// How one argument reaches a callee: the verb is part of the call.
#[derive(Debug, Clone)]
pub enum Arg {
    Value(Operand),
    Lend { place: Place, mutable: bool },
    Give(Place),
    Copy(Place),
    /// `...` — `count` elements of one list, unpacked here.
    Spread { place: Place, count: usize, verb: Option<crate::ast::Verb> },
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub stmts: Vec<Statement>,
    pub term: Terminator,
    /// Where the block leaves from: a call's own place in the source, so a
    /// message about an argument points at the call and not at whatever
    /// happened to be first in the span table.
    pub term_span: SpanId,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    /// The instance this is: one source function, one shape. A task body
    /// is nobody's instance.
    pub inst: Option<FnInst>,
    pub params: Vec<LocalId>,
    pub locals: Vec<LocalDecl>,
    pub blocks: Vec<BasicBlock>,
    pub ret: Ty,
    /// Where a `return` puts its value. `None` when the function returns
    /// nothing.
    pub ret_local: Option<LocalId>,
    /// A borrow handed back (owner, 2026-09-29): whether `lent.mut`, and
    /// the parameters, by position, the caller's result borrows.
    pub ret_lent: Option<(bool, Vec<usize>)>,
    pub fails: Vec<(String, Option<Ty>)>,
    pub sites: Vec<Site>,
    /// Spans, indexed by `SpanId`. Not part of the canonical form.
    pub spans: Vec<Span>,
    pub is_main: bool,
    /// A `test fn`: `cca test` runs it; a normal build leaves it out.
    pub is_test: bool,
    /// This function is a task's body: it was written inside a `concur`.
    pub is_task: bool,
    /// Its last parameter is a variadic bundle: a call hands the values one
    /// by one, and the call builds the bundle (`functions.md`).
    pub variadic: bool,
    /// A task body's captures, in the order its parent passes them.
    pub captures: Vec<LocalId>,
    /// For an exact instruction, keyed by (block, statement): every value
    /// its operands and its answer take on every run, from Precog. Plain
    /// math is exact, and "Precog still uses narrow sizes where it proves
    /// they fit" (`operators.md`): inside 64 bits, codegen works at 64.
    pub ranges: std::collections::HashMap<(u32, u32), (i128, i128)>,
}

impl Function {
    pub fn span(&self, id: SpanId) -> Span {
        self.spans[id.0 as usize]
    }
    pub fn block(&self, id: BlockId) -> &BasicBlock {
        &self.blocks[id.0 as usize]
    }
    pub fn local(&self, id: LocalId) -> &LocalDecl {
        &self.locals[id.0 as usize]
    }
    pub fn site(&self, id: SiteId) -> &Site {
        &self.sites[id.0 as usize]
    }
    /// The writer's name for a place, for a message.
    pub fn place_name(&self, p: &Place, structs: &[crate::types::StructInfo]) -> String {
        let mut s = match &self.local(p.local).name {
            Some(n) => format!("'{}'", n),
            None => "a value".to_string(),
        };
        let mut ty = self.local(p.local).ty.clone();
        for pr in &p.proj {
            match pr {
                Proj::Field(i) => {
                    if let Ty::Struct { id, args } = ty.base().clone() {
                        let info = &structs[id.0 as usize];
                        s.push('.');
                        s.push_str(&info.fields[*i].name);
                        ty = info.fields[*i].ty.subst(&args);
                    }
                }
                Proj::Index(_) => {
                    s.push_str("[…]");
                    if let Ty::Seq { elem, .. } = ty.base().clone() {
                        ty = (*elem).clone();
                    }
                }
                Proj::BundleElem(i) => s.push_str(&format!("[{}]", i + 1)),
                Proj::Deref => s = format!("what {} points at", s),
            }
        }
        s
    }
}

/// The whole program in MIR.
pub struct Program {
    pub functions: Vec<Function>,
    /// The entry, when there is one.
    pub main: Option<FnIdx>,
    pub items: crate::resolve::Items,
    /// The instances the checker made, for their signatures.
    pub instances: Vec<crate::check::Instance>,
}

impl Program {
    pub fn f(&self, i: FnIdx) -> &Function {
        &self.functions[i.0 as usize]
    }
}
