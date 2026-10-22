//! `deci` literals, worked out at compile time (`types.md`, owner
//! 2026-09-29): a literal is exactly the decimal written, in the machine's
//! layout (BID here, as the runtime's `deci.rs` holds it), and one with
//! more digits than the width holds is an error, never rounded.

/// The digits a `deci` of `bits` holds.
pub fn digits(bits: u16) -> u32 {
    match bits {
        32 => 7,
        64 => 16,
        _ => 34,
    }
}

/// The smallest exponent (of the last digit) a width reaches.
/// The largest exponent a coefficient can take: emax less the digits
/// past the first.
fn qmax(bits: u16) -> i64 {
    match bits {
        32 => 90,
        64 => 369,
        _ => 6111,
    }
}

fn qmin(bits: u16) -> i64 {
    match bits {
        32 => -101,
        64 => -398,
        _ => -6176,
    }
}

fn ebits(bits: u16) -> u32 {
    match bits {
        32 => 8,
        64 => 10,
        _ => 14,
    }
}

/// Why a written number is not a `deci` of the width.
pub enum LitErr {
    /// It has this many digits.
    Digits(u32),
    /// Its last digit is below the smallest place the width reaches.
    Range,
}

/// The layout of `neg x coef x 10^exp`.
pub fn encode(neg: bool, coef: u128, exp: i64, bits: u16) -> u128 {
    let w = bits as u32;
    let e = (exp - qmin(bits)) as u128;
    // A deci has no -0 (owner, 2026-10-07).
    let sign = ((neg && coef != 0) as u128) << (w - 1);
    let small = w - 1 - ebits(bits);
    if coef < (1u128 << small) {
        sign | (e << small) | coef
    } else {
        let low = w - 3 - ebits(bits);
        sign | (3u128 << (w - 3)) | (e << low) | (coef & ((1u128 << low) - 1))
    }
}

/// `neg x coef x 10^exp` back out of the layout `encode` writes.
pub fn decode(v: u128, bits: u16) -> (bool, u128, i64) {
    let w = bits as u32;
    let mask = |n: u32| -> u128 { if n >= 128 { u128::MAX } else { (1u128 << n) - 1 } };
    let neg = (v >> (w - 1)) & 1 == 1;
    let eb = ebits(bits);
    let (e, coef) = if (v >> (w - 3)) & 3 == 3 {
        let low = w - 3 - eb;
        ((v >> low) & mask(eb), (4u128 << low) | (v & mask(low)))
    } else {
        let small = w - 1 - eb;
        ((v >> small) & mask(eb), v & mask(small))
    };
    (neg, coef, e as i64 + qmin(bits))
}

/// Two decimals compared exactly, however far apart their exponents:
/// by sign, then by where the leading digit sits, then digit by digit.
pub fn compare(a: (bool, u128, i64), b: (bool, u128, i64)) -> std::cmp::Ordering {
    use std::cmp::Ordering::*;
    let norm = |(neg, mut c, mut e): (bool, u128, i64)| {
        if c == 0 {
            return (false, String::new(), 0i64);
        }
        while c % 10 == 0 {
            c /= 10;
            e += 1;
        }
        let s = c.to_string();
        let lead = s.len() as i64 + e;
        (neg, s, lead)
    };
    let (an, ad, al) = norm(a);
    let (bn, bd, bl) = norm(b);
    let magnitude = || -> std::cmp::Ordering {
        match (ad.is_empty(), bd.is_empty()) {
            (true, true) => return Equal,
            (true, false) => return Less,
            (false, true) => return Greater,
            _ => {}
        }
        if al != bl {
            return al.cmp(&bl);
        }
        let n = ad.len().max(bd.len());
        format!("{:0<n$}", ad, n = n).cmp(&format!("{:0<n$}", bd, n = n))
    };
    match (an && !ad.is_empty(), bn && !bd.is_empty()) {
        (false, true) => Greater,
        (true, false) => Less,
        (false, false) => magnitude(),
        (true, true) => magnitude().reverse(),
    }
}

