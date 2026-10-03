//! Whole numbers of any size, for what does not fit `i128`: integers of
//! any width (owner, 2026-10-03), up to LLVM's 2^23 bits, as Precog
//! reasons about them and as literals are written. Exact; a magnitude in
//! base 2^32, least significant first, and a sign.

use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Big {
    neg: bool,
    mag: Vec<u32>,
}

/// Past this many bits a value is past every width there is, and further
/// arithmetic on it says nothing a check could use.
const CAP_BITS: u64 = 2 * crate::types::WIDEST_INT as u64 + 64;

impl Big {
    pub fn zero() -> Big {
        Big { neg: false, mag: Vec::new() }
    }

    pub fn from_i128(v: i128) -> Big {
        let mut m = v.unsigned_abs();
        let mut mag = Vec::new();
        while m != 0 {
            mag.push(m as u32);
            m >>= 32;
        }
        Big { neg: v < 0, mag }
    }

    /// 2^k.
    pub fn pow2(k: u32) -> Big {
        let mut mag = vec![0u32; (k / 32) as usize + 1];
        mag[(k / 32) as usize] = 1 << (k % 32);
        Big { neg: false, mag }
    }

    pub fn to_i128(&self) -> Option<i128> {
        if self.mag.len() > 4 {
            return None;
        }
        let mut m: u128 = 0;
        for (i, w) in self.mag.iter().enumerate() {
            m |= (*w as u128) << (32 * i);
        }
        if self.neg {
            if m <= i128::MAX as u128 + 1 {
                Some((m as i128).wrapping_neg())
            } else {
                None
            }
        } else if m <= i128::MAX as u128 {
            Some(m as i128)
        } else {
            None
        }
    }

    /// Decimal digits, with an optional leading `-`; `_` separators skipped.
    pub fn parse(s: &str) -> Option<Big> {
        let (neg, digits) = match s.strip_prefix('-') {
            Some(d) => (true, d),
            None => (false, s),
        };
        let mut out = Big::zero();
        let mut any = false;
        for c in digits.chars() {
            if c == '_' {
                continue;
            }
            let d = c.to_digit(10)?;
            out = out.mul_small(10).add_small(d);
            any = true;
        }
        if !any {
            return None;
        }
        out.neg = neg && !out.is_zero();
        Some(out)
    }

    pub fn is_zero(&self) -> bool {
        self.mag.is_empty()
    }

    pub fn is_neg(&self) -> bool {
        self.neg
    }

    /// How many bits the magnitude takes.
    pub fn bits(&self) -> u64 {
        match self.mag.last() {
            Some(top) => 32 * (self.mag.len() as u64 - 1) + (32 - top.leading_zeros()) as u64,
            None => 0,
        }
    }

    fn trim(mut self) -> Big {
        while self.mag.last() == Some(&0) {
            self.mag.pop();
        }
        if self.mag.is_empty() {
            self.neg = false;
        }
        self
    }

    fn mul_small(&self, k: u32) -> Big {
        let mut carry: u64 = 0;
        let mut mag = Vec::with_capacity(self.mag.len() + 1);
        for w in &self.mag {
            let x = *w as u64 * k as u64 + carry;
            mag.push(x as u32);
            carry = x >> 32;
        }
        if carry != 0 {
            mag.push(carry as u32);
        }
        Big { neg: self.neg, mag }.trim()
    }

    fn add_small(&self, k: u32) -> Big {
        self.add(&Big { neg: false, mag: vec![k] }.trim())
    }

    fn cmp_mag(a: &[u32], b: &[u32]) -> Ordering {
        if a.len() != b.len() {
            return a.len().cmp(&b.len());
        }
        for i in (0..a.len()).rev() {
            match a[i].cmp(&b[i]) {
                Ordering::Equal => {}
                o => return o,
            }
        }
        Ordering::Equal
    }

