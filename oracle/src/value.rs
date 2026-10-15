//! Runtime values and the two ways to print them (design/syntax-sketch.md:
//! `print` shows the stored value, `print.useful` the useful one).

use crate::ast::{CharUnit, IntWidth, Ty};
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i128),
    /// a whole number past i128, for the wide `!std_` integers (owner, 2026-10-03); a value that
    /// fits i128 is always `Int`, never `Big`
    Big(crate::bigint::BigInt),
    Bin(f64),
    /// a brain float (owner, 2026-10-02), held as the exact value it stores
    Brain(f64),
    /// IEEE decimal (design/types.md, owner 2026-09-29)
    Deci(crate::deci::Deci),
    Bool(bool),
    Str(String),
    Char(CharUnit, String),
    List(Vec<Value>),
    /// a duration in nanoseconds (design/concurrency.md)
    Duration(i128),
    /// a channel (design/concurrency.md): shared between tasks
    Chan(Rc<RefCell<Chan>>),
    /// a vector: N plain values side by side, all of one type (design/simd.md)
    Simd(Vec<Value>),
    /// a map, insertion-ordered (the oracle's order; the design leaves iteration open): (key, value) pairs
    Map(Vec<(Value, Value)>),
    /// a struct value: blueprint name + fields in declaration order
    Struct(String, Vec<(String, Value)>),
    /// an enum value (design/enums.md): type name, case name, the case's data if any
    Enum(String, String, Option<Box<Value>>),
    /// a function as a value (design/functions-as-values.md): the definition and its captures
    Func(FuncVal),
    /// a C pointer (design/ffi.md): into a lent Cyborg list/array/str, or from
    /// an emulated extern; `cell: None` is C's null
    Ptr(Ptr),
    Nothing,
}

/// A function value: its definition plus captured variables (name, cell, mutable).
#[derive(Clone)]
pub struct FuncVal {
    pub def: std::rc::Rc<crate::ast::Func>,
    pub captures: Vec<(String, std::rc::Rc<std::cell::RefCell<Value>>, bool)>,
    /// which captures the function owns (`copy`/`give`), beside the ones it borrows (`lend`)
    pub owned: Vec<bool>,
}
impl PartialEq for FuncVal { fn eq(&self, o: &FuncVal) -> bool { std::rc::Rc::ptr_eq(&self.def, &o.def) } }
impl std::fmt::Debug for FuncVal { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "fn {}", self.def.name) } }

#[derive(Debug, Clone)]
pub struct Ptr {
    pub cell: Option<std::rc::Rc<std::cell::RefCell<Value>>>,
    pub index: i128,
    pub mutable: bool,
}

impl PartialEq for Ptr {
    fn eq(&self, o: &Ptr) -> bool {
        match (&self.cell, &o.cell) {
            (None, None) => true,
            (Some(a), Some(b)) => std::rc::Rc::ptr_eq(a, b) && self.index == o.index,
            _ => false,
        }
    }
}

impl Value {
    /// Owned values live on the heap and need give/copy/lend to move around
    /// (design/ownership-syntax.md). Plain values copy silently.
    pub fn is_owned(&self) -> bool {
        match self {
            Value::Str(_) | Value::List(_) | Value::Map(_) | Value::Chan(_) | Value::Char(CharUnit::Grapheme, _) => true,
            // Decided 2026-09-18: a struct is always an owned value, even when every
            // field is plain — a struct owns its fields, so the verbs always apply.
            Value::Struct(..) => true,
            // Decided 2026-09-20: an enum is always owned, like a struct — one rule.
            Value::Enum(..) => true,
            // Decided 2026-09-24 (owner: a): a function value that carries a `copy`/`give` capture
            // owns it, and takes the verbs like a list; one carrying nothing, or only lends, is plain.
            Value::Func(f) => f.owned.iter().any(|o| *o),
            _ => false,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Value::Int(_) | Value::Big(_) => "integer",
            Value::Bin(_) => "bin",
            Value::Brain(_) => "brain16",
            Value::Deci(_) => "deci",
            Value::Bool(_) => "bool",
            Value::Str(_) => "str",
            Value::Char(..) => "char",
            Value::List(_) => "list",
            Value::Map(_) => "map",
            Value::Simd(_) => "simd",
            Value::Duration(_) => "duration",
            Value::Chan(_) => "channel",
            Value::Struct(..) => "struct",
            Value::Enum(..) => "enum",
            Value::Func(..) => "function",
            Value::Ptr(_) => "ptr",
            Value::Nothing => "nothing",
        }
    }

