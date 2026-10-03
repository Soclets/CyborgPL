//! Whole numbers wider than 128 bits, for `!std_int256` and up (owner, 2026-10-03: "Do it now",
//! written here rather than borrowed). Sign and magnitude, the magnitude in 32-bit limbs, least
//! significant first, never with a leading zero limb; zero is never negative. Only what the
//! language's integer operations need, and written for being obviously right, not for speed.

use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BigInt { neg: bool, mag: Vec<u32> }

fn trim(mut m: Vec<u32>) -> Vec<u32> { while m.last() == Some(&0) { m.pop(); } m }

fn cmp_mag(a: &[u32], b: &[u32]) -> Ordering {
    if a.len() != b.len() { return a.len().cmp(&b.len()); }
    for i in (0..a.len()).rev() { if a[i] != b[i] { return a[i].cmp(&b[i]); } }
    Ordering::Equal
}

fn add_mag(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut out = Vec::with_capacity(a.len().max(b.len()) + 1);
    let mut carry = 0u64;
    for i in 0..a.len().max(b.len()) {
        let s = *a.get(i).unwrap_or(&0) as u64 + *b.get(i).unwrap_or(&0) as u64 + carry;
        out.push(s as u32);
        carry = s >> 32;
    }
    if carry > 0 { out.push(carry as u32); }
    trim(out)
}

/// a - b, with a >= b
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
    if a.is_empty() || b.is_empty() { return Vec::new(); }
    let mut out = vec![0u32; a.len() + b.len()];
    for (i, &x) in a.iter().enumerate() {
        let mut carry = 0u64;
        for (j, &y) in b.iter().enumerate() {
            let t = out[i + j] as u64 + x as u64 * y as u64 + carry;
            out[i + j] = t as u32;
            carry = t >> 32;
        }
        let mut k = i + b.len();
        while carry > 0 { let t = out[k] as u64 + carry; out[k] = t as u32; carry = t >> 32; k += 1; }
    }
    trim(out)
}

fn bits_mag(a: &[u32]) -> u64 { match a.last() { None => 0, Some(&top) => (a.len() as u64 - 1) * 32 + (32 - top.leading_zeros() as u64) } }

fn bit_mag(a: &[u32], i: u64) -> bool { a.get((i / 32) as usize).map_or(false, |l| (l >> (i % 32)) & 1 == 1) }

/// (a div b, a mod b) on magnitudes, b nonzero: shift and subtract, one bit at a time
fn divrem_mag(a: &[u32], b: &[u32]) -> (Vec<u32>, Vec<u32>) {
    if cmp_mag(a, b) == Ordering::Less { return (Vec::new(), a.to_vec()); }
    let n = bits_mag(a);
    let mut q = vec![0u32; a.len()];
    let mut r: Vec<u32> = Vec::new();
    for i in (0..n).rev() {
        r = shl_mag(&r, 1);
        if bit_mag(a, i) { if r.is_empty() { r.push(1); } else { r[0] |= 1; } }
        if cmp_mag(&r, b) != Ordering::Less { r = sub_mag(&r, b); q[(i / 32) as usize] |= 1 << (i % 32); }
    }
    (trim(q), r)
}

fn shl_mag(a: &[u32], n: u64) -> Vec<u32> {
    if a.is_empty() { return Vec::new(); }
    let (limbs, bits) = ((n / 32) as usize, (n % 32) as u32);
    let mut out = vec![0u32; limbs];
    let mut carry = 0u32;
    for &x in a {
        out.push(if bits == 0 { x } else { (x << bits) | carry });
        carry = if bits == 0 { 0 } else { x >> (32 - bits) };
    }
    if carry > 0 { out.push(carry); }
    trim(out)
}

