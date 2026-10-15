//! How a division rounds (owner, 2026-10-02): `div.floor` down, `div.ceil`
//! up, `div.trunc` toward zero, `div.near` to the nearest with a tie to
//! even, and `mod.X` what its `div.X` leaves over, `a == (a div.X b) x b +
//! (a mod.X b)` (`operators.md`). The one meaning, worked on true values,
//! for everything that works a division out before the program runs:
//! consts, Precog's sets, the tests.

use crate::ast::Round;

/// The quotient and remainder of `x / y` on whole numbers, rounded as
/// named. `None` for a zero divisor, or an answer past 128 bits.
pub fn int_divmod(x: i128, y: i128, r: Round) -> Option<(i128, i128)> {
    if y == 0 {
        return None;
    }
    // Toward zero first, the hardware's; then one step where the rounding
    // asks for it.
    let q = x.checked_div(y)?;
    let m = x - q * y;
    if m == 0 {
        return Some((q, 0));
    }
    // Which way the true quotient lies from `q`: up when the remainder
    // and the divisor share a sign.
    let up = (m < 0) == (y < 0);
    let step = match r {
        Round::Trunc => false,
        Round::Floor | Round::Bare => !up,
        Round::Ceil => up,
        Round::Near => {
            // |m| against |y| - |m|: the nearer whole number, a tie to even.
            let (am, ay) = (m.unsigned_abs(), y.unsigned_abs());
            let rest = ay - am;
            am > rest || (am == rest && q % 2 != 0)
        }
    };
    if !step {
        return Some((q, m));
    }
    if up {
        Some((q.checked_add(1)?, m - y))
    } else {
        Some((q.checked_sub(1)?, m + y))
    }
}

/// `div.X` and `mod.X` on bins, at 64 bits, as the oracle works them
/// (0d25799): `fmod`, the quotient `(x - r) / y` rounded to a whole number,
/// then one step for `floor` (`q - 1`, `r + y`) or `ceil` (`q + 1`, `r -
/// y`) where the rounding lies that way — the true quotient, rounded as
/// named, Python's `//`. A remainder the step brings to the divisor's size
/// is 0, so it stays below the divisor: `mod.floor` takes the divisor's
/// sign there and whenever it is 0, `mod.ceil` is `+0` there, and any other
/// 0 is `fmod`'s, the dividend's sign. `div.near` is ordinary division and
/// has no remainder.
pub fn bin_divmod(x: f64, y: f64, r: Round) -> (f64, f64) {
    if r == Round::Near {
        return (x / y, f64::NAN);
    }
    let m = x % y;
    let q = ((x - m) / y).round_ties_even();
    let differ = m != 0.0 && (m < 0.0) != (y < 0.0);
    let same = m != 0.0 && (m < 0.0) == (y < 0.0);
    match r {
        Round::Floor | Round::Bare => {
            let (mut q, mut m) = (q, m);
            if differ {
                q -= 1.0;
                m = if m + y == y { 0.0 } else { m + y };
            }
            if m == 0.0 {
                m = 0.0f64.copysign(y);
            }
            (q, m)
        }
        Round::Ceil if same => (q + 1.0, if m - y == -y { 0.0 } else { m - y }),
        _ => (q, m),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `examples/div_rounding.cpls`, checked against Python by design
    /// claude.
    #[test]
    fn the_four_roundings_on_whole_numbers() {
        let both = |x, y, r| int_divmod(x, y, r).unwrap();
        assert_eq!([both(7, 2, Round::Floor), both(7, 2, Round::Ceil), both(7, 2, Round::Trunc), both(7, 2, Round::Near)], [(3, 1), (4, -1), (3, 1), (4, -1)]);
        assert_eq!([both(-7, 2, Round::Floor), both(-7, 2, Round::Ceil), both(-7, 2, Round::Trunc), both(-7, 2, Round::Near)], [(-4, 1), (-3, -1), (-3, -1), (-4, 1)]);
        assert_eq!(both(5, 2, Round::Near), (2, 1));
        assert_eq!(both(-127, 2, Round::Near), (-64, 1));
        assert_eq!(both(-128, -1, Round::Near), (128, 0));
        // Every pair keeps the identity, and a nearest remainder is at most
        // half the divisor.
        for x in -40..=40 {
            for y in [-7, -3, -2, -1, 1, 2, 3, 7] {
                for r in [Round::Floor, Round::Ceil, Round::Trunc, Round::Near] {
                    let (q, m) = both(x, y, r);
                    assert_eq!(q * y + m, x);
                    assert!(m.abs() < y.abs());
                }
                let (_, m) = both(x, y, Round::Near);
                assert!(2 * m.abs() <= y.abs());
            }
        }
    }

    #[test]
    fn bins_as_python() {
        assert_eq!(bin_divmod(7.5, 2.0, Round::Floor), (3.0, 1.5));
        assert_eq!(bin_divmod(7.5, 2.0, Round::Ceil), (4.0, -0.5));
        assert_eq!(bin_divmod(-370.0, 360.0, Round::Floor), (-2.0, 350.0));
        assert_eq!(bin_divmod(-370.0, 360.0, Round::Trunc), (-1.0, -10.0));
        // The signed zeros the oracle gives.
        let sign = |v: f64| v.is_sign_negative();
        assert!(!sign(bin_divmod(-4.0, 2.0, Round::Floor).1));
        assert!(sign(bin_divmod(-4.0, 2.0, Round::Ceil).1));
        assert!(!sign(bin_divmod(4.0, 2.0, Round::Ceil).1));
        assert!(!sign(bin_divmod(-1.0, 3.0, Round::Ceil).0));
        assert!(sign(bin_divmod(1.0, -3.0, Round::Ceil).0));
        assert!(!sign(bin_divmod(-1e-20, -360.0, Round::Ceil).1));
        assert_eq!(bin_divmod(-1e-20, 360.0, Round::Floor), (-1.0, 0.0));
        // `examples/div_edges.cpls`: the true quotient, the remainder 0.
        assert_eq!(bin_divmod(1e-20, 2.0, Round::Ceil), (1.0, 0.0));
    }
}
