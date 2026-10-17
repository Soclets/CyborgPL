//! `deci32` / `deci64` / `deci128` — IEEE 754 decimal (design/types.md, owner 2026-09-29).
//! A value is a sign, a coefficient of at most 7 / 16 / 34 digits, and an exponent, so
//! `1.50` (150 × 10⁻²) and `1.5` (15 × 10⁻¹) are equal but kept apart, as IEEE keeps them.
//! Every operation is exact and then rounded to the width, half to even. Digits are held as
//! decimal digits (most significant first): slow, and plainly right, which is the oracle's job.

use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub struct Deci {
    pub neg: bool,
    /// most significant first, no leading zeros; zero is `[0]`
    pub coef: Vec<u8>,
    pub exp: i32,
    /// 32, 64 or 128
    pub width: u8,
}

pub fn precision(width: u8) -> usize { match width { 32 => 7, 64 => 16, _ => 34 } }
fn emax(width: u8) -> i32 { match width { 32 => 96, 64 => 384, _ => 6144 } }
/// the largest and smallest exponent a coefficient of `precision` digits may carry
fn qmax(width: u8) -> i32 { emax(width) - precision(width) as i32 + 1 }
fn qmin(width: u8) -> i32 { 1 - emax(width) - precision(width) as i32 + 1 }

fn trim(mut d: Vec<u8>) -> Vec<u8> {
    let nz = d.iter().position(|&x| x != 0).unwrap_or(d.len().saturating_sub(1));
    d.drain(..nz);
    if d.is_empty() { d.push(0); }
    d
}
fn is_zero(d: &[u8]) -> bool { d.iter().all(|&x| x == 0) }
fn cmp_mag(a: &[u8], b: &[u8]) -> Ordering { a.len().cmp(&b.len()).then_with(|| a.cmp(b)) }
fn add_mag(a: &[u8], b: &[u8]) -> Vec<u8> {
    let (mut out, mut carry) = (Vec::new(), 0u8);
    let (mut i, mut j) = (a.len(), b.len());
    while i > 0 || j > 0 || carry > 0 {
        let x = if i > 0 { i -= 1; a[i] } else { 0 };
        let y = if j > 0 { j -= 1; b[j] } else { 0 };
        let s = x + y + carry;
        out.push(s % 10); carry = s / 10;
    }
    out.reverse(); trim(out)
}
/// a − b, with a ≥ b
fn sub_mag(a: &[u8], b: &[u8]) -> Vec<u8> {
    let (mut out, mut borrow) = (Vec::new(), 0i8);
    let (mut i, mut j) = (a.len(), b.len());
    while i > 0 {
        i -= 1;
        let y = if j > 0 { j -= 1; b[j] as i8 } else { 0 };
        let mut d = a[i] as i8 - y - borrow;
        borrow = if d < 0 { d += 10; 1 } else { 0 };
        out.push(d as u8);
    }
    out.reverse(); trim(out)
}
fn mul_mag(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut acc = vec![0u32; a.len() + b.len()];
    for (i, &x) in a.iter().enumerate().rev() {
        for (j, &y) in b.iter().enumerate().rev() { acc[i + j + 1] += x as u32 * y as u32; }
    }
    for k in (1..acc.len()).rev() { let c = acc[k] / 10; acc[k] %= 10; acc[k - 1] += c; }
    trim(acc.into_iter().map(|x| x as u8).collect())
}
/// (a div b, a mod b) by long division
fn divmod_mag(a: &[u8], b: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let (mut q, mut r) = (Vec::new(), vec![0u8]);
    for &d in a {
        r.push(d); r = trim(r);
        let mut k = 0;
        while cmp_mag(&r, b) != Ordering::Less { r = sub_mag(&r, b); k += 1; }
        q.push(k);
    }
    (trim(q), r)
}
fn shift(d: &[u8], zeros: usize) -> Vec<u8> {
    if is_zero(d) { return vec![0]; }
    let mut v = d.to_vec(); v.extend(std::iter::repeat(0).take(zeros)); v
}

impl Deci {
    fn new(neg: bool, coef: Vec<u8>, exp: i32, width: u8) -> Deci { Deci { neg, coef: trim(coef), exp, width } }
    pub fn is_zero(&self) -> bool { is_zero(&self.coef) }

