//! Domains: what a value can be. Layer 2 of `design/precog.md` — exact
//! sets while small, intervals past a cap. Every MIR temporary carries
//! one from the day it is created: a type's own domain to begin with,
//! narrowed by Precog later. A domain is never allowed to be *smaller*
//! than the truth; being wider only costs a check.

use crate::types::{Bound, BinW, Enc, IntW, SeqKind, Ty, Unit};

/// Provisional (Kespar's pick, `precog.md` Open): a set of more than this
/// many values becomes an interval.
pub const SET_CAP: usize = 4096;
/// Provisional: an interval of at most this many values is spelled out again.
pub const SET_REOPEN: u64 = 64;

/// A set of integers: exact while small, an interval when not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ISet {
    /// Sorted, deduplicated, never empty.
    Exact(Vec<i128>),
    /// Both ends included. `lo <= hi`.
    Range { lo: i128, hi: i128 },
    /// No value: unreachable code, or a certain failure.
    Empty,
}

impl ISet {
    pub fn one(v: i128) -> ISet {
        ISet::Exact(vec![v])
    }
    pub fn range(lo: i128, hi: i128) -> ISet {
        if lo > hi {
            ISet::Empty
        } else if hi.checked_sub(lo).map(|d| d < SET_REOPEN as i128).unwrap_or(false) {
            ISet::Exact((lo..=hi).collect())
        } else {
            ISet::Range { lo, hi }
        }
    }
    pub fn is_empty(&self) -> bool {
        matches!(self, ISet::Empty)
    }
    pub fn bounds(&self) -> Option<(i128, i128)> {
        match self {
            ISet::Exact(v) => Some((*v.first()?, *v.last()?)),
            ISet::Range { lo, hi } => Some((*lo, *hi)),
            ISet::Empty => None,
        }
    }
    pub fn count(&self) -> Option<u64> {
        match self {
            ISet::Exact(v) => Some(v.len() as u64),
            ISet::Range { lo, hi } => hi.checked_sub(*lo).and_then(|d| d.checked_add(1)).and_then(|n| u64::try_from(n).ok()),
            ISet::Empty => Some(0),
        }
    }
    /// Every value in the set is inside `[lo, hi]`.
    pub fn within(&self, lo: i128, hi: i128) -> bool {
        match self.bounds() {
            Some((a, b)) => a >= lo && b <= hi,
            None => true,
        }
    }
    /// No value in the set is inside `[lo, hi]`.
    pub fn misses(&self, lo: i128, hi: i128) -> bool {
        match self {
            ISet::Empty => true,
            ISet::Exact(v) => v.iter().all(|x| *x < lo || *x > hi),
            ISet::Range { lo: a, hi: b } => *b < lo || *a > hi,
        }
    }
    pub fn contains(&self, v: i128) -> bool {
        match self {
            ISet::Empty => false,
            ISet::Exact(x) => x.binary_search(&v).is_ok(),
            ISet::Range { lo, hi } => v >= *lo && v <= *hi,
        }
    }
    /// The one value, when there is only one.
    pub fn single(&self) -> Option<i128> {
        match self {
            ISet::Exact(v) if v.len() == 1 => Some(v[0]),
            ISet::Range { lo, hi } if lo == hi => Some(*lo),
            _ => None,
        }
    }
    pub fn union(&self, other: &ISet) -> ISet {
        match (self, other) {
            (ISet::Empty, x) | (x, ISet::Empty) => x.clone(),
            (ISet::Exact(a), ISet::Exact(b)) if a == b => ISet::Exact(a.clone()),
            (ISet::Exact(a), ISet::Exact(b)) if a.len() + b.len() <= SET_CAP => {
                // Two sorted lists merged in one pass; anything else sorted.
                // Precog joins every block's facts on every visit, and
                // sorting each join was most of a 144 s build (check
                // claude, seed 6798).
                if a.windows(2).all(|w| w[0] < w[1]) && b.windows(2).all(|w| w[0] < w[1]) {
                    let mut v = Vec::with_capacity(a.len() + b.len());
                    let (mut i, mut j) = (0, 0);
                    while i < a.len() && j < b.len() {
                        match a[i].cmp(&b[j]) {
                            std::cmp::Ordering::Less => {
                                v.push(a[i]);
                                i += 1;
                            }
                            std::cmp::Ordering::Greater => {
                                v.push(b[j]);
                                j += 1;
                            }
                            std::cmp::Ordering::Equal => {
                                v.push(a[i]);
                                i += 1;
                                j += 1;
                            }
                        }
                    }
                    v.extend_from_slice(&a[i..]);
                    v.extend_from_slice(&b[j..]);
                    return ISet::Exact(v);
                }
                let mut v = a.clone();
                v.extend_from_slice(b);
                v.sort_unstable();
                v.dedup();
                ISet::Exact(v)
            }
            _ => {
                let (a1, b1) = self.bounds().unwrap();
                let (a2, b2) = other.bounds().unwrap();
                ISet::Range { lo: a1.min(a2), hi: b1.max(b2) }
            }
        }
    }
    /// Values that are in both. Sound: never wider than the truth.
    pub fn meet(&self, other: &ISet) -> ISet {
        match (self, other) {
            (ISet::Empty, _) | (_, ISet::Empty) => ISet::Empty,
            (ISet::Exact(a), b) => {
                let v: Vec<i128> = a.iter().copied().filter(|x| b.contains(*x)).collect();
                if v.is_empty() {
                    ISet::Empty
                } else {
                    ISet::Exact(v)
                }
            }
            (a, ISet::Exact(b)) => {
                let v: Vec<i128> = b.iter().copied().filter(|x| a.contains(*x)).collect();
                if v.is_empty() {
                    ISet::Empty
                } else {
                    ISet::Exact(v)
                }
            }
            (ISet::Range { lo: a1, hi: b1 }, ISet::Range { lo: a2, hi: b2 }) => ISet::range((*a1).max(*a2), (*b1).min(*b2)),
        }
    }
    /// Keep only what satisfies `lo <= x <= hi`.
    pub fn clamp(&self, lo: i128, hi: i128) -> ISet {
        self.meet(&ISet::range(lo, hi))
    }
    /// The set as an exact list, if it is small enough to walk.
    pub fn values(&self) -> Option<Vec<i128>> {
        match self {
            ISet::Exact(v) => Some(v.clone()),
            ISet::Range { lo, hi } if hi.checked_sub(*lo).map(|d| d < SET_CAP as i128).unwrap_or(false) => Some((*lo..=*hi).collect()),
            ISet::Range { .. } => None,
            ISet::Empty => Some(Vec::new()),
        }
    }
}

/// A length, per unit where text has three (`strings.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lengths {
    pub bytes: ISet,
    pub points: ISet,
    pub graphemes: ISet,
}

