//! A small signed big integer, for bare numbers alone, which are worked out
//! exactly however wide they get (the oracle; corpus
//! 2026-10-07-bare-pow-past-128-bits-beside-a-typed-operand): enough for
//! `+ - * pow` and the rounded divisions, nothing more.

use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Big {
    neg: bool,
    /// Little-endian 32-bit digits, no high zeros; zero is empty.
    mag: Vec<u32>,
}

fn trim(mut v: Vec<u32>) -> Vec<u32> {
    while v.last() == Some(&0) {
        v.pop();
    }
    v
}

fn cmp_mag(a: &[u32], b: &[u32]) -> Ordering {
    a.len().cmp(&b.len()).then_with(|| a.iter().rev().cmp(b.iter().rev()))
}

fn add_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(a.len().max(b.len()) + 1);
    let mut carry = 0u64;
    for i in 0..a.len().max(b.len()) {
        let s = *a.get(i).unwrap_or(&0) as u64 + *b.get(i).unwrap_or(&0) as u64 + carry;
        out.push(s as u32);
        carry = s >> 32;
    }
    if carry > 0 {
        out.push(carry as u32);
    }
    trim(out)
}

/// `a - b` for `a >= b`.
fn sub_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(a.len());
    let mut borrow = 0i64;
    for i in 0..a.len() {
        let mut d = a[i] as i64 - *b.get(i).unwrap_or(&0) as i64 - borrow;
        borrow = if d < 0 { d += 1 << 32; 1 } else { 0 };
        out.push(d as u32);
    }
    trim(out)
}

fn mul_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut out = vec![0u32; a.len() + b.len()];
    for (i, &x) in a.iter().enumerate() {
        let mut carry = 0u64;
        for (j, &y) in b.iter().enumerate() {
            let t = out[i + j] as u64 + x as u64 * y as u64 + carry;
            out[i + j] = t as u32;
            carry = t >> 32;
        }
        let mut k = i + b.len();
        while carry > 0 {
            let t = out[k] as u64 + carry;
            out[k] = t as u32;
            carry = t >> 32;
            k += 1;
        }
    }
    trim(out)
}

fn bits(a: &[u32]) -> usize {
    a.last().map_or(0, |h| (a.len() - 1) * 32 + (32 - h.leading_zeros() as usize))
}

fn bit(a: &[u32], i: usize) -> bool {
    a.get(i / 32).map_or(false, |d| d >> (i % 32) & 1 == 1)
}

/// Truncated magnitude division, bit by bit (the numbers are a few hundred
/// bits at most). `b` is not zero.
fn divrem_mag(a: &[u32], b: &[u32]) -> (Vec<u32>, Vec<u32>) {
    let mut q = vec![0u32; a.len()];
    let mut r: Vec<u32> = vec![];
    for i in (0..bits(a)).rev() {
        // r = r * 2 + bit i of a
        r = add_mag(&r, &r);
        if bit(a, i) {
            r = add_mag(&r, &[1]);
        }
        if cmp_mag(&r, b) != Ordering::Less {
            r = sub_mag(&r, b);
            q[i / 32] |= 1 << (i % 32);
        }
    }
    (trim(q), r)
}

impl Big {
    fn new(neg: bool, mag: Vec<u32>) -> Big {
        let mag = trim(mag);
        Big { neg: neg && !mag.is_empty(), mag }
    }

    pub fn from_i128(v: i128) -> Big {
        let m = v.unsigned_abs();
        Big::new(v < 0, vec![m as u32, (m >> 32) as u32, (m >> 64) as u32, (m >> 96) as u32])
    }

    pub fn to_i128(&self) -> Option<i128> {
        if self.mag.len() > 4 {
            return None;
        }
        let m = self.mag.iter().enumerate().fold(0u128, |acc, (i, d)| acc | (*d as u128) << (32 * i));
        if self.neg {
            if m <= i128::MAX as u128 + 1 { Some((m as i128).wrapping_neg()) } else { None }
        } else {
            i128::try_from(m).ok()
        }
    }

    pub fn is_neg(&self) -> bool {
        self.neg
    }

    pub fn is_zero(&self) -> bool {
        self.mag.is_empty()
    }