    fn add_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len().max(b.len()) + 1);
        let mut carry = 0u64;
        for i in 0..a.len().max(b.len()) {
            let x = *a.get(i).unwrap_or(&0) as u64 + *b.get(i).unwrap_or(&0) as u64 + carry;
            out.push(x as u32);
            carry = x >> 32;
        }
        if carry != 0 {
            out.push(carry as u32);
        }
        out
    }

    /// a - b, |a| >= |b|.
    fn sub_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len());
        let mut borrow = 0i64;
        for i in 0..a.len() {
            let mut x = a[i] as i64 - *b.get(i).unwrap_or(&0) as i64 - borrow;
            borrow = 0;
            if x < 0 {
                x += 1 << 32;
                borrow = 1;
            }
            out.push(x as u32);
        }
        out
    }

    pub fn neg(&self) -> Big {
        Big { neg: !self.neg, mag: self.mag.clone() }.trim()
    }

    pub fn add(&self, o: &Big) -> Big {
        if self.neg == o.neg {
            return Big { neg: self.neg, mag: Big::add_mag(&self.mag, &o.mag) }.trim();
        }
        match Big::cmp_mag(&self.mag, &o.mag) {
            Ordering::Less => Big { neg: o.neg, mag: Big::sub_mag(&o.mag, &self.mag) }.trim(),
            _ => Big { neg: self.neg, mag: Big::sub_mag(&self.mag, &o.mag) }.trim(),
        }
    }

    pub fn sub(&self, o: &Big) -> Big {
        self.add(&o.neg())
    }

    pub fn mul(&self, o: &Big) -> Big {
        if self.is_zero() || o.is_zero() {
            return Big::zero();
        }
        let mut out = vec![0u32; self.mag.len() + o.mag.len()];
        for (i, a) in self.mag.iter().enumerate() {
            let mut carry = 0u64;
            for (j, b) in o.mag.iter().enumerate() {
                let x = out[i + j] as u64 + *a as u64 * *b as u64 + carry;
                out[i + j] = x as u32;
                carry = x >> 32;
            }
            let mut k = i + o.mag.len();
            while carry != 0 {
                let x = out[k] as u64 + carry;
                out[k] = x as u32;
                carry = x >> 32;
                k += 1;
            }
        }
        Big { neg: self.neg != o.neg, mag: out }.trim()
    }

    /// The quotient toward zero and the remainder (the dividend's sign);
    /// `None` for a zero divisor. Long division a bit at a time.
    pub fn divrem_trunc(&self, o: &Big) -> Option<(Big, Big)> {
        if o.is_zero() {
            return None;
        }
        if Big::cmp_mag(&self.mag, &o.mag) == Ordering::Less {
            return Some((Big::zero(), self.clone()));
        }
        let mut q = vec![0u32; self.mag.len()];
        let mut r = Big::zero();
        let d = Big { neg: false, mag: o.mag.clone() };
        for i in (0..self.bits()).rev() {
            r = r.mul_small(2);
            if (self.mag[(i / 32) as usize] >> (i % 32)) & 1 == 1 {
                r = r.add_small(1);
            }
            if Big::cmp_mag(&r.mag, &d.mag) != Ordering::Less {
                r = Big { neg: false, mag: Big::sub_mag(&r.mag, &d.mag) }.trim();
                q[(i / 32) as usize] |= 1 << (i % 32);
            }
        }
        let q = Big { neg: self.neg != o.neg, mag: q }.trim();
        let r = Big { neg: self.neg, mag: r.mag }.trim();
        Some((q, r))
    }

    /// `div.X` and `mod.X` (owner, 2026-10-02), as `rounding::int_divmod`.
    pub fn divmod(&self, o: &Big, r: crate::ast::Round) -> Option<(Big, Big)> {
        use crate::ast::Round;
        let (q, m) = self.divrem_trunc(o)?;
        if m.is_zero() {
            return Some((q, m));
        }
        let up = m.neg == o.neg;
        let step = match r {
            Round::Trunc => false,
            Round::Floor | Round::Bare => !up,
            Round::Ceil => up,
            Round::Near => {
                let twice = Big { neg: false, mag: m.mag.clone() }.mul_small(2);
                match Big::cmp_mag(&twice.mag, &o.mag) {
                    Ordering::Greater => true,
                    Ordering::Equal => q.mag.first().is_some_and(|w| w & 1 == 1),
                    Ordering::Less => false,
                }
            }
        };
        if !step {
            return Some((q, m));
        }
        let one = Big::from_i128(1);
        Some(if up { (q.add(&one), m.sub(o)) } else { (q.sub(&one), m.add(o)) })
    }

    /// self^e; `None` past every width there is.
    pub fn pow(&self, e: u64) -> Option<Big> {
        if self.bits().saturating_mul(e) > CAP_BITS {
            return None;
        }
        let mut acc = Big::from_i128(1);
        let mut base = self.clone();
        let mut e = e;
        while e > 0 {
            if e & 1 == 1 {
                acc = acc.mul(&base);
            }
            e >>= 1;
            if e > 0 {
                base = base.mul(&base);
            }
        }
        Some(acc)
    }

    /// Too far past every width there is to be worth carrying.
    pub fn huge(&self) -> bool {
        self.bits() > CAP_BITS
    }
}