    /// `print`: exactly what is stored. For a bin, the full decimal expansion.
    pub fn show_stored(&self) -> String {
        match self {
            Value::Int(i) => i.to_string(),
            Value::Big(b) => b.to_string(),
            Value::Bin(b) => exact_decimal(*b),
            Value::Brain(b) => exact_decimal(*b),
            Value::Deci(d) => d.show_stored(),
            Value::Bool(b) => if *b { "true".into() } else { "false".into() },
            Value::Str(s) => s.clone(),
            Value::Char(_, c) => c.clone(),
            Value::List(items) => format!("[{}]", items.iter().map(|v| v.show_stored()).collect::<Vec<_>>().join(", ")),
            Value::Map(ps) => format!("map[{}]", ps.iter().map(|(k, v)| format!("{} = {}", k.show_stored(), v.show_stored())).collect::<Vec<_>>().join(", ")),
            Value::Simd(lanes) => format!("simd[{}]", lanes.iter().map(|v| v.show_stored()).collect::<Vec<_>>().join(", ")),
            Value::Duration(n) => show_duration(*n),
            Value::Chan(_) => "channel".into(),
            Value::Struct(n, fs) => format!("{n}[{}]", fs.iter().map(|(k, v)| format!("{k} = {}", v.show_stored())).collect::<Vec<_>>().join(", ")),
            Value::Ptr(p) => if p.cell.is_none() { "null".into() } else { format!("ptr+{}", p.index) },
            Value::Enum(t, c, d) => match d { Some(v) => format!("{t}.{c}[{}]", v.show_stored()), None => format!("{t}.{c}") },
            Value::Func(f) => format!("fn {}", f.def.name),
            Value::Nothing => "nothing".into(),
        }
    }

    /// `print.useful`: the useful value. For a bin, the shortest text that reads back
    /// to the same stored value. Identical for every other type.
    pub fn show_useful(&self) -> String {
        match self {
            Value::Bin(b) => format!("{b}"),
            Value::Brain(b) => brain_useful(*b),
            Value::Deci(d) => d.show_useful(),
            Value::List(items) => format!("[{}]", items.iter().map(|v| v.show_useful()).collect::<Vec<_>>().join(", ")),
            Value::Map(ps) => format!("map[{}]", ps.iter().map(|(k, v)| format!("{} = {}", k.show_useful(), v.show_useful())).collect::<Vec<_>>().join(", ")),
            Value::Struct(n, fs) => format!("{n}[{}]", fs.iter().map(|(k, v)| format!("{k} = {}", v.show_useful())).collect::<Vec<_>>().join(", ")),
            // a case alone is its bare word for people, `big`; `print` and `to.str.utf8` give `Size.big`
            // (owner, 2026-09-30)
            Value::Enum(t, c, d) => match d { Some(v) => format!("{t}.{c}[{}]", v.show_useful()), None => c.clone() },
            other => other.show_stored(),
        }
    }
}

/// An integer type's range at any width, as big numbers (owner, 2026-10-03: integers of any width).
pub fn big_range(ty: &Ty) -> Option<(crate::bigint::BigInt, crate::bigint::BigInt)> {
    use crate::bigint::BigInt;
    let one = BigInt::from_i128(1);
    match ty {
        Ty::Int(w) => { let n = w_num(w) as u64; Some((BigInt::pow2(n - 1).neg(), BigInt::pow2(n - 1).sub(&one))) }
        Ty::Uint(w) => { let n = w_num(w) as u64; Some((BigInt::zero(), BigInt::pow2(n).sub(&one))) }
        _ => None,
    }
}