    /// Round to the width's digits, half to even; `sticky` says something non-zero was cut off
    /// beyond the digits given. Err when it is past the largest value.
    fn round(neg: bool, coef: Vec<u8>, exp: i32, width: u8, sticky: bool) -> Result<Deci, String> {
        let p = precision(width);
        let mut coef = trim(coef);
        let mut exp = exp;
        let mut drop = coef.len().saturating_sub(p);
        // below the smallest exponent, digits go (the subnormal range)
        if exp + (drop as i32) < qmin(width) { drop = (qmin(width) - exp) as usize; }
        if drop > 0 {
            let len = coef.len();
            // the digits cut off, padded with zeros in front when more go than there are
            let (kept, tail) = if drop >= len {
                let mut t = vec![0u8; drop - len]; t.extend(&coef); (vec![0u8], t)
            } else { (coef[..len - drop].to_vec(), coef[len - drop..].to_vec()) };
            let first = tail[0];
            let rest = tail.len() > 1 && !is_zero(&tail[1..]) || sticky;
            let odd = kept.last().map_or(false, |d| d % 2 == 1);
            let up = first > 5 || (first == 5 && (rest || odd));
            coef = if up { add_mag(&kept, &[1]) } else { trim(kept) };
            exp += drop as i32;
            if coef.len() > p { coef.pop(); exp += 1; }
        }
        if exp > qmax(width) {
            // fold down: pad with zeros while the digits allow, as IEEE does
            let room = p - coef.len();
            let need = (exp - qmax(width)) as usize;
            if is_zero(&coef) { exp = qmax(width); }
            else if need <= room { coef = shift(&coef, need); exp -= need as i32; }
            else { return Err(format!("does not fit deci{width}: past the largest value")); }
        }
        Ok(Deci::new(neg, coef, exp, width))
    }