impl PartialOrd for Big {
    fn partial_cmp(&self, o: &Big) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl Ord for Big {
    fn cmp(&self, o: &Big) -> Ordering {
        match (self.neg, o.neg) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (false, false) => Big::cmp_mag(&self.mag, &o.mag),
            (true, true) => Big::cmp_mag(&o.mag, &self.mag),
        }
    }
}

impl std::fmt::Display for Big {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_zero() {
            return write!(f, "0");
        }
        // Nine digits at a time.
        let mut chunks = Vec::new();
        let mut m = Big { neg: false, mag: self.mag.clone() };
        let billion = Big::from_i128(1_000_000_000);
        while !m.is_zero() {
            let (q, r) = m.divrem_trunc(&billion).unwrap();
            chunks.push(r.to_i128().unwrap() as u32);
            m = q;
        }
        if self.neg {
            write!(f, "-")?;
        }
        write!(f, "{}", chunks.last().unwrap())?;
        for c in chunks.iter().rev().skip(1) {
            write!(f, "{:09}", c)?;
        }
        Ok(())
    }
}

/// Where a whole number rounds past the largest of a bin16, bin32, bin64
/// or brain16: the largest plus half a step, the tie going to infinity.
/// A value fits when its size is below it.
pub fn bin_round_limit(bits: u16) -> Big {
    match bits {
        16 => Big::from_i128(65520),
        32 => Big::pow2(128).sub(&Big::pow2(103)),
        crate::types::BRAIN_BITS => Big::pow2(128).sub(&Big::pow2(119)),
        _ => Big::pow2(1024).sub(&Big::pow2(970)),
    }
}

/// What an integer type holds, at any width.
pub fn type_range(signed: bool, bits: u32) -> (Big, Big) {
    if signed && bits & crate::types::EXACT_FLAG != 0 {
        let limit = bits & !crate::types::EXACT_FLAG;
        return (Big::pow2(limit - 1).neg(), Big::pow2(limit).sub(&Big::from_i128(1)));
    }
    if signed {
        (Big::pow2(bits - 1).neg(), Big::pow2(bits - 1).sub(&Big::from_i128(1)))
    } else {
        (Big::zero(), Big::pow2(bits).sub(&Big::from_i128(1)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_as_python() {
        let max = Big::parse("115792089237316195423570985008687907853269984665640564039457584007913129639935").unwrap();
        assert_eq!(max, Big::pow2(256).sub(&Big::from_i128(1)));
        assert_eq!(max.divmod(&Big::from_i128(2), crate::ast::Round::Floor).unwrap().0.to_string(), "57896044618658097711785492504343953926634992332820282019728792003956564819967");
        let p = Big::pow2(200).neg();
        let (q, _) = p.divmod(&Big::from_i128(3), crate::ast::Round::Near).unwrap();
        assert_eq!(q.to_string(), "-535646014752996758513987364113720867507400997927597611767125");
        let (_, m) = p.divmod(&Big::from_i128(7), crate::ast::Round::Floor).unwrap();
        assert_eq!(m.to_string(), "3");
        assert_eq!(Big::from_i128(-7).to_i128(), Some(-7));
        assert_eq!(Big::from_i128(i128::MIN).to_i128(), Some(i128::MIN));
        assert_eq!(Big::from_i128(2).pow(255).unwrap(), Big::pow2(255));
        assert!(Big::from_i128(-3) < Big::from_i128(2));
    }
}