pub fn int_range(ty: &Ty) -> Option<(i128, i128)> {
    match ty {
        Ty::Int(w) => Some(match w {
            IntWidth::W8 => (i8::MIN as i128, i8::MAX as i128),
            IntWidth::W16 => (i16::MIN as i128, i16::MAX as i128),
            IntWidth::W32 => (i32::MIN as i128, i32::MAX as i128),
            IntWidth::W64 => (i64::MIN as i128, i64::MAX as i128),
            IntWidth::W128 => (i128::MIN, i128::MAX),
            IntWidth::Other(n) if *n > 127 => return None,
            IntWidth::Other(n) => (-(1i128 << (n - 1)), (1i128 << (n - 1)) - 1),
        }),
        Ty::Uint(w) => Some(match w {
            IntWidth::W8 => (0, u8::MAX as i128),
            IntWidth::W16 => (0, u16::MAX as i128),
            IntWidth::W32 => (0, u32::MAX as i128),
            IntWidth::W64 => (0, u64::MAX as i128),
            IntWidth::W128 => (0, i128::MAX), // uint128's top half is beyond the oracle's i128
            IntWidth::Other(n) if *n > 127 => return None,
            IntWidth::Other(n) => (0, if *n >= 127 { i128::MAX } else { (1i128 << n) - 1 }),
        }),
        _ => None,
    }
}

/// A channel's state. `after`/`every` are std::time's: ready only when nothing else can run.
#[derive(Debug, Clone, PartialEq)]
pub struct Chan { pub queue: std::collections::VecDeque<Value>, pub cap: Option<usize>, pub timer: Option<Timer>, pub id: usize }
#[derive(Debug, Clone, PartialEq)]
pub enum Timer { After { fired: bool, nanos: i128 }, Every { nanos: i128 } }

/// `5 seconds` / `1500 millis`: the largest unit that divides evenly, singular for 1.
pub fn show_duration(n: i128) -> String {
    for (name, per) in [("hour", 3_600_000_000_000i128), ("minute", 60_000_000_000), ("second", 1_000_000_000), ("milli", 1_000_000), ("micro", 1_000), ("nano", 1)] {
        if n % per == 0 { let k = n / per; return format!("{k} {name}{}", if k == 1 { "" } else { "s" }); }
    }
    format!("{n} nanos")
}

pub fn unit_word(u: &CharUnit) -> &'static str { match u { CharUnit::Byte => "bytes", CharUnit::Point => "points", CharUnit::Grapheme => "graphemes" } }