    /// `0.1`, `1.50`, `-2.5e3`: the exact digits; Err(why) when they do not fit the width.
    pub fn parse(text: &str, width: u8) -> Result<Option<Deci>, String> {
        let (neg, t) = match text.strip_prefix('-') { Some(r) => (true, r), None => (false, text.strip_prefix('+').unwrap_or(text)) };
        let (mant, e) = match t.find(['e', 'E']) { Some(i) => (&t[..i], Some(&t[i + 1..])), None => (t, None) };
        let (int, frac) = match mant.split_once('.') { Some((a, b)) => (a, b), None => (mant, "") };
        let digits = |d: &str| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit());
        if !digits(int) || (mant.contains('.') && !digits(frac)) { return Ok(None); }
        let e: i64 = match e {
            None => 0,
            Some(e) => { let body = e.strip_prefix(['-', '+']).unwrap_or(e); if !digits(body) { return Ok(None); } e.parse().unwrap_or(if e.starts_with('-') { i64::MIN / 2 } else { i64::MAX / 2 }) }
        };
        let mut coef = trim(format!("{int}{frac}").bytes().map(|b| b - b'0').collect());
        let mut exp = e - frac.len() as i64;
        let p = precision(width);
        // trailing zeros past the width move into the exponent, the value unchanged: 9220000000000000000
        // is 922 × 10^16 (check claude, seed 25294: a brain16's useful form is such a number)
        while coef.len() > p && coef.last() == Some(&0) { coef.pop(); exp += 1; }
        if coef.len() > p { return Err(format!("has {} digits; deci{width} holds {p}", coef.len())); }
        if exp > qmax(width) as i64 + p as i64 || exp < qmin(width) as i64 { return Err(format!("is past what deci{width} holds")); }
        Deci::round(neg, coef, exp as i32, width, false).map(Some)
    }

    pub fn from_int(i: i128, width: u8) -> Result<Deci, String> {
        let coef: Vec<u8> = i.unsigned_abs().to_string().bytes().map(|b| b - b'0').collect();
        if coef.len() > precision(width) { return Err(format!("has {} digits; deci{width} holds {}", coef.len(), precision(width))); }
        Ok(Deci::new(i < 0, coef, 0, width))
    }

    /// `to.deci64['w']`: the bin's exact value, rounded to the width
    pub fn from_bin_exact(x: f64, width: u8) -> Result<Deci, String> {
        if x == 0.0 { return Ok(Deci::new(x.is_sign_negative(), vec![0], 0, width)); }
        let s = format!("{:.*e}", precision(width) - 1, x.abs());
        let (m, e) = s.split_once('e').unwrap();
        let coef: Vec<u8> = m.bytes().filter(u8::is_ascii_digit).map(|b| b - b'0').collect();
        let e: i32 = e.parse().unwrap();
        Deci::round(x < 0.0, coef, e - (precision(width) as i32 - 1), width, false)
    }
    /// `to.deci64.useful['w']`: the shortest decimal that reads back as the same bin
    pub fn from_bin_useful(x: f64, width: u8) -> Result<Deci, String> {
        let s = format!("{}", x.abs());
        let (int, frac) = s.split_once('.').unwrap_or((&s, ""));
        let coef: Vec<u8> = format!("{int}{frac}").bytes().map(|b| b - b'0').collect();
        Deci::round(x < 0.0 || (x == 0.0 && x.is_sign_negative()), coef, -(frac.len() as i32), width, false)
    }

    pub fn neg(&self) -> Deci { let mut d = self.clone(); d.neg = !d.neg; d }

    fn aligned(a: &Deci, b: &Deci) -> (Vec<u8>, Vec<u8>, i32) {
        let e = a.exp.min(b.exp);
        (shift(&a.coef, (a.exp - e) as usize), shift(&b.coef, (b.exp - e) as usize), e)
    }
    pub fn add(&self, b: &Deci) -> Result<Deci, String> {
        let (x, y, e) = Deci::aligned(self, b);
        let (neg, mag) = if self.neg == b.neg { (self.neg, add_mag(&x, &y)) }
            else { match cmp_mag(&x, &y) { Ordering::Less => (b.neg, sub_mag(&y, &x)), _ => (self.neg, sub_mag(&x, &y)) } };
        // a sum that is exactly zero is +0, as IEEE rounds half to even
        let neg = if is_zero(&mag) { self.neg && b.neg } else { neg };
        Deci::round(neg, mag, e, self.width, false)
    }
    pub fn sub(&self, b: &Deci) -> Result<Deci, String> { self.add(&b.neg()) }
    pub fn mul(&self, b: &Deci) -> Result<Deci, String> {
        Deci::round(self.neg != b.neg, mul_mag(&self.coef, &b.coef), self.exp + b.exp, self.width, false)
    }
    /// None: division by zero (a check site)
    pub fn div(&self, b: &Deci) -> Option<Result<Deci, String>> {
        if b.is_zero() { return None; }
        let neg = self.neg != b.neg;
        let ideal = self.exp - b.exp;
        if self.is_zero() { return Some(Deci::round(neg, vec![0], ideal, self.width, false)); }
        let p = precision(self.width);
        let sh = (p + 1 + b.coef.len()).saturating_sub(self.coef.len());
        let (mut q, r) = divmod_mag(&shift(&self.coef, sh), &b.coef);
        let mut exp = ideal - sh as i32;
        let exact = is_zero(&r);
        // an exact quotient keeps the exponent closest to the ideal, as IEEE says
        if exact { while exp < ideal && q.len() > 1 && *q.last().unwrap() == 0 { q.pop(); exp += 1; } }
        Some(Deci::round(neg, q, exp, self.width, !exact))
    }
    /// by value: `1.50 == 1.5`
    pub fn cmp_value(&self, b: &Deci) -> Ordering {
        let (az, bz) = (self.is_zero(), b.is_zero());
        if az && bz { return Ordering::Equal; }
        let sa = if az { 0 } else if self.neg { -1 } else { 1 };
        let sb = if bz { 0 } else if b.neg { -1 } else { 1 };
        if sa != sb { return sa.cmp(&sb); }
        let (x, y, _) = Deci::aligned(self, b);
        let m = cmp_mag(&x, &y);
        if sa < 0 { m.reverse() } else { m }
    }

    /// Stored form: every digit it holds, trailing zeros included (`1.50`).
    pub fn show_stored(&self) -> String {
        let digits: String = self.coef.iter().map(|d| (b'0' + d) as char).collect();
        let body = if self.exp >= 0 {
            if self.is_zero() { "0".to_string() } else { format!("{digits}{}", "0".repeat(self.exp as usize)) }
        } else {
            let places = (-self.exp) as usize;
            let padded = if digits.len() <= places { format!("{}{digits}", "0".repeat(places - digits.len() + 1)) } else { digits };
            let (a, b) = padded.split_at(padded.len() - places);
            format!("{a}.{b}")
        };
        if self.neg { format!("-{body}") } else { body }
    }
    /// Useful form: the same value, trailing zeros dropped (`1.5`, `5`).
    pub fn show_useful(&self) -> String {
        let mut d = self.clone();
        while d.exp < 0 && d.coef.len() > 1 && *d.coef.last().unwrap() == 0 { d.coef.pop(); d.exp += 1; }
        if d.is_zero() && d.exp < 0 { d.exp = 0; }
        d.show_stored()
    }
    /// A whole value as an integer, or None when it has a fraction (`not_whole`) — Some(Err) past i128.
    pub fn to_int(&self) -> Option<Result<i128, ()>> {
        let whole: Vec<u8> = if self.exp >= 0 { shift(&self.coef, self.exp as usize) } else {
            let places = (-self.exp) as usize;
            if places >= self.coef.len() { if self.is_zero() { vec![0] } else { return None; } }
            else { if !is_zero(&self.coef[self.coef.len() - places..]) { return None; } self.coef[..self.coef.len() - places].to_vec() }
        };
        let s: String = whole.iter().map(|d| (b'0' + d) as char).collect();
        Some(s.parse::<i128>().map(|v| if self.neg { -v } else { v }).map_err(|_| ()))
    }
    pub fn to_f64(&self) -> f64 { self.show_stored().parse().unwrap_or(f64::NAN) }
    /// a copy at another width: exact when it fits
    pub fn widen(&self, width: u8) -> Deci { let mut d = self.clone(); d.width = width; d }
}