impl Lengths {
    pub fn any() -> Lengths {
        Lengths { bytes: ISet::Range { lo: 0, hi: i64::MAX as i128 }, points: ISet::Range { lo: 0, hi: i64::MAX as i128 }, graphemes: ISet::Range { lo: 0, hi: i64::MAX as i128 } }
    }
    pub fn of(text: &str) -> Lengths {
        Lengths {
            bytes: ISet::one(text.len() as i128),
            points: ISet::one(text.chars().count() as i128),
            graphemes: ISet::one(grapheme_count(text) as i128),
        }
    }
    pub fn get(&self, u: Unit) -> &ISet {
        match u {
            Unit::Byte => &self.bytes,
            Unit::Point => &self.points,
            _ => &self.graphemes,
        }
    }
    pub fn union(&self, o: &Lengths) -> Lengths {
        Lengths { bytes: self.bytes.union(&o.bytes), points: self.points.union(&o.points), graphemes: self.graphemes.union(&o.graphemes) }
    }
}

/// A rough grapheme count. The real runtime needs UAX #29; this is enough
/// to fold a literal's length and is never used to *widen* a domain.
pub fn grapheme_count(s: &str) -> usize {
    let mut n = 0usize;
    let mut prev_joiner = false;
    for c in s.chars() {
        let combining = matches!(c as u32, 0x0300..=0x036F | 0x0E31 | 0x0E34..=0x0E3A | 0x0E47..=0x0E4E | 0xFE00..=0xFE0F | 0x1F3FB..=0x1F3FF);
        if c == '\u{200D}' {
            prev_joiner = true;
            continue;
        }
        if prev_joiner || combining {
            prev_joiner = false;
            continue;
        }
        n += 1;
    }
    n.max(if s.is_empty() { 0 } else { 1 })
}

/// A fact about the characters of a text (`precog.md`, owner: c). The
/// vocabulary is open; these are the ones a rule can be written for today.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class {
    AsciiDigits,
    Ascii,
    NoNewline,
    NonEmpty,
}

