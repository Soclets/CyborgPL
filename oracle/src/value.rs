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
        Ty::Int(w) => format!("{}int{}", if matches!(w, IntWidth::Other(_)) { "!std_" } else { "" }, w_num(w)),
        Ty::Uint(w) => format!("{}uint{}", if matches!(w, IntWidth::Other(_)) { "!std_" } else { "" }, w_num(w)),
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
        Ty::Ranged(t, lo, hi) => format!("{}[{}, {}]", ty_name(t), bound_name(lo), bound_name(hi)),
        Ty::Channel(t) => format!("channel[{}]", ty_name(t)),
        Ty::Duration => "std::duration".into(),
        Ty::Simd(t, n) => format!("simd[{}, {n}]", ty_name(t)),
        Ty::Map(k, v, o) => format!("map{}[{}, {}]", match o { crate::ast::MapOrder::Any => "", crate::ast::MapOrder::Ordered => ".ordered", crate::ast::MapOrder::Unordered => ".unordered" }, ty_name(k), ty_name(v)),
        Ty::TextRanged(e, lo, hi, u) => format!("{}[{}, {} {}]", ty_name(&Ty::Str(e.clone())), lo, hi, unit_word(u)),
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

pub fn bound_name(b: &crate::ast::Bound) -> String { match b { crate::ast::Bound::Min => "min".into(), crate::ast::Bound::Max => "max".into(), crate::ast::Bound::Int(i) => i.to_string(), crate::ast::Bound::Bin(x) => format!("{x}") } }

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