pub fn ty_name(ty: &Ty) -> String {
    match ty {
        Ty::Int(w) => format!("{}int{}", if matches!(w, IntWidth::Other(_)) { "not_std." } else { "" }, w_num(w)),
        Ty::Uint(w) => format!("{}uint{}", if matches!(w, IntWidth::Other(_)) { "not_std." } else { "" }, w_num(w)),
        Ty::Bin(n) => format!("bin{n}"),
        Ty::Brain => "brain16".into(),
        Ty::Deci(n) => format!("deci{n}"),
        Ty::Str(e) => format!("str.{}", match e { crate::ast::Encoding::Utf8 => "utf8", crate::ast::Encoding::Utf16 => "utf16", crate::ast::Encoding::Utf32 => "utf32", crate::ast::Encoding::Ascii => "ascii" }),
        Ty::Bool => "bool".into(),
        Ty::Char(u) => format!("char.{}", match u { CharUnit::Byte => "byte", CharUnit::Point => "point", CharUnit::Grapheme => "grapheme" }),
        Ty::List(t) => format!("list[{}]", ty_name(t)),
        Ty::Array(t) => format!("array[{}]", ty_name(t)),
        Ty::Struct(n) => n.clone(),
        Ty::Var(n) => format!("<{n}>"),
        // as it is written since 2026-10-12: each side labelled, a side that is the type's own limit left out
        // (a length's low 0 and unlimited high, a channel's low 0)
        Ty::Ranged(t, lo, hi) => {
            let length = matches!(**t, Ty::List(_) | Ty::Array(_) | Ty::Map(..) | Ty::Channel(_));
            let lo_open = matches!(lo, crate::ast::Bound::Min) || (length && *lo == crate::ast::Bound::Int(0));
            let hi_open = matches!(hi, crate::ast::Bound::Max) || (length && *hi == crate::ast::Bound::Int(i128::MAX));
            let mut sides = Vec::new();
            if !lo_open { sides.push(format!("min {}", bound_name(lo))); }
            if !hi_open { sides.push(format!("max {}", bound_name(hi))); }
            if sides.is_empty() { ty_name(t) } else { format!("{}[{}]", ty_name(t), sides.join(", ")) }
        }
        Ty::Channel(t) => format!("channel[{}]", ty_name(t)),
        Ty::Duration => "std::duration".into(),
        Ty::Simd(t, n) => format!("simd[{}, {n}]", ty_name(t)),
        Ty::Map(k, v, o) => format!("map{}[{}, {}]", match o { crate::ast::MapOrder::Any => "", crate::ast::MapOrder::Ordered => ".ordered", crate::ast::MapOrder::Unordered => ".unordered" }, ty_name(k), ty_name(v)),
        Ty::TextRanged(e, lo, hi, u) => {
            let mut sides = Vec::new();
            if *lo != 0 { sides.push(format!("min {lo}")); }
            if *hi != i128::MAX { sides.push(format!("max {hi}")); }
            if sides.is_empty() { sides.push("min 0".into()); }
            format!("{}[{} {}]", ty_name(&Ty::Str(e.clone())), sides.join(", "), unit_word(u))
        }
        Ty::Fn(ps, r, fails) => {
            let ps: Vec<String> = ps.iter().map(|(k, t)| format!("{}{}", match k { crate::ast::ParamKind::Owned => "", crate::ast::ParamKind::Lent => "lent ", crate::ast::ParamKind::LentMut => "lent.mut " }, ty_name(t))).collect();
            let mut s = format!("fn[{}]", ps.join(", "));
            if let Some(r) = r { s.push_str(&format!(" -> {}", ty_name(r))); }
            if !fails.is_empty() { s.push_str(&format!("{} fail[{}]", if r.is_some() { " or" } else { " ->" }, fails.join(", "))); }
            s
        }
        Ty::Inst(n, args) => format!("{n}[{}]", args.iter().map(ty_name).collect::<Vec<_>>().join(", ")),
        Ty::Ptr(t, m) => format!("ptr{}[{}]", if *m { ".mut" } else { "" }, ty_name(t)),
    }
}

pub fn bound_name(b: &crate::ast::Bound) -> String { match b { crate::ast::Bound::Min => "min".into(), crate::ast::Bound::Max => "max".into(), crate::ast::Bound::Int(i) => i.to_string(), crate::ast::Bound::Bin(x) => { let t = format!("{x}"); if t.contains(['.', 'e', 'i', 'N']) { t } else { format!("{t}.0") } } } }   // a bin bound keeps its point, as it is written (compiler claude)

fn w_num(w: &IntWidth) -> u32 {
    match w { IntWidth::W8 => 8, IntWidth::W16 => 16, IntWidth::W32 => 32, IntWidth::W64 => 64, IntWidth::W128 => 128, IntWidth::Other(n) => *n }
}

