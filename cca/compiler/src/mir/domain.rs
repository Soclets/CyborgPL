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
        } else if (hi - lo) < SET_REOPEN as i128 {
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
            ISet::Range { lo, hi } => u64::try_from(hi - lo + 1).ok(),
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
            (ISet::Exact(a), ISet::Exact(b)) if a.len() + b.len() <= SET_CAP => {
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
            ISet::Range { lo, hi } if (hi - lo) < SET_CAP as i128 => Some((*lo..=*hi).collect()),
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

#[derive(Debug, Clone, PartialEq)]
pub enum Domain {
    /// Nothing is known beyond the type (or the type says everything).
    Any,
    Ints(ISet),
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
    /// layer 2 starts, and it is already a real fact: a `uint8` is 0..255,
    /// an `int32[0, 100]` is 0..100, a `list[T][0, 5]` has 0..5 elements.
    pub fn of_ty(ty: &Ty) -> Domain {
        match ty {
            Ty::Bounded { inner, bound } => {
                let inner_domain = Domain::of_ty(inner);
                match (bound, inner_domain) {
                    (Bound::Int { lo, hi }, Domain::Ints(s)) => Domain::Ints(s.clamp(*lo, *hi)),
                    (Bound::Bin { lo, hi }, _) => Domain::Bins { lo: lo.parse().ok(), hi: hi.parse().ok() },
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
            Ty::Int(IntW::Fixed { signed, bits }) => {
                let (lo, hi) = crate::resolve::int_range(*signed, *bits);
                Domain::Ints(ISet::Range { lo, hi })
            }
            // A width Precog has not chosen yet: the value decides the width,
            // so nothing is ruled out until it does.
            Ty::Int(IntW::Free(_)) => Domain::Ints(ISet::Range { lo: i64::MIN as i128, hi: i64::MAX as i128 }),
            Ty::Bin(_) => Domain::Bins { lo: None, hi: None },
            Ty::Bool => Domain::Bools(true, true),
            Ty::Str(_) => Domain::Text { len: Lengths::any(), values: None, classes: Vec::new() },
            Ty::Char(_) => Domain::Chars { values: None },
            Ty::Seq { elem, .. } => Domain::Seq { len: ISet::Range { lo: 0, hi: i64::MAX as i128 }, elem: Box::new(Domain::of_ty(elem)) },
            Ty::Map { key, val, .. } => Domain::Map { len: ISet::Range { lo: 0, hi: i64::MAX as i128 }, key: Box::new(Domain::of_ty(key)), val: Box::new(Domain::of_ty(val)) },
            Ty::Duration => Domain::Duration(ISet::Range { lo: 0, hi: i64::MAX as i128 }),
            _ => Domain::Any,
        }
    }

    /// Everything either side allows: what a join of two paths knows.
    pub fn union(&self, other: &Domain) -> Domain {
        match (self, other) {
            (Domain::Empty, d) | (d, Domain::Empty) => d.clone(),
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
    pub fn describe(&self) -> String {
        match self {
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
pub fn narrowest_int(s: &ISet) -> Option<(bool, u16)> {
    let (lo, hi) = s.bounds()?;
    for signed in [false, true] {
        if signed || lo >= 0 {
            for bits in [8u16, 16, 32, 64, 128] {
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

/// The widths a bin domain can take.
pub fn bin_width(ty: &Ty) -> Option<BinW> {
    match ty.base() {
        Ty::Bin(w) => Some(*w),
        _ => None,
    }
}