impl Deci {
    /// `mod`: the divisor's sign, as integer `mod` (owner, 2026-09-29: a) — exact before it is rounded.
    /// None: `mod` by zero (a check site).
    pub fn rem_floor(&self, b: &Deci) -> Option<Result<Deci, String>> {
        if b.is_zero() { return None; }
        let (x, y, e) = Deci::aligned(self, b);
        let (_, r0) = divmod_mag(&x, &y);
        let (neg, mag) = if is_zero(&r0) { (b.neg, vec![0]) }
            else if self.neg == b.neg { (b.neg, r0) }
            else { (b.neg, sub_mag(&y, &r0)) };
        Some(Deci::round(neg, mag, e, self.width, false))
    }

    /// `div.X` / `mod.X` to a whole quotient (owner, 2026-10-02): the exact whole quotient rounded
    /// `floor`, `ceil`, `trunc` or `near` (half to even), and what is left over, so
    /// `a == q x b + r` exactly. None: division by zero.
    pub fn divmod_whole(&self, b: &Deci, mode: &str) -> Option<Result<(Deci, Deci), String>> {
        if b.is_zero() { return None; }
        let (x, y, e) = Deci::aligned(self, b);
        let (q0, r0) = divmod_mag(&x, &y);
        let qneg = self.neg != b.neg;
        let rest = !is_zero(&r0);
        let away = match mode {
            "floor" => rest && qneg,
            "ceil" => rest && !qneg,
            "near" => rest && match cmp_mag(&mul_mag(&r0, &[2]), &y) {
                Ordering::Greater => true,
                Ordering::Less => false,
                Ordering::Equal => q0.last().map_or(false, |d| d % 2 == 1),
            },
            // half away from zero (`round.half_up`, owner 2026-10-17)
            "half_up" => rest && cmp_mag(&mul_mag(&r0, &[2]), &y) != Ordering::Less,
            _ => false,
        };
        let (q, r, rneg) = if away { (add_mag(&q0, &[1]), sub_mag(&y, &r0), !self.neg) } else { (q0, r0, self.neg) };
        let rneg = if is_zero(&r) { mode == "floor" && b.neg } else { rneg };
        let qneg = qneg && !is_zero(&q);
        let qd = match Deci::round(qneg, q, 0, self.width, false) { Ok(d) => d, Err(m) => return Some(Err(m)) };
        let rd = match Deci::round(rneg, r, e, self.width, false) { Ok(d) => d, Err(m) => return Some(Err(m)) };
        Some(Ok((qd, rd)))
    }

    /// `xx` with a whole exponent (owner, 2026-09-29: a): the exact power, rounded once; a negative
    /// exponent is one over it. None: zero to a negative power (division by zero). Err(why) past the
    /// largest value, or past the oracle's limit on exact digits.
    pub fn pow_int(&self, n: i128) -> Option<Result<Deci, String>> {
        let k = n.unsigned_abs();
        if k as usize * self.coef.len() > 20_000 { return Some(Err("is a power past the oracle's limit of exact digits".into())); }
        let (mut coef, mut exp) = (vec![1u8], 0i64);
        let (mut base, mut bexp, mut m) = (self.coef.clone(), self.exp as i64, k);
        while m > 0 {
            if m & 1 == 1 { coef = mul_mag(&coef, &base); exp += bexp; }
            m >>= 1;
            if m > 0 { base = mul_mag(&base, &base); bexp *= 2; }
        }
        let neg = self.neg && k % 2 == 1;
        if exp > i32::MAX as i64 / 2 || exp < i32::MIN as i64 / 2 { return Some(Err(format!("does not fit deci{}: past the largest value", self.width))); }
        if n >= 0 { return Some(Deci::round(neg, coef, exp as i32, self.width, false)); }
        if self.is_zero() { return None; }
        // one over the exact power, rounded once
        let exact = Deci { neg, coef: trim(coef), exp: exp as i32, width: self.width };
        Deci { neg: false, coef: vec![1], exp: 0, width: self.width }.div(&exact)
    }

    /// `to.deci32[a deci64]`: exact when its digits fit the narrower width, else `too_many_digits`
    pub fn narrow(&self, width: u8) -> Result<Deci, &'static str> {
        if self.coef.len() > precision(width) { return Err("too_many_digits"); }
        Deci::round(self.neg, self.coef.clone(), self.exp, width, false).map_err(|_| "out_of_range")
    }
}
