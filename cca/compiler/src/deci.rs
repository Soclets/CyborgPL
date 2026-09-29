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
    let sign = (neg as u128) << (w - 1);
    let small = w - 1 - ebits(bits);
    if coef < (1u128 << small) {
        sign | (e << small) | coef
    } else {
        let low = w - 3 - ebits(bits);
        sign | (3u128 << (w - 3)) | (e << low) | (coef & ((1u128 << low) - 1))
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
    let sig = all.trim_start_matches('0');
    let n = sig.len() as u32;
    if n > digits(bits) {
        return Err(LitErr::Digits(n));
    }
    let coef: u128 = if sig.is_empty() { 0 } else { sig.parse().map_err(|_| LitErr::Range)? };
    let exp = -(frac.len() as i64);
    if exp < qmin(bits) {
        return Err(LitErr::Range);
    }
    Ok(encode(neg, coef, exp, bits))
}

/// How many digits the largest value of a whole-number type has: whether
/// it always fits a `deci` (`types.md`: "an `int32` into a `deci64`
/// always fits and takes no `otherwise`").
pub fn int_digits(signed: bool, bits: u16) -> u32 {
    let max: u128 = if bits >= 128 { if signed { i128::MAX as u128 } else { u128::MAX } } else if signed { (1u128 << (bits - 1)) - 1 } else { (1u128 << bits) - 1 };
    max.to_string().len() as u32
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
        (Ty::Deci(_), Ty::Int(_)) => vec!["not_whole"],
        (Ty::Deci(_), Ty::Bin(_) | Ty::Str(_)) => vec![],
        (Ty::Str(_), Ty::Deci(_)) => vec!["not_a_number", "out_of_range", "too_many_digits"],
        _ => return None,
    };
    if to.bound().is_some() && !fails.contains(&"out_of_range") {
        fails.push("out_of_range");
    }
    Some(fails)
}