    pub fn neg(&self) -> Big {
        Big::new(!self.neg, self.mag.clone())
    }

    pub fn add(&self, o: &Big) -> Big {
        if self.neg == o.neg {
            return Big::new(self.neg, add_mag(&self.mag, &o.mag));
        }
        match cmp_mag(&self.mag, &o.mag) {
            Ordering::Less => Big::new(o.neg, sub_mag(&o.mag, &self.mag)),
            _ => Big::new(self.neg, sub_mag(&self.mag, &o.mag)),
        }
    }

    pub fn sub(&self, o: &Big) -> Big {
        self.add(&o.neg())
    }

    pub fn mul(&self, o: &Big) -> Big {
        Big::new(self.neg != o.neg, mul_mag(&self.mag, &o.mag))
    }

    pub fn pow(&self, mut e: u32) -> Big {
        let (mut acc, mut base) = (Big::from_i128(1), self.clone());
        while e > 0 {
            if e & 1 == 1 {
                acc = acc.mul(&base);
            }
            base = base.mul(&base);
            e >>= 1;
        }
        acc
    }

    /// The quotient rounded as `op` says and what it leaves, as
    /// `BinOp::quotient` / `divide` do for i128. None for a zero divisor.
    pub fn divide(&self, o: &Big, op: crate::ast::BinOp) -> Option<Big> {
        use crate::ast::BinOp::*;
        if o.is_zero() {
            return None;
        }
        let (qm, rm) = divrem_mag(&self.mag, &o.mag);
        let neg = self.neg != o.neg;
        let q = Big::new(neg, qm);
        let r = Big::new(self.neg, rm);
        let away = || if neg { q.sub(&Big::from_i128(1)) } else { q.add(&Big::from_i128(1)) };
        let q = if r.is_zero() {
            q
        } else {
            match op {
                DivTrunc | ModTrunc => q,
                Div | Mod => if neg { away() } else { q },
                DivCeil | ModCeil => if neg { q } else { away() },
                _ => {
                    let twice = add_mag(&r.mag, &r.mag);
                    let odd = q.mag.first().map_or(false, |d| d & 1 == 1);
                    match cmp_mag(&twice, &o.mag) {
                        Ordering::Greater => away(),
                        Ordering::Equal if odd => away(),
                        _ => q,
                    }
                }
            }
        };
        Some(if op.division()? { self.sub(&q.mul(o)) } else { q })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::BinOp;

    #[test]
    fn agrees_with_i128_where_it_fits() {
        let vals = [-1000i128, -77, -10, -9, -7, -3, -2, -1, 0, 1, 2, 3, 7, 9, 10, 77, 1000, i64::MAX as i128, i64::MIN as i128];
        let ops = [BinOp::Div, BinOp::Mod, BinOp::DivTrunc, BinOp::ModTrunc, BinOp::DivCeil, BinOp::ModCeil, BinOp::DivNear, BinOp::ModNear];
        for &x in &vals {
            for &y in &vals {
                let (bx, by) = (Big::from_i128(x), Big::from_i128(y));
                assert_eq!(bx.add(&by).to_i128(), x.checked_add(y));
                assert_eq!(bx.sub(&by).to_i128(), x.checked_sub(y));
                assert_eq!(bx.mul(&by).to_i128(), x.checked_mul(y));
                for op in ops {
                    assert_eq!(bx.divide(&by, op).and_then(|v| v.to_i128()), op.divide(x, y), "{x} {op:?} {y}");
                }
            }
        }
        assert_eq!(Big::from_i128(i128::MIN).to_i128(), Some(i128::MIN));
        assert_eq!(Big::from_i128(-3).pow(3).to_i128(), Some(-27));
    }

    #[test]
    fn past_128_bits() {
        // 3744743927^4 mod.floor 4294967278 (seed 122, 2026-10-15)
        let p = Big::from_i128(3744743927).pow(4);
        assert!(p.to_i128().is_none());
        let m = p.divide(&Big::from_i128(4294967278), BinOp::Mod).unwrap().to_i128().unwrap();
        let mut acc: i128 = 1;
        for _ in 0..4 {
            acc = acc * 3744743927 % 4294967278;
        }
        assert_eq!(m, acc);
    }
}