impl BigInt {
    pub fn zero() -> BigInt { BigInt { neg: false, mag: Vec::new() } }
    fn new(neg: bool, mag: Vec<u32>) -> BigInt { let mag = trim(mag); let neg = neg && !mag.is_empty(); BigInt { neg, mag } }
    pub fn from_i128(v: i128) -> BigInt {
        let mut m = v.unsigned_abs();
        let mut mag = Vec::new();
        while m > 0 { mag.push(m as u32); m >>= 32; }
        BigInt::new(v < 0, mag)
    }
    /// 2^n
    pub fn pow2(n: u64) -> BigInt { BigInt::new(false, shl_mag(&[1], n)) }
    pub fn to_i128(&self) -> Option<i128> {
        if self.mag.len() > 4 { return None; }
        let mut m: u128 = 0;
        for (i, &l) in self.mag.iter().enumerate() { m |= (l as u128) << (32 * i); }
        if self.neg { if m <= 1u128 << 127 { Some((m as i128).wrapping_neg()) } else { None } } else if m < 1u128 << 127 { Some(m as i128) } else { None }
    }
    pub fn to_u64(&self) -> Option<u64> { if self.neg || self.mag.len() > 2 { None } else { Some(self.mag.iter().enumerate().fold(0u64, |acc, (i, &l)| acc | (l as u64) << (32 * i))) } }
    pub fn is_zero(&self) -> bool { self.mag.is_empty() }
    pub fn is_neg(&self) -> bool { self.neg }
    /// how many bits the magnitude takes
    pub fn bits(&self) -> u64 { bits_mag(&self.mag) }
    pub fn neg(&self) -> BigInt { BigInt::new(!self.neg, self.mag.clone()) }
    pub fn add(&self, b: &BigInt) -> BigInt {
        if self.neg == b.neg { return BigInt::new(self.neg, add_mag(&self.mag, &b.mag)); }
        match cmp_mag(&self.mag, &b.mag) {
            Ordering::Less => BigInt::new(b.neg, sub_mag(&b.mag, &self.mag)),
            _ => BigInt::new(self.neg, sub_mag(&self.mag, &b.mag)),
        }
    }
    pub fn sub(&self, b: &BigInt) -> BigInt { self.add(&b.neg()) }
    pub fn mul(&self, b: &BigInt) -> BigInt { BigInt::new(self.neg != b.neg, mul_mag(&self.mag, &b.mag)) }
    /// truncated division, as C's `/` and `%`: None for a zero divisor
    pub fn divrem_trunc(&self, b: &BigInt) -> Option<(BigInt, BigInt)> {
        if b.is_zero() { return None; }
        let (q, r) = divrem_mag(&self.mag, &b.mag);
        Some((BigInt::new(self.neg != b.neg, q), BigInt::new(self.neg, r)))
    }
    /// self to a whole, non-negative power
    pub fn pow(&self, mut e: u64) -> BigInt {
        let (mut base, mut acc) = (self.clone(), BigInt::from_i128(1));
        while e > 0 { if e & 1 == 1 { acc = acc.mul(&base); } e >>= 1; if e > 0 { base = base.mul(&base); } }
        acc
    }
    /// self modulo 2^n, as a non-negative number below 2^n
    pub fn mod_pow2(&self, n: u64) -> BigInt {
        let limbs = (n / 32) as usize;
        let mut low: Vec<u32> = self.mag.iter().take(limbs + 1).cloned().collect();
        if low.len() > limbs { if n % 32 == 0 { low.truncate(limbs); } else { low[limbs] &= (1u32 << (n % 32)) - 1; } }
        let low = trim(low);
        if !self.neg || low.is_empty() { BigInt::new(false, low) } else { BigInt::new(false, sub_mag(&shl_mag(&[1], n), &low)) }
    }
    /// `and.bit` / `or.bit` / `xor` (b'&', b'|', b'^'), on two's complement with the sign extended
    /// for ever, as i128's own operators do
    pub fn bitop(&self, b: &BigInt, op: u8) -> BigInt {
        let n = self.bits().max(b.bits()) + 2;
        let (x, y) = (self.mod_pow2(n), b.mod_pow2(n));
        let len = x.mag.len().max(y.mag.len());
        let mut out = Vec::with_capacity(len);
        for i in 0..len {
            let (p, q) = (*x.mag.get(i).unwrap_or(&0), *y.mag.get(i).unwrap_or(&0));
            out.push(match op { b'&' => p & q, b'|' => p | q, _ => p ^ q });
        }
        let r = BigInt::new(false, out).mod_pow2(n);
        if bit_mag(&r.mag, n - 1) { r.sub(&BigInt::pow2(n)) } else { r }
    }
    /// self x 2^n
    pub fn shl(&self, n: u64) -> BigInt { BigInt::new(self.neg, shl_mag(&self.mag, n)) }
    /// self / 2^n rounded down, as an arithmetic shift right
    pub fn shr_floor(&self, n: u64) -> BigInt {
        let (q, r) = self.divrem_trunc(&BigInt::pow2(n)).unwrap();
        if self.neg && !r.is_zero() { q.sub(&BigInt::from_i128(1)) } else { q }
    }
    pub fn cmp(&self, b: &BigInt) -> Ordering {
        match (self.neg, b.neg) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (false, false) => cmp_mag(&self.mag, &b.mag),
            (true, true) => cmp_mag(&b.mag, &self.mag),
        }
    }
    pub fn parse(text: &str) -> Option<BigInt> {
        let (neg, digits) = match text.strip_prefix('-') { Some(d) => (true, d), None => (false, text) };
        if digits.is_empty() { return None; }
        let mut mag: Vec<u32> = Vec::new();
        for c in digits.chars() {
            if c == '_' { continue; }
            let d = c.to_digit(10)?;
            mag = add_mag(&mul_mag(&mag, &[10]), &if d == 0 { Vec::new() } else { vec![d] });
        }
        Some(BigInt::new(neg, mag))
    }
}

