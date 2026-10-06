//! Types as the checker sees them. A type here is a *family with slots*:
//! a width, an encoding, a unit, a list-or-array choice may be **free**
//! — Precog fills it from every value the binding ever holds
//! (`core-features.md`: `:=` computes the best type). A free integer is
//! exact: arithmetic on it never overflows, and the store into a typed
//! slot is the check site. A declared width is a machine width, and each
//! operation on it is an overflow check site.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InferId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StructId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IfaceId(pub u32);

/// An integer's width: declared, or free for Precog to choose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntW {
    Fixed { signed: bool, bits: u32 },
    Free(InferId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinW {
    Fixed(u16),
    Free(InferId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Enc {
    Utf8,
    Utf16,
    Utf32,
    Ascii,
    Free(InferId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Unit {
    Byte,
    Point,
    Grapheme,
    Free(InferId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeqKind {
    List,
    Array,
    /// Never grown → array; Precog decides.
    Free(InferId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapOrder {
    Ordered,
    Unordered,
    /// Precog picks: unordered when never walked.
    Free(InferId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParamKind {
    Owned,
    Lent,
    LentMut,
}

impl ParamKind {
    pub fn spelled(self) -> &'static str {
        match self {
            ParamKind::Owned => "",
            ParamKind::Lent => "lent ",
            ParamKind::LentMut => "lent.mut ",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnTy {
    pub params: Vec<(ParamKind, Ty)>,
    pub ret: Ty,
    pub fails: Vec<(String, Option<Ty>)>,
    /// It carries what it captured by `copy` or `give`, so it is owned
    /// like a list (owner, 2026-09-24: a). Not part of what a `fn[…]` type
    /// says: a value's, known where the literal is written.
    pub owning: bool,
}

/// A bound written into a type: `int32[0, 100]`, `list[T][0, 10]`,
/// `str.utf8[1, 40 graphemes]`, `map[K, V][0, 2]`, `channel[T][0, N]`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Bound {
    /// A numeric range, both ends included, as exact integers.
    Int { lo: i128, hi: i128 },
    /// A bin range, as the literal's text (compared exactly by Precog).
    Bin { lo: String, hi: String },
    /// A length or count bound; `unit` only for text.
    Len { lo: u64, hi: u64, unit: Option<Unit> },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ty {
    Int(IntW),
    Bin(BinW),
    /// `brain16` (also `bfloat16`, `bf16`; owner, 2026-10-02): the brain
    /// float, `bin32`'s range in 16 bits, a family of its own.
    Brain,
    Deci(u16),
    Bool,
    Str(Enc),
    Char(Unit),
    Seq { elem: Box<Ty>, kind: SeqKind },
    Map { key: Box<Ty>, val: Box<Ty>, order: MapOrder },
    Channel(Box<Ty>),
    /// `simd[T, N]` — N plain values in one register (`simd.md`). Legal
    /// only in a shape the target actually has.
    Simd { elem: Box<Ty>, lanes: u32 },
    Struct { id: StructId, args: Vec<Ty> },
    Enum { id: EnumId, args: Vec<Ty> },
    Fn(Box<FnTy>),
    Ptr { mutable: bool, elem: Box<Ty> },
    Duration,
    /// A heterogeneous variadic bundle: one type per argument.
    Bundle(Vec<Ty>),
    /// The type of a function with no return value.
    Nothing,
    /// A type not known yet: unified as the program is checked.
    Infer(InferId),
    /// A generic blank, in the abstract; instantiations substitute.
    Var(u32),
    Bounded { inner: Box<Ty>, bound: Bound },
}

/// Marks [`Ty::exact`]: no width anyone writes has this bit, and the rest
/// is the limit in between (owner, 2026-10-03: the widest operand's width,
/// at least 64 bits).
pub const EXACT_FLAG: u32 = 1 << 31;
/// The exact type of 64-bit operands, the common one.
pub const EXACT_BITS: u32 = EXACT_FLAG | 64;
/// The widest integer there is: LLVM's 2^23 bits (owner, 2026-10-03).
pub const WIDEST_INT: u32 = 1 << 23;

impl Ty {
    pub fn int(signed: bool, bits: u32) -> Ty {
        Ty::Int(IntW::Fixed { signed, bits })
    }
    pub fn u64() -> Ty {
        Ty::int(false, 64)
    }
    /// The true answer of plain integer math (`+ - x xx`, owner 2026-09-23,
    /// `operators.md`): exact, checked where it lands, and in between held
    /// to what some 64-bit type holds — `int64`'s lowest to `uint64`'s
    /// highest. Not a type anyone writes: 65 bits carry that range, and
    /// `resolve::int_range` gives it exactly.
    pub fn exact() -> Ty {
        Ty::int(true, EXACT_BITS)
    }
    /// The exact answer whose in-between limit is `limit` bits: what some
    /// `limit`-bit type holds, the signed one's lowest to the unsigned
    /// one's highest. Never below 64.
    pub fn exact_at(limit: u32) -> Ty {
        Ty::int(true, EXACT_FLAG | limit.max(64))
    }
    pub fn is_exact(&self) -> bool {
        self.exact_limit().is_some()
    }
    /// The limit in between of an exact answer.
    pub fn exact_limit(&self) -> Option<u32> {
        match self.base() {
            Ty::Int(IntW::Fixed { signed: true, bits }) if bits & EXACT_FLAG != 0 => Some(bits & !EXACT_FLAG),
            _ => None,
        }
    }
    pub fn list(elem: Ty) -> Ty {
        Ty::Seq { elem: Box::new(elem), kind: SeqKind::List }
    }
    pub fn utf8() -> Ty {
        Ty::Str(Enc::Utf8)
    }

    /// The type under any bound.
    pub fn base(&self) -> &Ty {
        match self {
            Ty::Bounded { inner, .. } => inner.base(),
            t => t,
        }
    }
    pub fn bound(&self) -> Option<&Bound> {
        match self {
            Ty::Bounded { bound, .. } => Some(bound),
            _ => None,
        }
    }
    pub fn is_int(&self) -> bool {
        matches!(self.base(), Ty::Int(_))
    }
    pub fn is_bin(&self) -> bool {
        matches!(self.base(), Ty::Bin(_))
    }
    pub fn is_numeric(&self) -> bool {
        matches!(self.base(), Ty::Int(_) | Ty::Bin(_) | Ty::Deci(_) | Ty::Brain)
    }
    pub fn is_brain(&self) -> bool {
        matches!(self.base(), Ty::Brain)
    }
    pub fn is_text(&self) -> bool {
        matches!(self.base(), Ty::Str(_))
    }
    pub fn is_char(&self) -> bool {
        matches!(self.base(), Ty::Char(_))
    }
    pub fn is_seq(&self) -> bool {
        matches!(self.base(), Ty::Seq { .. })
    }
    pub fn is_simd(&self) -> bool {
        matches!(self.base(), Ty::Simd { .. })
    }
    /// A vector's lane type, when it is one.
    pub fn lane(&self) -> Option<&Ty> {
        match self.base() {
            Ty::Simd { elem, .. } => Some(elem),
            _ => None,
        }
    }
    pub fn is_infer(&self) -> bool {
        matches!(self, Ty::Infer(_))
    }

    /// Owned values need the verbs; plain values copy by themselves.
    /// A struct or enum is always owned (decided 2026-09-18, 2026-09-20).
    pub fn is_owned(&self) -> bool {
        match self.base() {
            Ty::Str(_) | Ty::Seq { .. } | Ty::Map { .. } | Ty::Struct { .. } | Ty::Enum { .. } | Ty::Channel(_) | Ty::Bundle(_) => true,
            Ty::Char(Unit::Grapheme) => true,
            Ty::Fn(f) => f.owning,
            Ty::Var(_) => true,
            _ => false,
        }
    }

    /// What C hands back that Cyborg must check before holding it: a range,
    /// a C enum's number, which may match no case, and a `layout.C` struct
    /// with either among its fields (owner, 2026-09-27: a).
    pub fn checked_at_door(&self, structs: &[StructInfo], enums: &[EnumInfo]) -> bool {
        if self.bound().is_some() {
            return true;
        }
        match self.base() {
            Ty::Enum { id, .. } => enums[id.0 as usize].numbers.is_some(),
            Ty::Struct { id, .. } => {
                let s = &structs[id.0 as usize];
                s.layout_c && s.fields.iter().any(|f| f.ty.checked_at_door(structs, enums))
            }
            _ => false,
        }
    }

    /// C-shaped: may cross to C (`ffi.md`).
    pub fn is_c_shaped(&self, structs: &[StructInfo], enums: &[EnumInfo]) -> bool {
        match self {
            Ty::Int(IntW::Fixed { .. }) | Ty::Bin(BinW::Fixed(_)) | Ty::Bool | Ty::Ptr { .. } => true,
            Ty::Char(Unit::Byte) | Ty::Char(Unit::Point) => true,
            Ty::Struct { id, .. } => structs[id.0 as usize].layout_c,
            Ty::Enum { id, .. } => enums[id.0 as usize].numbers.is_some(),
            Ty::Fn(f) => f.params.iter().all(|(k, t)| *k == ParamKind::Owned && t.is_c_shaped(structs, enums)) && f.fails.is_empty() && (f.ret == Ty::Nothing || f.ret.is_c_shaped(structs, enums)),
            _ => false,
        }
    }

    pub fn contains_var(&self) -> bool {
        match self {
            Ty::Var(_) => true,
            Ty::Seq { elem, .. } | Ty::Channel(elem) | Ty::Ptr { elem, .. } | Ty::Simd { elem, .. } => elem.contains_var(),
            Ty::Map { key, val, .. } => key.contains_var() || val.contains_var(),
            Ty::Struct { args, .. } | Ty::Enum { args, .. } => args.iter().any(|a| a.contains_var()),
            Ty::Fn(f) => f.params.iter().any(|(_, t)| t.contains_var()) || f.ret.contains_var() || f.fails.iter().any(|(_, t)| t.as_ref().map(|t| t.contains_var()).unwrap_or(false)),
            Ty::Bundle(ts) => ts.iter().any(|t| t.contains_var()),
            Ty::Bounded { inner, .. } => inner.contains_var(),
            _ => false,
        }
    }

    /// Replace `Var(i)` by `args[i]`.
    pub fn subst(&self, args: &[Ty]) -> Ty {
        match self {
            Ty::Var(i) => args.get(*i as usize).cloned().unwrap_or(Ty::Var(*i)),
            Ty::Seq { elem, kind } => Ty::Seq { elem: Box::new(elem.subst(args)), kind: *kind },
            Ty::Channel(e) => Ty::Channel(Box::new(e.subst(args))),
            Ty::Simd { elem, lanes } => Ty::Simd { elem: Box::new(elem.subst(args)), lanes: *lanes },
            Ty::Ptr { mutable, elem } => Ty::Ptr { mutable: *mutable, elem: Box::new(elem.subst(args)) },
            Ty::Map { key, val, order } => Ty::Map { key: Box::new(key.subst(args)), val: Box::new(val.subst(args)), order: *order },
            Ty::Struct { id, args: a } => Ty::Struct { id: *id, args: a.iter().map(|t| t.subst(args)).collect() },
            Ty::Enum { id, args: a } => Ty::Enum { id: *id, args: a.iter().map(|t| t.subst(args)).collect() },
            Ty::Fn(f) => Ty::Fn(Box::new(FnTy {
                params: f.params.iter().map(|(k, t)| (*k, t.subst(args))).collect(),
                ret: f.ret.subst(args),
                fails: f.fails.iter().map(|(n, t)| (n.clone(), t.as_ref().map(|t| t.subst(args)))).collect(),
                owning: f.owning,
            })),
            Ty::Bundle(ts) => Ty::Bundle(ts.iter().map(|t| t.subst(args)).collect()),
            Ty::Bounded { inner, bound } => Ty::Bounded { inner: Box::new(inner.subst(args)), bound: bound.clone() },
            t => t.clone(),
        }
    }
}

// ---------------------------------------------------------------- declared shapes

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub immut: bool,
    pub ty: Ty,
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub path: String,
    pub layout_c: bool,
    pub fields: Vec<FieldInfo>,
    /// The generic blanks, in order of first appearance.
    pub blanks: Vec<String>,
    /// What each blank must satisfy, by the same position (owner,
    /// 2026-10-02); none for `any`.
    pub bounds: Vec<Vec<IfaceId>>,
    pub span: crate::source::Span,
}

#[derive(Debug, Clone)]
pub struct CaseInfo {
    pub name: String,
    pub payload: Option<Ty>,
}

#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub path: String,
    pub cases: Vec<CaseInfo>,
    /// A `layout.C enum`: each case's number, as C sees it (owner,
    /// 2026-09-27: a). Its tag is that number, 32 bits.
    pub numbers: Option<Vec<i64>>,
    pub blanks: Vec<String>,
    /// What each blank must satisfy, by the same position (owner,
    /// 2026-10-02); none for `any`.
    pub bounds: Vec<Vec<IfaceId>>,
    pub span: crate::source::Span,
}

#[derive(Debug, Clone)]
pub struct IfaceEntryInfo {
    /// `fn show` or `op <`.
    pub is_op: bool,
    pub name: String,
    pub params: Vec<(ParamKind, Ty)>,
    pub ret: Ty,
    pub fails: Vec<(String, Option<Ty>)>,
}

/// A `brain16`'s width code where a runtime call takes a bin's width
/// (`runtime::BRAIN`).
pub const BRAIN_BITS: u16 = 0xB16;

#[derive(Debug, Clone)]
pub struct IfaceInfo {
    pub name: String,
    pub path: String,
    pub entries: Vec<IfaceEntryInfo>,
    /// The types it accepts, a closed list (owner, 2026-10-02); none for
    /// an interface that only lists operations.
    pub types: Vec<Ty>,
    pub span: crate::source::Span,
}

// ---------------------------------------------------------------- spelling

/// How a type is written back to the writer.
pub struct TySpelling<'a> {
    pub ty: &'a Ty,
    pub structs: &'a [StructInfo],
    pub enums: &'a [EnumInfo],
}

impl<'a> fmt::Display for TySpelling<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sp = |t: &'a Ty| TySpelling { ty: t, structs: self.structs, enums: self.enums };
        match self.ty {
            Ty::Int(IntW::Fixed { signed: true, bits }) if bits & EXACT_FLAG != 0 => write!(f, "integer"),
            Ty::Int(IntW::Fixed { signed, bits }) if matches!(bits, 8 | 16 | 32 | 64 | 128) => write!(f, "{}int{}", if *signed { "" } else { "u" }, bits),
            // Not a standard width: marked as such (owner, 2026-10-03).
            Ty::Int(IntW::Fixed { signed, bits }) => write!(f, "!std.{}int{}", if *signed { "" } else { "u" }, bits),
            Ty::Int(IntW::Free(_)) => write!(f, "integer"),
            Ty::Bin(BinW::Fixed(b)) => write!(f, "bin{}", b),
            Ty::Bin(BinW::Free(_)) => write!(f, "bin"),
            Ty::Brain => write!(f, "brain16"),
            Ty::Deci(b) => write!(f, "deci{}", b),
            Ty::Bool => write!(f, "bool"),
            Ty::Str(e) => match e {
                Enc::Utf8 => write!(f, "str.utf8"),
                Enc::Utf16 => write!(f, "str.utf16"),
                Enc::Utf32 => write!(f, "str.utf32"),
                Enc::Ascii => write!(f, "str.ascii"),
                Enc::Free(_) => write!(f, "str"),
            },
            Ty::Char(u) => match u {
                Unit::Byte => write!(f, "char.byte"),
                Unit::Point => write!(f, "char.point"),
                Unit::Grapheme => write!(f, "char.grapheme"),
                Unit::Free(_) => write!(f, "char"),
            },
            Ty::Seq { elem, kind } => {
                let k = match kind {
                    SeqKind::List => "list",
                    SeqKind::Array => "array",
                    SeqKind::Free(_) => "list",
                };
                write!(f, "{}[{}]", k, sp(elem))
            }
            Ty::Map { key, val, order } => {
                let o = match order {
                    MapOrder::Ordered => "map.ordered",
                    MapOrder::Unordered => "map.unordered",
                    MapOrder::Free(_) => "map",
                };
                write!(f, "{}[{}, {}]", o, sp(key), sp(val))
            }
            Ty::Channel(e) => write!(f, "channel[{}]", sp(e)),
            Ty::Simd { elem, lanes } => write!(f, "simd[{}, {}]", sp(elem), lanes),
            Ty::Struct { id, args } => {
                write!(f, "{}", self.structs[id.0 as usize].name)?;
                if !args.is_empty() {
                    write!(f, "[{}]", args.iter().map(|a| sp(a).to_string()).collect::<Vec<_>>().join(", "))?;
                }
                Ok(())
            }
            Ty::Enum { id, args } => {
                write!(f, "{}", self.enums[id.0 as usize].name)?;
                if !args.is_empty() {
                    write!(f, "[{}]", args.iter().map(|a| sp(a).to_string()).collect::<Vec<_>>().join(", "))?;
                }
                Ok(())
            }
            Ty::Fn(ft) => {
                write!(f, "fn[{}]", ft.params.iter().map(|(k, t)| format!("{}{}", k.spelled(), sp(t))).collect::<Vec<_>>().join(", "))?;
                if ft.ret != Ty::Nothing {
                    write!(f, " -> {}", sp(&ft.ret))?;
                }
                if !ft.fails.is_empty() {
                    write!(f, "{} fail[{}]", if ft.ret != Ty::Nothing { " or" } else { " ->" }, ft.fails.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>().join(", "))?;
                }
                Ok(())
            }
            Ty::Ptr { mutable, elem } => write!(f, "ptr{}[{}]", if *mutable { ".mut" } else { "" }, sp(elem)),
            Ty::Duration => write!(f, "duration"),
            Ty::Bundle(ts) => write!(f, "a bundle of {} value(s)", ts.len()),
            Ty::Nothing => write!(f, "nothing"),
            Ty::Infer(_) => write!(f, "(not yet known)"),
            Ty::Var(i) => write!(f, "<blank {}>", i),
            Ty::Bounded { inner, bound } => {
                write!(f, "{}", sp(inner))?;
                match bound {
                    Bound::Int { lo, hi } => write!(f, "[{}, {}]", lo, hi),
                    Bound::Bin { lo, hi } => write!(f, "[{}, {}]", lo, hi),
                    Bound::Len { lo, hi, unit } => {
                        let u = match unit {
                            Some(Unit::Byte) => " bytes",
                            Some(Unit::Point) => " points",
                            Some(Unit::Grapheme) => " graphemes",
                            _ => "",
                        };
                        write!(f, "[{}, {}{}]", lo, hi, u)
                    }
                }
            }
        }
    }
}

/// The register shapes a target has (`simd.md`, owner: b — only the
/// target's own). ARM64's 128-bit NEON, which is what the oracle fixed
/// and what this Mac is; a real target description comes from
/// the manifest (`CCA.cplr`, `tooling.md`) and never from the host.
pub const TARGET_SIMD_BITS: u32 = 128;

/// Is `simd[elem, lanes]` a register this target has?
pub fn simd_shape_ok(elem: &Ty, lanes: u32) -> bool {
    match simd_lane_bits(elem) {
        Some(bits) => bits * lanes == TARGET_SIMD_BITS,
        None => false,
    }
}

/// How wide one lane is, for a type that may be one.
pub fn simd_lane_bits(elem: &Ty) -> Option<u32> {
    match elem.base() {
        Ty::Int(IntW::Fixed { bits, .. }) => Some(*bits as u32),
        Ty::Bin(BinW::Fixed(b)) => Some(*b as u32),
        // A lane of answers: one bit each, as wide as the vector it came from.
        Ty::Bool => None,
        _ => None,
    }
}

/// The lanes a vector has once nothing more can give its lane a width: a
/// lane still free (`let 'a' := simd.fill[5]`) is the natural width, as a lone
/// number's is (`core-features.md`) and as codegen builds it — 64 bits. It
/// had no shape at all: 0 lanes. While checking, a free lane stays open
/// for its place to settle (`simd_shapes_for`).
pub fn simd_final_lanes(elem: &Ty) -> u32 {
    match elem.base() {
        Ty::Int(IntW::Free(_)) | Ty::Bin(BinW::Free(_)) => TARGET_SIMD_BITS / 64,
        _ => simd_shapes_for(elem).unwrap_or(0),
    }
}

/// Every shape this target has for a lane type, for the message that says so.
pub fn simd_shapes_for(elem: &Ty) -> Option<u32> {
    simd_lane_bits(elem).map(|bits| TARGET_SIMD_BITS / bits)
}
