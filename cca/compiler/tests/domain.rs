//! Layer 1's value sets, against arithmetic.
//!
//! A value set that leaves out a value the program can have is a wrong
//! fold, not a missed proof: `mod` took the ends of the dividend's range,
//! both ends of `uint64` leave 0 divided by 255, and "always 0" went into
//! the binary (check claude, generator seed 106). These walk every pair in
//! ranges too big for the exact path and ask whether each answer is in the
//! set.

use cca::ast::BinOp;
use cca::mir::domain::ISet;

/// Windows of each sign and across zero, wide enough that the product of
/// two is past the exact path's cap.
fn windows(width: i128) -> Vec<(i128, i128)> {
    let mut out = Vec::new();
    for lo in [-3 * width, -width - 7, -width / 2, -3, 0, 1, 5, width / 3, 2 * width] {
        out.push((lo, lo + width));
    }
    out
}

fn check(name: &str, a: (i128, i128), b: (i128, i128), got: &ISet, truth: impl Fn(i128, i128) -> Option<i128>) {
    for x in a.0..=a.1 {
        for y in b.0..=b.1 {
            if let Some(v) = truth(x, y) {
                assert!(
                    got.contains(v),
                    "{} of {:?} and {:?}: {} {} {} is {}, not in {:?}",
                    name, a, b, x, name, y, v, got
                );
            }
        }
    }
}

fn floor_mod(x: i128, y: i128) -> Option<i128> {
    if y == 0 {
        return None;
    }
    let r = x % y;
    Some(if r != 0 && ((r < 0) != (y < 0)) { r + y } else { r })
}

fn floor_div(x: i128, y: i128) -> Option<i128> {
    if y == 0 {
        return None;
    }
    let (q, r) = (x / y, x % y);
    Some(if r != 0 && ((r < 0) != (y < 0)) { q - 1 } else { q })
}

/// A power, saturated at 128 bits the way the domain saturates.
fn power(x: i128, y: i128) -> Option<i128> {
    if y < 0 {
        return None;
    }
    Some(match x.checked_pow(y as u32) {
        Some(v) => v,
        None if x < 0 && y % 2 == 1 => i128::MIN,
        None => i128::MAX,
    })
}

#[test]
fn a_remainder_is_every_remainder_the_range_can_leave() {
    for a in windows(300) {
        for b in windows(30) {
            let got = ISet::range(a.0, a.1).mod_floor(&ISet::range(b.0, b.1));
            check("mod", a, b, &got, floor_mod);
        }
    }
    // The finding itself: all of `uint64` by 255 is not 0.
    let got = ISet::range(0, u64::MAX as i128).mod_floor(&ISet::one(255));
    assert!(got.contains(1) && got.contains(254), "{:?}", got);
}

#[test]
fn a_quotient_is_every_quotient() {
    for a in windows(300) {
        for b in windows(30) {
            let got = ISet::range(a.0, a.1).div_floor(&ISet::range(b.0, b.1));
            check("/", a, b, &got, floor_div);
        }
    }
}

#[test]
fn a_power_is_every_power() {
    for a in [(-100, 100), (-100, -1), (0, 200), (-7, 250), (1, 300)] {
        for b in [(0, 20), (1, 21), (3, 40)] {
            let got = ISet::range(a.0, a.1).pow(&ISet::range(b.0, b.1));
            check("xx", a, b, &got, power);
        }
    }
}

#[test]
fn a_bitwise_answer_is_in_the_set() {
    let ops: [(BinOp, fn(i128, i128) -> Option<i128>); 5] = [
        (BinOp::AndBit, |x, y| Some(x & y)),
        (BinOp::OrBit, |x, y| Some(x | y)),
        (BinOp::Xor, |x, y| Some(x ^ y)),
        (BinOp::Shl, |x, y| if (0..127).contains(&y) { Some(x.saturating_mul(1i128 << y)) } else { None }),
        (BinOp::Shr, |x, y| if (0..127).contains(&y) { Some(x >> y) } else { None }),
    ];
    for (op, truth) in ops {
        let shift = matches!(op, BinOp::Shl | BinOp::Shr);
        let values: &[(i128, i128)] = if shift { &[(0, 300), (5, 900), (1000, 1300), (-900, -91), (-400, 300), (-32767, -32000)] } else { &[(0, 300), (5, 900), (1000, 1300)] };
        for &a in values {
            for b in [(0, 40), (3, 60), (200, 260), (0, 0)] {
                let Some(got) = ISet::range(a.0, a.1).bitwise(&ISet::range(b.0, b.1), &op) else { continue };
                check(&format!("{:?}", op), a, b, &got, truth);
            }
        }
    }
}

/// Bin intervals (`precog.md`: "Bins are intervals from the start"): every
/// answer the target's IEEE arithmetic gives for operands in the two
/// intervals, at 32 and at 64 bits, is inside what Precog says.
#[test]
fn a_bin_answer_is_in_the_interval() {
    use cca::mir::domain::Domain;
    let mut seed: u64 = 0x2545_f491_4f6c_dd1d;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed
    };
    let mut unit = move || (next() >> 11) as f64 / (1u64 << 53) as f64;
    let ops = [BinOp::Add, BinOp::Sub, BinOp::Mul, BinOp::Div];
    for round in 0..4000 {
        let scale = [1e-40, 1e-3, 1.0, 3e3, 1.6e7, 1e30][round % 6];
        let mut pick = |u: &mut dyn FnMut() -> f64| {
            let a = (u() - 0.5) * scale;
            let b = a + u() * scale;
            (a, b)
        };
        let (a, b) = (pick(&mut unit), pick(&mut unit));
        for op in &ops {
            for bits in [32u16, 64] {
                // The endpoints as the width holds them.
                let at = |v: f64| if bits == 32 { v as f32 as f64 } else { v };
                let (a, b) = ((at(a.0), at(a.1)), (at(b.0), at(b.1)));
                let Some(Domain::Bins { lo: Some(lo), hi: Some(hi) }) = cca::precog::bin_interval(op, a, b, bits) else { continue };
                for i in 0..=8 {
                    for j in 0..=8 {
                        let x = at(a.0 + (a.1 - a.0) * i as f64 / 8.0);
                        let y = at(b.0 + (b.1 - b.0) * j as f64 / 8.0);
                        let v = if bits == 32 {
                            let (x, y) = (x as f32, y as f32);
                            (match op { BinOp::Add => x + y, BinOp::Sub => x - y, BinOp::Mul => x * y, _ => x / y }) as f64
                        } else {
                            match op { BinOp::Add => x + y, BinOp::Sub => x - y, BinOp::Mul => x * y, _ => x / y }
                        };
                        assert!(lo <= v && v <= hi, "{:?} at {} bits of {:?} and {:?}: {} {:?} {} is {}, not in {}..{}", op, bits, a, b, x, op, y, v, lo, hi);
                    }
                }
            }
        }
    }
}