/// The exact decimal expansion of an IEEE 754 binary64 value — every digit
/// that is really stored. 0.1 → 0.1000000000000000055511151231257827021181583404541015625
pub fn exact_decimal(x: f64) -> String {
    if x.is_nan() { return "NaN".into(); }
    if x.is_infinite() { return if x > 0.0 { "infinity".into() } else { "-infinity".into() }; }
    if x == 0.0 { return if x.is_sign_negative() { "-0".into() } else { "0".into() }; }

    let bits = x.to_bits();
    let neg = (bits >> 63) == 1;
    let exp_bits = ((bits >> 52) & 0x7ff) as i32;
    let frac = bits & ((1u64 << 52) - 1);
    let (mantissa, exp) = if exp_bits == 0 { (frac, -1074) } else { (frac | (1u64 << 52), exp_bits - 1075) };
    // value = mantissa × 2^exp

    // Little-endian base-1e9 big integer.
    let mut big: Vec<u64> = vec![mantissa % 1_000_000_000, mantissa / 1_000_000_000];
    while big.len() > 1 && *big.last().unwrap() == 0 { big.pop(); }
    let mul = |big: &mut Vec<u64>, m: u64| {
        let mut carry = 0u64;
        for d in big.iter_mut() {
            let v = *d * m + carry;
            *d = v % 1_000_000_000;
            carry = v / 1_000_000_000;
        }
        while carry > 0 { big.push(carry % 1_000_000_000); carry /= 1_000_000_000; }
    };

    let mut decimals = 0usize;
    if exp >= 0 {
        for _ in 0..exp { mul(&mut big, 2); }
    } else {
        // mantissa × 2^-k = mantissa × 5^k / 10^k
        for _ in 0..(-exp) { mul(&mut big, 5); }
        decimals = (-exp) as usize;
    }

    let mut digits = String::new();
    write!(digits, "{}", big.last().unwrap()).unwrap();
    for d in big.iter().rev().skip(1) { write!(digits, "{d:09}").unwrap(); }

    let mut out = String::new();
    if neg { out.push('-'); }
    if decimals == 0 {
        out.push_str(&digits);
    } else {
        if digits.len() <= decimals {
            out.push_str("0.");
            for _ in 0..(decimals - digits.len()) { out.push('0'); }
            out.push_str(digits.trim_end_matches('0'));
        } else {
            let split = digits.len() - decimals;
            out.push_str(&digits[..split]);
            let tail = digits[split..].trim_end_matches('0');
            if !tail.is_empty() { out.push('.'); out.push_str(tail); }
        }
    }
    out
}

/// The nearest brain float to `x` (owner, 2026-10-02): through 32 bits and then the top 16 of
/// them, round to nearest even — as LLVM's `fptrunc … to bfloat` and the hardware's `bfcvt` do.
pub fn round_brain(x: f64) -> f64 {
    let f = x as f32;
    if f.is_nan() || f.is_infinite() { return f as f64; }
    let bits = f.to_bits();
    let rounded = bits.wrapping_add(0x7FFF + ((bits >> 16) & 1)) & 0xFFFF_0000;
    f32::from_bits(rounded) as f64
}

/// The nearest brain float to an integer, rounded once (types.md: a number "rounds to the nearest brain16"),
/// not through 32 bits first, which can round twice and miss the nearest (check claude, 2026-10-15, corpus
/// 7457b90a: 25262379159781017 lands on a half step at 32 bits and tied up to 180 x 2^47).
pub fn round_brain_int(i: i128) -> f64 {
    if i == 0 { return 0.0; }
    let m = i.unsigned_abs();
    let len = 128 - m.leading_zeros();
    let mag = if len <= 8 { m as f64 } else {
        let shift = len - 8;
        let mut kept = m >> shift;
        let rem = m & ((1u128 << shift) - 1);
        let half = 1u128 << (shift - 1);
        if rem > half || (rem == half && kept & 1 == 1) { kept += 1; }
        (kept as f64) * 2f64.powi(shift as i32)
    };
    if i < 0 { -mag } else { mag }
}