/// See `Domain::key`: every value, exactly, never a summary.
fn iset_key(s: &ISet, out: &mut String) {
    use core::fmt::Write as _;
    match s {
        ISet::Empty => out.push_str("[]"),
        ISet::Range { lo, hi } => {
            let _ = write!(out, "[{}..{}]", lo, hi);
        }
        ISet::Exact(v) => {
            out.push('[');
            for x in v {
                let _ = write!(out, "{},", x);
            }
            out.push(']');
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Domain {
    /// Nothing is known beyond the type (or the type says everything).
    Any,
    Ints(ISet),
    /// Whole numbers past what `i128` holds (owner, 2026-10-03: integers
    /// of any width): an interval of exact values. A domain whose ends
    /// both fit `i128` is `Ints` instead ([`Domain::wide`]).
    Wide { lo: crate::bigint::Big, hi: crate::bigint::Big },
    /// Bins are intervals from the start (`precog.md`); `None` is "any bin".
    Bins { lo: Option<f64>, hi: Option<f64> },
    Bools(bool, bool),
    Text { len: Lengths, values: Option<Vec<String>>, classes: Vec<Class> },
    Chars { values: Option<Vec<String>> },
    Seq { len: ISet, elem: Box<Domain> },
    Map { len: ISet, key: Box<Domain>, val: Box<Domain> },
    Struct(Vec<Domain>),
    /// Which cases are possible, and each payload's domain.
    Enum(Vec<(usize, Domain)>),
    Duration(ISet),
    /// A value that cannot exist: unreachable, or a certain failure.
    Empty,
}

impl Domain {
    pub fn is_empty(&self) -> bool {
        match self {
            Domain::Empty => true,
            Domain::Ints(s) => s.is_empty(),
            _ => false,
        }
    }

    /// The domain a type allows, before anything is proven. This is where
    /// layer 2 starts, and it is already a real fact: a `u8` is 0..255,
    /// an `i32[0, 100]` is 0..100, a `list[T][0, 5]` has 0..5 elements.
    pub fn of_ty(ty: &Ty, items: &crate::resolve::Items) -> Domain {
        match ty {
            // Nothing about a decimal is followed yet, its bounds
            // included: they are decimals, and a bin interval would round
            // them.
            Ty::Bounded { inner, .. } if matches!(inner.base(), Ty::Deci(_)) => Domain::Any,
            Ty::Bounded { inner, bound } => {
                let inner_domain = Domain::of_ty(inner, items);
                match (bound, inner_domain) {
                    (Bound::Int { lo, hi, .. }, Domain::Ints(s)) => Domain::Ints(s.clamp(*lo, *hi)),
                    (Bound::Bin { lo, hi, .. }, _) => Domain::Bins { lo: lo.parse().ok(), hi: hi.parse().ok() },
                    (Bound::Len { lo, hi, unit }, Domain::Text { mut len, values, classes }) => {
                        let s = ISet::range(*lo as i128, *hi as i128);
                        match unit {
                            Some(Unit::Byte) => len.bytes = len.bytes.meet(&s),
                            Some(Unit::Point) => len.points = len.points.meet(&s),
                            _ => len.graphemes = len.graphemes.meet(&s),
                        }
                        Domain::Text { len, values, classes }
                    }
                    (Bound::Len { lo, hi, .. }, Domain::Seq { elem, .. }) => Domain::Seq { len: ISet::range(*lo as i128, *hi as i128), elem },
                    (Bound::Len { lo, hi, .. }, Domain::Map { key, val, .. }) => Domain::Map { len: ISet::range(*lo as i128, *hi as i128), key, val },
                    (_, d) => d,
                }
            }
            // A range `i128` cannot hold is not claimed (owner, 2026-10-03:
            // integers of any width): nothing is ruled out, so nothing is
            // proven from it.
            Ty::Int(IntW::Fixed { signed, bits }) => match crate::resolve::int_range_exact(*signed, *bits) {
                Some((lo, hi)) => Domain::Ints(ISet::Range { lo, hi }),
                None => {
                    let (lo, hi) = crate::bigint::type_range(*signed, *bits);
                    Domain::Wide { lo, hi }
                }
            },
            // A width Precog has not chosen yet: the value decides the width,
            // so nothing is ruled out until it does.
            Ty::Int(IntW::Free(_)) => Domain::Ints(ISet::Range { lo: i64::MIN as i128, hi: i64::MAX as i128 }),
            // Each width holds a finite range, and that range is a fact
            // about the type as much as `i8`'s is.
            Ty::Bin(w) => {
                let m = bin_max(*w);
                Domain::Bins { lo: m.map(|x| -x), hi: m }
            }
            Ty::Brain => {
                let m = bin_max(BinW::Fixed(crate::types::BRAIN_BITS));
                Domain::Bins { lo: m.map(|x| -x), hi: m }
            }
            Ty::Bool => Domain::Bools(true, true),
            // A vector: what any of its lanes can be.
            Ty::Simd { elem, .. } => Domain::of_ty(elem, items),
            Ty::Str(_) => Domain::Text { len: Lengths::any(), values: None, classes: Vec::new() },
            Ty::Char(_) => Domain::Chars { values: None },
            Ty::Seq { elem, .. } => Domain::Seq { len: ISet::Range { lo: 0, hi: i64::MAX as i128 }, elem: Box::new(Domain::of_ty(elem, items)) },
            Ty::Map { key, val, .. } => Domain::Map { len: ISet::Range { lo: 0, hi: i64::MAX as i128 }, key: Box::new(Domain::of_ty(key, items)), val: Box::new(Domain::of_ty(val, items)) },
            // A struct is its fields, so what is known about one field is
            // known about it: a guardrail on `'s'.scores` reaches the list.
            Ty::Struct { id, args } => {
                let info = &items.structs[id.0 as usize];
                Domain::Struct(info.fields.iter().map(|f| Domain::of_ty(&f.ty.subst(args), items)).collect())
            }
            Ty::Duration => Domain::Duration(ISet::Range { lo: 0, hi: i64::MAX as i128 }),
            _ => Domain::Any,
        }
    }

    /// Everything either side allows: what a join of two paths knows.
    /// Whole numbers from `lo` to `hi`, at any size: `Ints` where both
    /// fit `i128`.
    pub fn wide(lo: crate::bigint::Big, hi: crate::bigint::Big) -> Domain {
        match (lo.to_i128(), hi.to_i128()) {
            (Some(l), Some(h)) => Domain::Ints(ISet::range(l, h)),
            _ => Domain::Wide { lo, hi },
        }
    }

    /// The ends of a whole-number domain, at any size.
    pub fn wide_bounds(&self) -> Option<(crate::bigint::Big, crate::bigint::Big)> {
        match self {
            Domain::Ints(s) => s.bounds().map(|(l, h)| (crate::bigint::Big::from_i128(l), crate::bigint::Big::from_i128(h))),
            Domain::Wide { lo, hi } => Some((lo.clone(), hi.clone())),
            _ => None,
        }
    }

    pub fn union(&self, other: &Domain) -> Domain {
        match (self, other) {
            (Domain::Empty, d) | (d, Domain::Empty) => d.clone(),
            (Domain::Wide { .. }, Domain::Ints(_) | Domain::Wide { .. }) | (Domain::Ints(_), Domain::Wide { .. }) => match (self.wide_bounds(), other.wide_bounds()) {
                (Some((l1, h1)), Some((l2, h2))) => Domain::wide(l1.min(l2), h1.max(h2)),
                _ => Domain::Any,
            },
            (Domain::Ints(a), Domain::Ints(b)) => Domain::Ints(a.union(b)),
            (Domain::Bins { lo: l1, hi: h1 }, Domain::Bins { lo: l2, hi: h2 }) => Domain::Bins {
                lo: match (l1, l2) {
                    (Some(a), Some(b)) => Some(a.min(*b)),
                    _ => None,
                },
                hi: match (h1, h2) {
                    (Some(a), Some(b)) => Some(a.max(*b)),
                    _ => None,
                },
            },
            (Domain::Bools(t1, f1), Domain::Bools(t2, f2)) => Domain::Bools(*t1 || *t2, *f1 || *f2),
            (Domain::Text { len: l1, values: v1, classes: c1 }, Domain::Text { len: l2, values: v2, classes: c2 }) => {
                let values = match (v1, v2) {
                    (Some(a), Some(b)) if a.len() + b.len() <= SET_CAP => {
                        let mut v = a.clone();
                        v.extend(b.iter().cloned());
                        v.sort();
                        v.dedup();
                        Some(v)
                    }
                    _ => None,
                };
                let classes: Vec<Class> = c1.iter().copied().filter(|c| c2.contains(c)).collect();
                Domain::Text { len: l1.union(l2), values, classes }
            }
            (Domain::Seq { len: l1, elem: e1 }, Domain::Seq { len: l2, elem: e2 }) => Domain::Seq { len: l1.union(l2), elem: Box::new(e1.union(e2)) },
            (Domain::Map { len: l1, key: k1, val: v1 }, Domain::Map { len: l2, key: k2, val: v2 }) => Domain::Map { len: l1.union(l2), key: Box::new(k1.union(k2)), val: Box::new(v1.union(v2)) },
            (Domain::Struct(a), Domain::Struct(b)) if a.len() == b.len() => Domain::Struct(a.iter().zip(b).map(|(x, y)| x.union(y)).collect()),
            (Domain::Enum(a), Domain::Enum(b)) => {
                let mut out = a.clone();
                for (case, d) in b {
                    match out.iter_mut().find(|(c, _)| c == case) {
                        Some((_, existing)) => *existing = existing.union(d),
                        None => out.push((*case, d.clone())),
                    }
                }
                out.sort_by_key(|(c, _)| *c);
                Domain::Enum(out)
            }
            (Domain::Duration(a), Domain::Duration(b)) => Domain::Duration(a.union(b)),
            _ => Domain::Any,
        }
    }

    /// The integer set, when this domain is integers.
    pub fn ints(&self) -> Option<&ISet> {
        match self {
            Domain::Ints(s) => Some(s),
            _ => None,
        }
    }

    /// How the report and a message describe it.
    /// A faithful name for a domain: two domains share one only when they
    /// are the same domain.
    ///
    /// This is what tells one call site from another when Precog runs a
    /// function over its arguments. `describe` is prose for a reader and
    /// says "anything its type allows" for a struct, an enum and a value
    /// nothing is known about alike — so using it here meant
    /// `get[P[x = 5]]` and `get[P[x = 9]]` looked like the same call, and
    /// the first one's answer was handed to the second. A cache key has to
    /// say everything the answer depends on.
    pub fn key(&self) -> String {
        let mut s = String::new();
        self.write_key(&mut s);
        s
    }

    fn write_key(&self, out: &mut String) {
        use core::fmt::Write as _;
        match self {
            Domain::Any => out.push_str("any"),
            Domain::Empty => out.push_str("none"),
            Domain::Wide { lo, hi } => {
                let _ = write!(out, "W[{},{}]", lo, hi);
            }
            Domain::Ints(s) => {
                out.push('i');
                iset_key(s, out);
            }
            Domain::Duration(s) => {
                out.push('d');
                iset_key(s, out);
            }
            Domain::Bins { lo, hi } => {
                // By the bits, so that two different numbers never share a
                // key however they would print.
                let _ = write!(out, "b({:?},{:?})", lo.map(f64::to_bits), hi.map(f64::to_bits));
            }
            Domain::Bools(t, f) => {
                let _ = write!(out, "l({},{})", t, f);
            }
            Domain::Text { len, values, classes } => {
                out.push_str("t(");
                iset_key(&len.bytes, out);
                iset_key(&len.points, out);
                iset_key(&len.graphemes, out);
                match values {
                    Some(v) => {
                        for x in v {
                            let _ = write!(out, "{:?}", x);
                        }
                    }
                    None => out.push('*'),
                }
                let _ = write!(out, "{:?})", classes);
            }
            Domain::Chars { values } => match values {
                Some(v) => {
                    let _ = write!(out, "c{:?}", v);
                }
                None => out.push_str("c*"),
            },
            Domain::Seq { len, elem } => {
                out.push_str("s(");
                iset_key(len, out);
                elem.write_key(out);
                out.push(')');
            }
            Domain::Map { len, key, val } => {
                out.push_str("m(");
                iset_key(len, out);
                key.write_key(out);
                val.write_key(out);
                out.push(')');
            }
            Domain::Struct(fs) => {
                out.push_str("r(");
                for f in fs {
                    f.write_key(out);
                    out.push(',');
                }
                out.push(')');
            }
            Domain::Enum(cases) => {
                out.push_str("e(");
                for (c, d) in cases {
                    let _ = write!(out, "{}:", c);
                    d.write_key(out);
                    out.push(',');
                }
                out.push(')');
            }
        }
    }

    pub fn describe(&self) -> String {
        match self {
            Domain::Wide { lo, hi } if lo == hi => format!("{}", lo),
            Domain::Wide { lo, hi } => format!("{} to {}", lo, hi),
            Domain::Ints(ISet::Exact(v)) if v.len() == 1 => format!("{}", v[0]),
            Domain::Ints(ISet::Exact(v)) if v.len() <= 6 => format!("one of {}", v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ")),
            Domain::Ints(ISet::Exact(v)) => format!("{} values from {} to {}", v.len(), v.first().unwrap(), v.last().unwrap()),
            Domain::Ints(ISet::Range { lo, hi }) => format!("{} to {}", lo, hi),
            Domain::Ints(ISet::Empty) => "no value".into(),
            Domain::Bins { lo: Some(a), hi: Some(b) } => format!("{} to {}", a, b),
            Domain::Bins { .. } => "any bin".into(),
            Domain::Bools(true, false) => "true".into(),
            Domain::Bools(false, true) => "false".into(),
            Domain::Bools(..) => "true or false".into(),
            Domain::Text { values: Some(v), .. } if v.len() == 1 => format!("{:?}", v[0]),
            Domain::Text { len, .. } => match len.bytes.bounds() {
                Some((a, b)) if a == b => format!("text of {} bytes", a),
                Some((a, b)) => format!("text of {} to {} bytes", a, b),
                None => "no text".into(),
            },
            Domain::Seq { len, .. } => match len.bounds() {
                Some((a, b)) if a == b => format!("{} elements", a),
                Some((a, b)) => format!("{} to {} elements", a, b),
                None => "no list".into(),
            },
            Domain::Map { len, .. } => match len.bounds() {
                Some((a, b)) if a == b => format!("{} entries", a),
                Some((a, b)) => format!("{} to {} entries", a, b),
                None => "no map".into(),
            },
            Domain::Duration(s) => match s.single() {
                Some(v) => format!("{} nanos", v),
                None => "a duration".into(),
            },
            Domain::Chars { values: Some(v) } if v.len() == 1 => format!("`{}`", v[0]),
            Domain::Chars { .. } => "a character".into(),
            Domain::Struct(_) | Domain::Enum(_) | Domain::Any => "anything its type allows".into(),
            Domain::Empty => "no value".into(),
        }
    }
}

/// The width a set of integers needs, narrowest first — the `:=` rule
/// (`core-features.md`), before the speed adjustment.
pub fn narrowest_int(s: &ISet) -> Option<(bool, u32)> {
    let (lo, hi) = s.bounds()?;
    for signed in [false, true] {
        if signed || lo >= 0 {
            for bits in [8u32, 16, 32, 64, 128] {
                let (min, max) = crate::resolve::int_range(signed, bits);
                if lo >= min && hi <= max {
                    return Some((signed, bits));
                }
            }
        }
    }
    None
}

/// `SeqKind::Free` resolves to `array` when nothing ever grows it; that is
/// a MIR question (is there an `append`/`remove`?), answered by Precog.
pub fn seq_kind_of(ty: &Ty) -> Option<SeqKind> {
    match ty.base() {
        Ty::Seq { kind, .. } => Some(*kind),
        _ => None,
    }
}

/// The encoding a text domain suggests, when nothing has fixed one.
pub fn enc_of(ty: &Ty) -> Option<Enc> {
    match ty.base() {
        Ty::Str(e) => Some(*e),
        _ => None,
    }
}

/// The largest finite value each binary float width holds. `f16` is
/// IEEE 754 half precision, whose largest is 65504.
pub fn bin_max(w: BinW) -> Option<f64> {
    match w {
        BinW::Fixed(16) => Some(65504.0),
        // The brain float: (2 - 2^-7) x 2^127 (owner, 2026-10-02).
        BinW::Fixed(crate::types::BRAIN_BITS) => Some(3.3895313892515355e38),
        BinW::Fixed(32) => Some(f32::MAX as f64),
        BinW::Fixed(64) | BinW::Free(_) => Some(f64::MAX),
        // Software arithmetic, and wider than anything this analysis
        // carries: nothing is ruled out.
        BinW::Fixed(128) => None,
        _ => None,
    }
}

/// A constant as the target actually stores it
/// (`reproducible-builds.md`: compile-time arithmetic matches the target).
/// `f16` and `f128` are not modelled: the value is kept as written and
/// only ever used to answer range questions, never folded into the binary.
pub fn bin_at_width(v: f64, w: BinW) -> f64 {
    match w {
        BinW::Fixed(32) => v as f32 as f64,
        _ => v,
    }
}

/// The widths a bin domain can take.
pub fn bin_width(ty: &Ty) -> Option<BinW> {
    match ty.base() {
        Ty::Bin(w) => Some(*w),
        // A brain16 is followed as a bin of its own width code.
        Ty::Brain => Some(BinW::Fixed(crate::types::BRAIN_BITS)),
        _ => None,
    }
}

// ---------------------------------------------------------------- arithmetic

/// Every pairwise result, while both sides are small enough to walk; an
/// interval otherwise. Sound either way: the interval contains every
/// pairwise result.
fn pointwise(a: &ISet, b: &ISet, f: impl Fn(i128, i128) -> Option<i128>) -> ISet {
    if let (Some(av), Some(bv)) = (a.values(), b.values()) {
        if av.len() * bv.len() <= SET_CAP {
            let mut out: Vec<i128> = Vec::with_capacity(av.len() * bv.len());
            for x in &av {
                for y in &bv {
                    match f(*x, *y) {
                        Some(v) => out.push(v),
                        // No answer for this pair (a division by zero): the
                        // check site is what refuses it, not the domain.
                        None => {}
                    }
                }
            }
            if out.is_empty() {
                return ISet::Empty;
            }
            out.sort_unstable();
            out.dedup();
            return ISet::Exact(out);
        }
    }
    ISet::Empty
}

/// The four corners of two intervals, for an operation that is monotone in
/// each argument (`+`, `-`, `x`).
fn corners(a: &ISet, b: &ISet, f: impl Fn(i128, i128) -> Option<i128>) -> ISet {
    let (Some((a1, a2)), Some((b1, b2))) = (a.bounds(), b.bounds()) else { return ISet::Empty };
    let mut lo = None;
    let mut hi = None;
    for x in [a1, a2] {
        for y in [b1, b2] {
            let Some(v) = f(x, y) else { continue };
            lo = Some(lo.map_or(v, |l: i128| l.min(v)));
            hi = Some(hi.map_or(v, |h: i128| h.max(v)));
        }
    }
    match (lo, hi) {
        (Some(l), Some(h)) => ISet::Range { lo: l, hi: h },
        _ => ISet::Empty,
    }
}

impl ISet {
    /// A bitwise operator: exact for known values (two's complement in 128
    /// bits agrees with every narrower width, sign-extended), and a safe
    /// range otherwise; `None` when nothing short of the type can be said.
    pub fn bitwise(&self, o: &ISet, op: &crate::ast::BinOp) -> Option<ISet> {
        use crate::ast::BinOp as B;
        let f = |x: i128, y: i128| -> Option<i128> {
            match op {
                B::AndBit => Some(x & y),
                B::OrBit => Some(x | y),
                B::Xor => Some(x ^ y),
                // A count outside 0 to width - 1 is its own check site; it
                // has no answer here.
                B::Shl if (0..127).contains(&y) => Some(x.saturating_mul(1i128 << y)),
                B::Shr if (0..127).contains(&y) => Some(x >> y),
                _ => None,
            }
        };
        let exact = pointwise(self, o, f);
        if !exact.is_empty() {
            return Some(exact);
        }
        let ((a1, a2), (b1, b2)) = (self.bounds()?, o.bounds()?);
        // A shift by a count inside the width is `x x 2^k` or `x / 2^k`
        // floored, monotone in each argument whatever the sign: the corners
        // bound it. Negative values shifted used to widen to the whole type
        // (check claude, seed 58).
        if matches!(op, B::Shl | B::Shr) && b1 >= 0 && b2 < 127 {
            return Some(corners(self, o, f));
        }
        // `x and.bit m` with one side never negative: that side's sign bit
        // is 0, so the answer's is, and its bits are some of that side's —
        // 0 to its largest, whatever the other side is (check claude, seed
        // 7141: `('x' and.bit 16383)` of any i16 is 0 to 16383).
        if *op == B::AndBit && (a1 < 0 || b1 < 0) {
            return match (a1 >= 0, b1 >= 0) {
                (true, _) => Some(ISet::range(0, a2)),
                (_, true) => Some(ISet::range(0, b2)),
                _ => None,
            };
        }
        if a1 < 0 || b1 < 0 {
            return None;
        }
        let spread = |v: i128| -> i128 { if v <= 0 { 0 } else { (1i128 << (128 - v.leading_zeros()).min(126)) - 1 } };
        match op {
            B::AndBit => Some(ISet::range(0, a2.min(b2))),
            B::OrBit | B::Xor => Some(ISet::range(0, spread(a2.max(b2)))),
            B::Shr => Some(ISet::range(a1 >> b2.min(126), a2 >> b1.min(126))),
            B::Shl if b2 < 126 => Some(ISet::range(a1.saturating_mul(1i128 << b1), a2.saturating_mul(1i128 << b2))),
            _ => None,
        }
    }

    // An answer too big for 128 bits is *saturated*, never dropped. It was
    // dropped, and `'n' x 10000000000000000000` with `'n'` anything a
    // `u64` holds came out as exactly 0 — the corner that overflowed was
    // the one that mattered — and the multiply was "proven" to fit. A
    // saturated end is outside every type a program can write, so the site
    // that computed it can never be proven by it.
    pub fn add(&self, o: &ISet) -> ISet {
        self.arith(o, |x, y| Some(x.saturating_add(y)), true)
    }
    pub fn sub(&self, o: &ISet) -> ISet {
        self.arith(o, |x, y| Some(x.saturating_sub(y)), true)
    }
    pub fn mul(&self, o: &ISet) -> ISet {
        self.arith(o, |x, y| Some(x.saturating_mul(y)), true)
    }
    /// `div.X` (`operators.md`). Division by zero has no answer and simply
    /// is not in the result. Each rounding is monotone in the same ways, so
    /// the same ends bound all four.
    pub fn div_round(&self, o: &ISet, r: crate::ast::Round) -> ISet {
        use crate::ast::Round;
        match r {
            Round::Floor | Round::Bare => self.div_floor(o),
            Round::Trunc => self.div_trunc(o),
            _ => self.arith(o, |x, y| crate::rounding::int_divmod(x, y, r).map(|(q, _)| q), false),
        }
    }
    /// `mod.X`: what `div.X` leaves over.
    pub fn mod_round(&self, o: &ISet, r: crate::ast::Round) -> ISet {
        use crate::ast::Round;
        match r {
            Round::Floor | Round::Bare => self.mod_floor(o),
            Round::Trunc => self.mod_trunc(o),
            Round::Ceil => self.mod_ceil(o),
            Round::Near => self.mod_near(o),
        }
    }
    /// Floor division (`operators.md`).
    pub fn div_floor(&self, o: &ISet) -> ISet {
        // Floor, not Euclid: they agree for a positive divisor and differ
        // for a negative one (`7 / -2` floors to -4; Euclid says -3).
        self.arith(
            o,
            |x, y| {
                if y == 0 {
                    return None;
                }
                let q = x.checked_div(y).unwrap_or(i128::MAX);
                let r = x.checked_rem(y).unwrap_or(0);
                Some(if r != 0 && ((r < 0) != (y < 0)) { q.saturating_sub(1) } else { q })
            },
            false,
        )
    }
    /// The mathematician's `mod`: the result has the divisor's sign.
    pub fn mod_floor(&self, o: &ISet) -> ISet {
        // The mathematician's `mod`: the remainder takes the divisor's sign,
        // and `a == (a / b) x b + (a mod b)` with `/` flooring
        // (`operators.md`). `1 mod -3` is -2, not -1.
        let f = |x: i128, y: i128| -> Option<i128> {
            if y == 0 {
                return None;
            }
            let r = x.checked_rem(y).unwrap_or(0);
            Some(if r != 0 && ((r < 0) != (y < 0)) { r + y } else { r })
        };
        if self.is_empty() || o.is_empty() {
            return ISet::Empty;
        }
        let exact = pointwise(self, o, f);
        if !exact.is_empty() {
            return exact;
        }
        // Not from the ends of the dividend's range: a remainder is not
        // monotone. Both ends of `u64` leave 0 when divided by 255 —
        // 2^64 - 1 is a multiple of it — and "always 0" was folded into
        // the binary. What is always true is the divisor's bound.
        let (Some((a1, a2)), Some((b1, b2))) = (self.bounds(), o.bounds()) else { return ISet::Empty };
        if b1 > 0 {
            // 0 to b - 1; a non-negative dividend below the divisor is itself.
            let hi = (b2 - 1).min(if a1 >= 0 { a2 } else { i128::MAX });
            ISet::range(0, hi)
        } else if b2 < 0 {
            let lo = (b1 + 1).max(if a2 <= 0 { a1 } else { i128::MIN });
            ISet::range(lo, 0)
        } else {
            // A divisor of either sign, or 0 and one sign: `x mod 1` is 0.
            ISet::range(b1.saturating_add(1).min(0), b2.saturating_sub(1).max(0))
        }
    }
    /// `div.trunc`: the quotient rounded toward zero, C's `/`
    /// (`operators.md`). Monotone in the same ways as the floored one, so
    /// the same ends bound it.
    pub fn div_trunc(&self, o: &ISet) -> ISet {
        self.arith(o, |x, y| if y == 0 { None } else { Some(x.checked_div(y).unwrap_or(i128::MAX)) }, false)
    }
    /// `mod.trunc`: the remainder takes the dividend's sign, C's `%`.
    /// `-7 mod.trunc 2` is -1.
    pub fn mod_trunc(&self, o: &ISet) -> ISet {
        let f = |x: i128, y: i128| -> Option<i128> { if y == 0 { None } else { Some(x.checked_rem(y).unwrap_or(0)) } };
        if self.is_empty() || o.is_empty() {
            return ISet::Empty;
        }
        let exact = pointwise(self, o, f);
        if !exact.is_empty() {
            return exact;
        }
        // Below the divisor's size, on the dividend's side of zero, and
        // never further from zero than the dividend.
        let (Some((a1, a2)), Some((b1, b2))) = (self.bounds(), o.bounds()) else { return ISet::Empty };
        let m = b1.unsigned_abs().max(b2.unsigned_abs()).min(i128::MAX as u128) as i128;
        let lo = if a1 >= 0 { 0 } else { a1.max(1 - m).min(0) };
        let hi = if a2 <= 0 { 0 } else { a2.min(m - 1).max(0) };
        ISet::range(lo, hi)
    }
    /// `mod.ceil`: the divisor's other sign, below its size, and never
    /// further from zero than a dividend on that side.
    fn mod_ceil(&self, o: &ISet) -> ISet {
        let f = |x: i128, y: i128| crate::rounding::int_divmod(x, y, crate::ast::Round::Ceil).map(|(_, m)| m);
        if self.is_empty() || o.is_empty() {
            return ISet::Empty;
        }
        let exact = pointwise(self, o, f);
        if !exact.is_empty() {
            return exact;
        }
        let (Some((a1, a2)), Some((b1, b2))) = (self.bounds(), o.bounds()) else { return ISet::Empty };
        if b1 > 0 {
            let lo = (1 - b2).max(if a2 <= 0 { a1 } else { i128::MIN });
            ISet::range(lo, 0)
        } else if b2 < 0 {
            let hi = (b1.saturating_neg() - 1).min(if a1 >= 0 { a2 } else { i128::MAX });
            ISet::range(0, hi)
        } else {
            let m = b1.unsigned_abs().max(b2.unsigned_abs()).min(i128::MAX as u128) as i128;
            ISet::range(1 - m.max(1), m.max(1) - 1)
        }
    }
    /// `mod.near`: at most half the divisor's size either way, and never
    /// further from zero than the dividend.
    fn mod_near(&self, o: &ISet) -> ISet {
        let f = |x: i128, y: i128| crate::rounding::int_divmod(x, y, crate::ast::Round::Near).map(|(_, m)| m);
        if self.is_empty() || o.is_empty() {
            return ISet::Empty;
        }
        let exact = pointwise(self, o, f);
        if !exact.is_empty() {
            return exact;
        }
        let (Some((a1, a2)), Some((b1, b2))) = (self.bounds(), o.bounds()) else { return ISet::Empty };
        let half = (b1.unsigned_abs().max(b2.unsigned_abs()) / 2).min(i128::MAX as u128) as i128;
        let far = a1.unsigned_abs().max(a2.unsigned_abs()).min(i128::MAX as u128) as i128;
        let m = half.min(far);
        ISet::range(-m, m)
    }
    pub fn pow(&self, o: &ISet) -> ISet {
        let f = |x: i128, y: i128| -> Option<i128> {
            if y < 0 {
                return None;
            }
            // Saturated, like the others: a power too big to write down
            // is still a power too big for the type.
            let mut acc: i128 = 1;
            let mut i = 0;
            while i < y {
                let next = acc.saturating_mul(x);
                if next == acc && (acc == i128::MAX || acc == i128::MIN) {
                    break;
                }
                acc = next;
                if acc == 0 || acc == 1 {
                    break;
                }
                i += 1;
                if i > 256 {
                    break;
                }
            }
            if acc == 1 && x == -1 && y % 2 == 1 {
                acc = -1;
            }
            Some(acc)
        };
        if self.is_empty() || o.is_empty() {
            return ISet::Empty;
        }
        let exact = pointwise(self, o, f);
        if !exact.is_empty() {
            return exact;
        }
        let (Some((a1, a2)), Some((b1, b2))) = (self.bounds(), o.bounds()) else { return ISet::Empty };
        let b1 = b1.max(0);
        if b2 < 0 {
            return ISet::Empty;
        }
        if a1 >= 0 {
            // Non-negative base, non-negative power: monotone in the base,
            // and at either end of the powers for a fixed one (0 and 1 are
            // the bases that are not increasing).
            let at = |x: i128, y: i128| f(x, y).unwrap_or(i128::MAX);
            return ISet::range(at(a1, b1).min(at(a1, b2)), at(a2, b1).max(at(a2, b2)));
        }
        // A negative base: the sign flips with the power, and a zero in the
        // range is a value too. Only the size is bounded.
        let m = a1.saturating_abs().max(a2.saturating_abs());
        let top = f(m, b2).unwrap_or(i128::MAX);
        // Saturated, an odd power of a negative base is MIN, one past -MAX.
        ISet::range(if top == i128::MAX { i128::MIN } else { -top }, top)
    }
    pub fn neg(&self) -> ISet {
        match self {
            ISet::Empty => ISet::Empty,
            ISet::Exact(v) => {
                let mut out: Vec<i128> = v.iter().map(|x| x.saturating_neg()).collect();
                out.sort_unstable();
                out.dedup();
                ISet::Exact(out)
            }
            ISet::Range { lo, hi } => ISet::Range { lo: hi.saturating_neg(), hi: lo.saturating_neg() },
        }
    }

    fn arith(&self, o: &ISet, f: impl Fn(i128, i128) -> Option<i128> + Copy, monotone: bool) -> ISet {
        if self.is_empty() || o.is_empty() {
            return ISet::Empty;
        }
        let exact = pointwise(self, o, f);
        if !exact.is_empty() {
            return exact;
        }
        if monotone {
            corners(self, o, f)
        } else {
            // Division: monotone in the dividend for a divisor of one sign,
            // and in the divisor for a dividend of one sign, so the ends
            // and the divisors nearest zero (-1 and 1) are the extremes.
            // `mod` and powers are not like this and have their own.
            match (self.bounds(), o.bounds()) {
                (Some((a1, a2)), Some((b1, b2))) => {
                    let mut lo: Option<i128> = None;
                    let mut hi: Option<i128> = None;
                    for x in [a1, a2] {
                        for y in [b1, b2, -1, 1] {
                            if y < b1 || y > b2 {
                                continue;
                            }
                            let Some(v) = f(x, y) else { continue };
                            lo = Some(lo.map_or(v, |l| l.min(v)));
                            hi = Some(hi.map_or(v, |h| h.max(v)));
                        }
                    }
                    match (lo, hi) {
                        (Some(l), Some(h)) => ISet::Range { lo: l, hi: h },
                        _ => ISet::Empty,
                    }
                }
                _ => ISet::Empty,
            }
        }
    }

    /// Can `self op other` be true? Can it be false? Both may be true.
    pub fn compare(&self, o: &ISet, op: Cmp) -> (bool, bool) {
        let (Some((a1, a2)), Some((b1, b2))) = (self.bounds(), o.bounds()) else { return (false, false) };
        let (always, never) = match op {
            Cmp::Lt => (a2 < b1, a1 >= b2),
            Cmp::Le => (a2 <= b1, a1 > b2),
            Cmp::Gt => (a1 > b2, a2 <= b1),
            Cmp::Ge => (a1 >= b2, a2 < b1),
            Cmp::Eq => {
                let one = self.single().zip(o.single()).map(|(x, y)| x == y).unwrap_or(false);
                (one, a2 < b1 || b2 < a1)
            }
            Cmp::Ne => {
                let one = self.single().zip(o.single()).map(|(x, y)| x == y).unwrap_or(false);
                (a2 < b1 || b2 < a1, one)
            }
        };
        (!never, !always)
    }

    /// What `self` can be, given `self op other` held.
    pub fn assume(&self, o: &ISet, op: Cmp) -> ISet {
        let Some((b1, b2)) = o.bounds() else { return self.clone() };
        match op {
            Cmp::Lt => self.clamp(i128::MIN / 4, b2.saturating_sub(1)),
            Cmp::Le => self.clamp(i128::MIN / 4, b2),
            Cmp::Gt => self.clamp(b1.saturating_add(1), i128::MAX / 4),
            Cmp::Ge => self.clamp(b1, i128::MAX / 4),
            Cmp::Eq => self.meet(o),
            Cmp::Ne => match o.single() {
                Some(v) => match self.values() {
                    Some(vs) => {
                        let kept: Vec<i128> = vs.into_iter().filter(|x| *x != v).collect();
                        if kept.is_empty() {
                            ISet::Empty
                        } else {
                            ISet::Exact(kept)
                        }
                    }
                    // A range too wide to list loses the value only at an
                    // end: a `u32` that is not 0 is 1 or more — what a
                    // `#strict` file's `'v' !== 0 and.then (15 mod 'v')`
                    // needs, with no caller to say what 'v' is (check
                    // claude, seed 6430).
                    None => match self.bounds() {
                        Some((lo, hi)) if v == lo && lo < hi => self.clamp(lo + 1, hi),
                        Some((lo, hi)) if v == hi && lo < hi => self.clamp(lo, hi - 1),
                        _ => self.clone(),
                    },
                },
                None => self.clone(),
            },
        }
    }
}

impl Domain {
    /// What a bin can be, given a comparison held.
    pub fn assume_bin(&self, other: &Domain, op: Cmp) -> Domain {
        let (Domain::Bins { lo: a1, hi: a2 }, Domain::Bins { lo: b1, hi: b2 }) = (self, other) else { return self.clone() };
        let tighten_hi = |cur: Option<f64>, limit: Option<f64>| match (cur, limit) {
            (Some(c), Some(l)) => Some(c.min(l)),
            (None, l) => l,
            (c, None) => c,
        };
        let tighten_lo = |cur: Option<f64>, limit: Option<f64>| match (cur, limit) {
            (Some(c), Some(l)) => Some(c.max(l)),
            (None, l) => l,
            (c, None) => c,
        };
        match op {
            Cmp::Lt | Cmp::Le => Domain::Bins { lo: *a1, hi: tighten_hi(*a2, *b2) },
            Cmp::Gt | Cmp::Ge => Domain::Bins { lo: tighten_lo(*a1, *b1), hi: *a2 },
            Cmp::Eq => Domain::Bins { lo: tighten_lo(*a1, *b1), hi: tighten_hi(*a2, *b2) },
            Cmp::Ne => self.clone(),
        }
    }

    /// Can a bin comparison be true? Can it be false?
    pub fn compare_bin(&self, other: &Domain, op: Cmp) -> (bool, bool) {
        let (Domain::Bins { lo: a1, hi: a2 }, Domain::Bins { lo: b1, hi: b2 }) = (self, other) else { return (true, true) };
        let (Some(a1), Some(a2), Some(b1), Some(b2)) = (*a1, *a2, *b1, *b2) else { return (true, true) };
        let (always, never) = match op {
            Cmp::Lt => (a2 < b1, a1 >= b2),
            Cmp::Le => (a2 <= b1, a1 > b2),
            Cmp::Gt => (a1 > b2, a2 <= b1),
            Cmp::Ge => (a1 >= b2, a2 < b1),
            Cmp::Eq => (a1 == a2 && b1 == b2 && a1 == b1, a2 < b1 || b2 < a1),
            Cmp::Ne => (a2 < b1 || b2 < a1, a1 == a2 && b1 == b2 && a1 == b1),
        };
        (!never, !always)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmp {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

impl Cmp {
    pub fn flip(self) -> Cmp {
        match self {
            Cmp::Lt => Cmp::Gt,
            Cmp::Le => Cmp::Ge,
            Cmp::Gt => Cmp::Lt,
            Cmp::Ge => Cmp::Le,
            Cmp::Eq => Cmp::Eq,
            Cmp::Ne => Cmp::Ne,
        }
    }
    pub fn negate(self) -> Cmp {
        match self {
            Cmp::Lt => Cmp::Ge,
            Cmp::Le => Cmp::Gt,
            Cmp::Gt => Cmp::Le,
            Cmp::Ge => Cmp::Lt,
            Cmp::Eq => Cmp::Ne,
            Cmp::Ne => Cmp::Eq,
        }
    }
}

impl Domain {
    /// Widen towards the type's own domain: what a loop does when a set
    /// keeps growing (`precog.md`, layer 2). Never narrower than the union,
    /// so it stays sound; it only gives up precision.
    pub fn widen(&self, next: &Domain, ty: &Ty, items: &crate::resolve::Items) -> Domain {
        self.widen_to(next, ty, items, &[])
    }

    /// Widening *with thresholds*: a bound still moving jumps to the next
    /// constant the function mentions (sorted in `steps`), and to the
    /// type's limit only past the last. Jumping straight to the limit lost
    /// every invariant a loop keeps below one: `'acc' = (… ) mod 1000000007`
    /// grows by a few each trip, widened to `i64`'s top, and the `+` in
    /// front of the `mod` could no longer be proven. It stays finite — a
    /// bound only ever moves up the list.
    pub fn widen_to(&self, next: &Domain, ty: &Ty, items: &crate::resolve::Items, steps: &[i128]) -> Domain {
        let joined = self.union(next);
        // Past `i128` a bound still moving goes to the type's end.
        if let (Domain::Wide { .. }, Some(old)) = (&joined, self.wide_bounds()) {
            let new = joined.wide_bounds().unwrap();
            if new != old {
                return match Domain::of_ty(ty, items) {
                    d @ Domain::Wide { .. } => d,
                    _ => Domain::Any,
                };
            }
            return joined;
        }
        match (&joined, self) {
            (Domain::Ints(new), Domain::Ints(old)) => {
                let (Some((n1, n2)), Some((o1, o2))) = (new.bounds(), old.bounds()) else { return joined };
                let top = match Domain::of_ty(ty, items) {
                    Domain::Ints(s) => s.bounds().unwrap_or((i64::MIN as i128, i64::MAX as i128)),
                    _ => (i64::MIN as i128, i64::MAX as i128),
                };
                let lo = if n1 < o1 { steps.iter().rev().copied().find(|t| *t <= n1 && *t >= top.0).unwrap_or(top.0) } else { n1 };
                let hi = if n2 > o2 { steps.iter().copied().find(|t| *t >= n2 && *t <= top.1).unwrap_or(top.1) } else { n2 };
                Domain::Ints(ISet::range(lo, hi))
            }
            // A bin bound that moved goes to the type's end: a sum of bins
            // grows by its step each time round and never settles.
            (Domain::Bins { lo: n1, hi: n2 }, Domain::Bins { lo: o1, hi: o2 }) => {
                let (tlo, thi) = match Domain::of_ty(ty, items) {
                    Domain::Bins { lo, hi } => (lo, hi),
                    _ => (None, None),
                };
                let lo = match (n1, o1) {
                    (Some(n), Some(o)) if n >= o => Some(*n),
                    _ => tlo,
                };
                let hi = match (n2, o2) {
                    (Some(n), Some(o)) if n <= o => Some(*n),
                    _ => thi,
                };
                Domain::Bins { lo, hi }
            }
            (Domain::Seq { len, elem }, Domain::Seq { len: old, .. }) => {
                let (Some((n1, n2)), Some((o1, o2))) = (len.bounds(), old.bounds()) else { return joined.clone() };
                let lo = if n1 < o1 { 0 } else { n1 };
                let hi = if n2 > o2 { i64::MAX as i128 } else { n2 };
                Domain::Seq { len: ISet::range(lo, hi), elem: elem.clone() }
            }
            (Domain::Text { len, values, classes }, Domain::Text { len: old, .. }) => {
                let widen_one = |n: &ISet, o: &ISet| -> ISet {
                    match (n.bounds(), o.bounds()) {
                        (Some((n1, n2)), Some((o1, o2))) => ISet::range(if n1 < o1 { 0 } else { n1 }, if n2 > o2 { i64::MAX as i128 } else { n2 }),
                        _ => n.clone(),
                    }
                };
                Domain::Text {
                    len: Lengths { bytes: widen_one(&len.bytes, &old.bytes), points: widen_one(&len.points, &old.points), graphemes: widen_one(&len.graphemes, &old.graphemes) },
                    values: values.clone().filter(|v| v.len() <= 8),
                    classes: classes.clone(),
                }
            }
            _ => joined,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::BinOp;

    /// `and.bit` with one side never negative is 0 to that side's largest,
    /// whatever the other (check claude, seed 7141).
    #[test]
    fn and_bit_with_a_mask_is_never_negative() {
        let any_int16 = ISet::range(-32768, 32767);
        let mask = ISet::one(16383);
        assert_eq!(any_int16.bitwise(&mask, &BinOp::AndBit), Some(ISet::range(0, 16383)));
        assert_eq!(mask.bitwise(&any_int16, &BinOp::AndBit), Some(ISet::range(0, 16383)));
        // Both sides possibly negative: nothing short of the type.
        assert_eq!(any_int16.bitwise(&ISet::range(-5, 5), &BinOp::AndBit), None);
        // And every pair it covers agrees with the machine.
        for x in [-32768i128, -20000, -1, 0, 1, 12768, 32767] {
            let got = x & 16383;
            assert!((0..=16383).contains(&got), "{} and.bit 16383 is {}", x, got);
        }
    }
}

/// Whole-number arithmetic over intervals at any size (owner, 2026-10-03:
/// integers of any width), for what `ISet` cannot hold: the answer's
/// ends, or `None` where nothing short of the type can be said. `b` is
/// `None` for a negation.
pub fn wide_arith(op: &crate::ast::BinOp, a: &(crate::bigint::Big, crate::bigint::Big), b: Option<&(crate::bigint::Big, crate::bigint::Big)>) -> Option<(crate::bigint::Big, crate::bigint::Big)> {
    use crate::ast::BinOp as B;
    use crate::bigint::Big;
    let (a1, a2) = a;
    let Some((b1, b2)) = b else {
        return match op {
            B::Sub => Some((a2.neg(), a1.neg())),
            _ => None,
        };
    };
    let ends = |vs: Vec<Big>| -> Option<(Big, Big)> {
        let lo = vs.iter().min()?.clone();
        let hi = vs.iter().max()?.clone();
        (!lo.huge() && !hi.huge()).then_some((lo, hi))
    };
    match op {
        B::Add | B::WrapAdd => ends(vec![a1.add(b1), a2.add(b2)]),
        B::Sub | B::WrapSub => ends(vec![a1.sub(b2), a2.sub(b1)]),
        B::Mul | B::WrapMul => ends(vec![a1.mul(b1), a1.mul(b2), a2.mul(b1), a2.mul(b2)]),
        B::Div(r) | B::DivWrap(r) | B::DivIeee(r) => {
            // Monotone in each for a divisor of one sign: the ends, and the
            // divisors nearest zero, as `ISet`'s division.
            let zero = Big::zero();
            let mut divisors = vec![b1.clone(), b2.clone()];
            for d in [Big::from_i128(-1), Big::from_i128(1)] {
                if *b1 <= d && d <= *b2 {
                    divisors.push(d);
                }
            }
            let mut vs = Vec::new();
            for x in [a1, a2] {
                for d in &divisors {
                    if *d != zero {
                        vs.push(x.divmod(d, *r)?.0);
                    }
                }
            }
            ends(vs)
        }
        B::Mod(r) => {
            if a1 == a2 && b1 == b2 {
                let m = a1.divmod(b1, *r)?.1;
                return Some((m.clone(), m));
            }
            // Never further from zero than the divisor's size.
            let m = if b1.is_neg() { b1.neg() } else { b1.clone() }.max(if b2.is_neg() { b2.neg() } else { b2.clone() });
            Some((m.neg(), m))
        }
        // A shift by a known count is a product or a floored quotient by
        // its power of two; out of range is its site's.
        B::Shl | B::Shr => {
            let k = b1.to_i128().filter(|k| b1 == b2 && *k >= 0 && *k <= crate::types::WIDEST_INT as i128)? as u32;
            let p = (Big::pow2(k), Big::pow2(k));
            if *op == B::Shl {
                wide_arith(&B::Mul, a, Some(&p))
            } else {
                wide_arith(&B::Div(crate::ast::Round::Floor), a, Some(&p))
            }
        }
        B::Pow | B::WrapPow => {
            let e = b1.to_i128().filter(|e| b1 == b2 && *e >= 0)? as u64;
            if a1 == a2 {
                let v = a1.pow(e)?;
                return Some((v.clone(), v));
            }
            // An even power of a range across zero: 0 up; otherwise the
            // ends, which a power keeps in order for whole numbers.
            let (p1, p2) = (a1.pow(e)?, a2.pow(e)?);
            if e % 2 == 0 && a1.is_neg() && !a2.is_neg() {
                return ends(vec![Big::zero(), p1, p2]);
            }
            ends(vec![p1, p2])
        }
        _ => None,
    }
}