/// A literal as written (`19.99`, `-1.5`, `1_000`) at `bits`.
pub fn literal(text: &str, bits: u16) -> Result<u128, LitErr> {
    let t: String = text.chars().filter(|c| *c != '_').collect();
    let (neg, t) = match t.strip_prefix('-') {
        Some(rest) => (true, rest.to_string()),
        None => (false, t),
    };
    let (int, frac) = match t.split_once('.') {
        Some((i, f)) => (i.to_string(), f.to_string()),
        None => (t.clone(), String::new()),
    };
    let all = format!("{}{}", int, frac);
    let mut sig = all.trim_start_matches('0');
    let mut exp = -(frac.len() as i64);
    // Trailing zeros are not digits the type must hold, as the oracle
    // counts them: `10000000` is 1 × 10^7, which a `d32` holds
    // (std claude, 2026-10-20). They are dropped only when there are too
    // many, so a literal that fits keeps the zeros it was written with.
    if sig.len() as u32 > digits(bits) {
        let kept = sig.trim_end_matches('0');
        exp += (sig.len() - kept.len()) as i64;
        sig = kept;
    }
    let n = sig.len() as u32;
    if n > digits(bits) {
        return Err(LitErr::Digits(n));
    }
    let coef: u128 = if sig.is_empty() { 0 } else { sig.parse().map_err(|_| LitErr::Range)? };
    if exp < qmin(bits) || exp > qmax(bits) {
        return Err(LitErr::Range);
    }
    Ok(encode(neg, coef, exp, bits))
}

/// How many digits the largest value of a whole-number type has: whether
/// it always fits a `deci` (`types.md`: "an `i32` into a `d64`
/// always fits and takes no `otherwise`").
pub fn int_digits(signed: bool, bits: u32) -> u32 {
    let (signed, bits) = crate::resolve::storage(signed, bits);
    if bits <= 128 {
        let max: u128 = if bits == 128 { if signed { i128::MAX as u128 } else { u128::MAX } } else if signed { (1u128 << (bits - 1)) - 1 } else { (1u128 << bits) - 1 };
        return max.to_string().len() as u32;
    }
    // 2^b - 1 has floor(b log10 2) + 1 digits; no power of 2 is a power
    // of 10, so the floor is never on an edge.
    let b = if signed { bits - 1 } else { bits };
    (b as f64 * std::f64::consts::LOG10_2).floor() as u32 + 1
}

/// What a conversion into or out of a `deci` can fail with, in the order
/// the checker lists them (a failure arm is its place in this list), or
/// None for one that is not a `deci`'s. `literal_digits`: a bare whole
/// number is its own digits, not its type's.
pub fn conv_fails(from: &crate::types::Ty, to: &crate::types::Ty, literal_digits: Option<u32>) -> Option<Vec<&'static str>> {
    use crate::types::{IntW, Ty};
    let mut fails = match (from.base(), to.base()) {
        (Ty::Int(w), Ty::Deci(b)) => {
            let have = match (literal_digits, w) {
                (Some(n), _) => n,
                (None, IntW::Fixed { signed, bits }) => int_digits(*signed, *bits),
                (None, _) => int_digits(true, 64),
            };
            if have > digits(*b) {
                vec!["too_many_digits"]
            } else {
                vec![]
            }
        }
        (Ty::Bin(_), Ty::Deci(_)) => vec![],
        (Ty::Deci(a), Ty::Deci(b)) => {
            if b < a {
                vec!["too_many_digits"]
            } else {
                vec![]
            }
        }
        // Too big for the integer fails too (owner, 2026-09-30: a).
        (Ty::Deci(_), Ty::Int(_)) => vec!["not_whole", "out_of_range"],
        (Ty::Deci(_), Ty::Bin(_) | Ty::Str(_)) => vec![],
        (Ty::Str(_), Ty::Deci(_)) => vec!["not_a_number", "out_of_range", "too_many_digits"],
        _ => return None,
    };
    if to.bound().is_some() && !fails.contains(&"out_of_range") {
        fails.push("out_of_range");
    }
    Some(fails)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering::*;

    fn d(t: &str, bits: u16) -> (bool, u128, i64) {
        decode(literal(t, bits).ok().unwrap(), bits)
    }

    #[test]
    fn decode_undoes_encode_and_compare_is_exact() {
        for (t, bits) in [("0.79", 64), ("-1.5", 64), ("19.99", 32), ("9999999999999999", 64), ("0", 128), ("-0.000001", 128)] {
            let v = literal(t, bits).ok().unwrap();
            let (n, c, e) = decode(v, bits);
            assert_eq!(encode(n, c, e, bits), v, "{}", t);
        }
        assert_eq!(compare(d("0.79", 64), d("0.0", 128)), Greater);
        assert_eq!(compare(d("0.79", 64), d("1.0", 128)), Less);
        assert_eq!(compare(d("1.00", 64), d("1", 128)), Equal);
        assert_eq!(compare(d("-0.5", 64), d("0.0", 128)), Less);
        assert_eq!(compare(d("-2", 64), d("-10", 128)), Greater);
        assert_eq!(compare(d("0.0001", 64), d("0.001", 128)), Less);
        assert_eq!(compare(d("-0", 64), d("0", 128)), Equal);
    }
}