/// The nearest brain float to a 64-bit value, rounded once (as `round_brain_int`); a value below 32-bit
/// floats' normal range keeps the 32-bit path
pub fn round_brain_direct(x: f64) -> f64 {
    if !x.is_finite() || x == 0.0 || x.abs() < f32::MIN_POSITIVE as f64 { return round_brain(x); }
    let bits = x.to_bits();
    let rounded = bits.wrapping_add((1u64 << 44) - 1 + ((bits >> 45) & 1)) & !((1u64 << 45) - 1);
    let y = f64::from_bits(rounded);
    let max = f32::from_bits(0x7F7F_0000) as f64;   // the largest brain float
    if y.abs() > max { if y < 0.0 { f64::NEG_INFINITY } else { f64::INFINITY } } else { y }
}

/// The nearest brain float to a number written in decimal (a literal, or text), rounded once from its exact
/// value. Its 64-bit value can land exactly on a brain16 half step the decimal is not on (check claude,
/// 2026-10-15, corpus 3877b6f7: 1.01171874999999999999); only then is the decimal itself compared with the
/// half step, exactly. Rounding to 64 bits never crosses a half step, which 64 bits holds exactly.
pub fn round_brain_text(text: &str) -> Option<f64> {
    let t: String = text.chars().filter(|c| *c != '_').collect();
    let x: f64 = t.parse().ok()?;
    let r = round_brain_direct(x);
    if !r.is_finite() || r == x { return Some(r); }
    let up = |v: f64| -> f64 {   // the next brain float away from zero
        let b = (v as f32).to_bits();
        f32::from_bits(b + 0x1_0000) as f64
    };
    let down = |v: f64| -> f64 { let b = (v as f32).to_bits(); if b & 0x7FFF_FFFF == 0 { v } else { f32::from_bits(b - 0x1_0000) as f64 } };
    let other = if x.abs() > r.abs() { up(r) } else { down(r) };
    if (r + other) / 2.0 != x { return Some(r); }
    let ord = cmp_decimal_with(&t, x)?;
    Some(match ord {
        std::cmp::Ordering::Equal => r,
        std::cmp::Ordering::Less => r.min(other),
        std::cmp::Ordering::Greater => r.max(other),
    })
}

/// A decimal written as text against a 64-bit value, exactly
fn cmp_decimal_with(text: &str, m: f64) -> Option<std::cmp::Ordering> {
    use crate::bigint::BigInt;
    let (neg, t) = match text.strip_prefix('-') { Some(r) => (true, r), None => (false, text.strip_prefix('+').unwrap_or(text)) };
    let (mant, exp) = match t.find(['e', 'E']) { Some(i) => (&t[..i], t[i + 1..].parse::<i64>().ok()?), None => (t, 0) };
    let (int, frac) = mant.split_once('.').unwrap_or((mant, ""));
    let mut d = BigInt::parse(&format!("{int}{frac}"))?;
    if neg { d = d.neg(); }
    let p = exp - frac.len() as i64;                 // text = d x 10^p
    let bits = m.to_bits();
    let e = ((bits >> 52) & 0x7FF) as i64;
    let mant_m = (bits & ((1u64 << 52) - 1)) | if e == 0 { 0 } else { 1u64 << 52 };
    let mut mm = BigInt::from_i128(mant_m as i128);
    if m < 0.0 { mm = mm.neg(); }
    let q = if e == 0 { -1074 } else { e - 1075 };   // m = mm x 2^q
    let ten = BigInt::from_i128(10);
    let (mut l, mut r) = (d, mm);
    if p >= 0 { l = l.mul(&ten.pow(p as u64)); } else { r = r.mul(&ten.pow((-p) as u64)); }
    if q >= 0 { r = r.mul(&BigInt::pow2(q as u64)); } else { l = l.mul(&BigInt::pow2((-q) as u64)); }
    Some(l.cmp(&r))
}

/// `print.useful` for a brain float: the shortest decimal that rounds back to the same value.
pub fn brain_useful(x: f64) -> String {
    if x == 0.0 { return if x.is_sign_negative() { "-0".into() } else { "0".into() }; }
    for digits in 1..=9 {
        let s = format!("{:.*e}", digits - 1, x);
        if let Ok(y) = s.parse::<f64>() {
            if round_brain(y) == x { return format!("{}", y); }
        }
    }
    format!("{x}")
}