impl std::fmt::Display for BigInt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.mag.is_empty() { return write!(f, "0"); }
        // nine decimal digits at a time
        let mut parts = Vec::new();
        let mut m = self.mag.clone();
        let billion = [1_000_000_000u32];
        while !m.is_empty() { let (q, r) = divrem_mag(&m, &billion); parts.push(r.first().copied().unwrap_or(0)); m = q; }
        let mut s = String::new();
        if self.neg { s.push('-'); }
        s.push_str(&parts.last().unwrap().to_string());
        for p in parts.iter().rev().skip(1) { s.push_str(&format!("{p:09}")); }
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::BigInt;
    #[test]
    fn against_i128() {
        let vals: [i128; 9] = [0, 1, -1, 7, -7, 1 << 40, -(1 << 63), i64::MAX as i128, 123_456_789_012_345_678_901];
        for &a in &vals { for &b in &vals {
            let (x, y) = (BigInt::from_i128(a), BigInt::from_i128(b));
            assert_eq!(x.add(&y).to_i128(), a.checked_add(b));
            assert_eq!(x.sub(&y).to_i128(), a.checked_sub(b));
            if let Some(p) = a.checked_mul(b) { assert_eq!(x.mul(&y).to_i128(), Some(p)); }
            if b != 0 { let (q, r) = x.divrem_trunc(&y).unwrap(); assert_eq!((q.to_i128(), r.to_i128()), (Some(a / b), Some(a % b))); }
            assert_eq!(x.cmp(&y), a.cmp(&b));
        }
        assert_eq!(BigInt::from_i128(a).to_string(), a.to_string());
        assert_eq!(BigInt::parse(&a.to_string()), Some(BigInt::from_i128(a)));
        assert_eq!(BigInt::from_i128(a).mod_pow2(16).to_i128(), Some(a.rem_euclid(1 << 16)));
        for &b in &vals {
            let (x, y) = (BigInt::from_i128(a), BigInt::from_i128(b));
            assert_eq!(x.bitop(&y, b'&').to_i128(), Some(a & b));
            assert_eq!(x.bitop(&y, b'|').to_i128(), Some(a | b));
            assert_eq!(x.bitop(&y, b'^').to_i128(), Some(a ^ b));
        }
        for s in [0u32, 1, 5, 40] { assert_eq!(BigInt::from_i128(a).shr_floor(s as u64).to_i128(), Some(a >> s)); }
        }
        assert_eq!(BigInt::pow2(256).sub(&BigInt::from_i128(1)).to_string(), "115792089237316195423570985008687907853269984665640564039457584007913129639935");
        assert_eq!(BigInt::from_i128(3).pow(100).to_string(), "515377520732011331036461129765621272702107522001");
        assert_eq!(BigInt::from_i128(i128::MIN).to_i128(), Some(i128::MIN));
        assert_eq!(BigInt::pow2(127).to_i128(), None);
    }
}
