//! `deci`: IEEE 754 decimal floating point (`types.md`, owner 2026-09-29).
//!
//! A `deci32`, `deci64` or `deci128` is held in the machine's own layout,
//! which on everything but IBM POWER and z is BID: the coefficient in
//! binary. There is no decimal hardware here, so every operation is done
//! in software, on the numbers the layout holds — a sign, a whole
//! coefficient and a power of ten — and rounded back to the width, half
//! to even. Each value crosses the C boundary as 128 bits in two halves,
//! whatever its width; the answer comes back through a pointer.
//!
//! What IEEE leaves to the implementation, and the oracle shows:
//! - an answer past the largest value stops the program (the compiler
//!   traps on status 1), as does a division by zero (status 2);
//! - a result below the smallest keeps rounding, down to zero;
//! - `print` shows every digit the coefficient holds, never an exponent.

use crate::{bin_grammar, expand, text_of, Text, STATIC};

/// One width's constants: digits held, the exponent range, the exponent
/// field's size.
#[derive(Clone, Copy)]
struct Width {
    bits: u32,
    digits: u32,
    qmin: i32,
    qmax: i32,
    ebits: u32,
}

fn width(bits: u32) -> Width {
    match bits {
        32 => Width { bits: 32, digits: 7, qmin: -101, qmax: 90, ebits: 8 },
        64 => Width { bits: 64, digits: 16, qmin: -398, qmax: 369, ebits: 10 },
        _ => Width { bits: 128, digits: 34, qmin: -6176, qmax: 6111, ebits: 14 },
    }
}

/// A finite decimal: (-1)^neg x coef x 10^exp.
#[derive(Clone, Copy)]
struct Dec {
    neg: bool,
    coef: u128,
    exp: i32,
}

fn mask(n: u32) -> u128 {
    if n >= 128 {
        u128::MAX
    } else {
        (1u128 << n) - 1
    }
}

fn pow10(n: u32) -> u128 {
    10u128.pow(n)
}

fn decode(x: u128, w: Width) -> Dec {
    let x = x & mask(w.bits);
    let neg = (x >> (w.bits - 1)) & 1 == 1;
    let top2 = (x >> (w.bits - 3)) & 3;
    let (e, coef) = if top2 == 3 {
        // The large form: the coefficient's top bits are an implied 100.
        let low = w.bits - 3 - w.ebits;
        let e = ((x >> low) & mask(w.ebits)) as i32;
        (e, (1u128 << (w.bits - 1 - w.ebits)) | (x & mask(low)))
    } else {
        let low = w.bits - 1 - w.ebits;
        (((x >> low) & mask(w.ebits)) as i32, x & mask(low))
    };
    // A coefficient past the width's digits is not canonical: IEEE reads
    // it as zero. (An infinity or NaN handed in from C lands here too.)
    let coef = if coef >= pow10(w.digits) { 0 } else { coef };
    Dec { neg, coef, exp: e + w.qmin }
}

fn encode(d: Dec, w: Width) -> u128 {
    let e = (d.exp - w.qmin) as u128;
    let sign = (d.neg as u128) << (w.bits - 1);
    let small = w.bits - 1 - w.ebits;
    if d.coef < (1u128 << small) {
        sign | (e << small) | d.coef
    } else {
        let low = w.bits - 3 - w.ebits;
        sign | (3u128 << (w.bits - 3)) | (e << low) | (d.coef & mask(low))
    }
}

// -------------------------------------------------------- wide numbers

/// 256 bits, least significant limb first: a product of two 34-digit
/// coefficients, or a sum lined up past them.
#[derive(Clone, Copy, PartialEq, Eq)]
struct U256([u64; 4]);

impl U256 {
    fn of(v: u128) -> U256 {
        U256([v as u64, (v >> 64) as u64, 0, 0])
    }

    fn is_zero(&self) -> bool {
        self.0 == [0; 4]
    }

    fn fits128(&self) -> bool {
        self.0[2] == 0 && self.0[3] == 0
    }

    fn low128(&self) -> u128 {
        self.0[0] as u128 | (self.0[1] as u128) << 64
    }

    fn mul_small(&mut self, k: u64) {
        let mut carry: u128 = 0;
        for limb in self.0.iter_mut() {
            let x = *limb as u128 * k as u128 + carry;
            *limb = x as u64;
            carry = x >> 64;
        }
    }

    fn times_pow10(&mut self, mut n: u32) {
        while n > 0 {
            let s = n.min(19);
            self.mul_small(10u64.pow(s));
            n -= s;
        }
    }

    /// Divides by `k` in place; the remainder.
    fn div_small(&mut self, k: u64) -> u64 {
        let mut rem: u128 = 0;
        for limb in self.0.iter_mut().rev() {
            let cur = rem << 64 | *limb as u128;
            *limb = (cur / k as u128) as u64;
            rem = cur % k as u128;
        }
        rem as u64
    }

    fn mul(a: u128, b: u128) -> U256 {
        let (a0, a1) = (a as u64 as u128, (a >> 64) as u64 as u128);
        let (b0, b1) = (b as u64 as u128, (b >> 64) as u64 as u128);
        let mut out = [0u64; 4];
        let mut add = |at: usize, v: u128| {
            let mut carry = v;
            let mut i = at;
            while carry != 0 && i < 4 {
                let x = out[i] as u128 + (carry as u64) as u128;
                out[i] = x as u64;
                carry = (carry >> 64) + (x >> 64);
                i += 1;
            }
        };
        add(0, a0 * b0);
        add(1, a0 * b1);
        add(1, a1 * b0);
        add(2, a1 * b1);
        U256(out)
    }

    fn add(&self, o: &U256) -> U256 {
        let mut out = [0u64; 4];
        let mut carry = 0u128;
        for (i, slot) in out.iter_mut().enumerate() {
            let x = self.0[i] as u128 + o.0[i] as u128 + carry;
            *slot = x as u64;
            carry = x >> 64;
        }
        U256(out)
    }

    /// `self - o`, with `self >= o`.
    fn sub(&self, o: &U256) -> U256 {
        let mut out = [0u64; 4];
        let mut borrow = 0i128;
        for (i, slot) in out.iter_mut().enumerate() {
            let x = self.0[i] as i128 - o.0[i] as i128 - borrow;
            if x < 0 {
                *slot = (x + (1i128 << 64)) as u64;
                borrow = 1;
            } else {
                *slot = x as u64;
                borrow = 0;
            }
        }
        U256(out)
    }

    fn cmp(&self, o: &U256) -> core::cmp::Ordering {
        for i in (0..4).rev() {
            match self.0[i].cmp(&o.0[i]) {
                core::cmp::Ordering::Equal => {}
                c => return c,
            }
        }
        core::cmp::Ordering::Equal
    }

    fn digits(&self) -> u32 {
        if self.fits128() {
            return digits128(self.low128());
        }
        let mut t = *self;
        let mut n = 0;
        while !t.fits128() {
            t.div_small(10_000_000_000_000_000_000);
            n += 19;
        }
        n + digits128(t.low128())
    }
}

fn digits128(v: u128) -> u32 {
    let mut n = 1;
    let mut t = v;
    while t >= 10 {
        t /= 10;
        n += 1;
    }
    n
}

// ------------------------------------------------------------ rounding

/// Why an answer has no value at the width.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Stop {
    /// Past the largest value.
    Over,
    /// A division by zero, in whatever form.
    ByZero,
}

/// The exact `(-1)^neg x c x 10^e` (plus a sticky part below it, when
/// `sticky`) rounded to the width, half to even.
fn round(neg: bool, c: U256, e: i32, sticky: bool, w: Width) -> Result<Dec, Stop> {
    let d = if c.is_zero() { 1 } else { c.digits() };
    let mut drop: i64 = (d as i64 - w.digits as i64).max(0);
    if (e as i64) + drop < w.qmin as i64 {
        drop = w.qmin as i64 - e as i64;
    }
    let mut coef;
    let mut exp = e as i64 + drop;
    if drop == 0 {
        coef = c.low128();
        // Only a sum lined up with a far smaller one carries a sticky part
        // with nothing dropped, and it sits below the last digit: it never
        // reaches half.
    } else if drop > d as i64 {
        // Everything is below half a unit of the last place kept.
        coef = 0;
    } else {
        let mut t = c;
        let mut rest = sticky;
        let mut left = (drop - 1) as u32;
        while left > 0 {
            let s = left.min(19);
            if t.div_small(10u64.pow(s)) != 0 {
                rest = true;
            }
            left -= s;
        }
        let digit = t.div_small(10);
        coef = t.low128();
        let up = digit > 5 || (digit == 5 && (rest || coef & 1 == 1));
        if up {
            coef += 1;
            if coef == pow10(w.digits) {
                coef /= 10;
                exp += 1;
            }
        }
    }
    if exp > w.qmax as i64 {
        if coef == 0 {
            exp = w.qmax as i64;
        } else {
            // Past the top exponent, a coefficient with room left takes
            // the zeros itself ("clamped", IEEE's word).
            let room = w.digits as i64 - digits128(coef) as i64;
            let need = exp - w.qmax as i64;
            if need > room {
                return Err(Stop::Over);
            }
            coef *= pow10(need as u32);
            exp = w.qmax as i64;
        }
    }
    Ok(Dec { neg, coef, exp: exp as i32 })
}

// ---------------------------------------------------------- arithmetic

fn add(a: Dec, b: Dec, w: Width) -> Result<Dec, Stop> {
    // The one with the larger exponent is lined up with the other.
    let (hi, lo) = if a.exp >= b.exp { (a, b) } else { (b, a) };
    if hi.coef == 0 {
        // Nothing to line up: the other, at the smaller exponent. Two
        // zeros are negative only when both are (IEEE 6.3).
        let neg = if lo.coef == 0 { hi.neg && lo.neg } else { lo.neg };
        return Ok(Dec { neg, ..lo });
    }
    let diff = (hi.exp as i64 - lo.exp as i64) as u32;
    let hd = digits128(hi.coef);
    let (big, small, e) = if hd + diff <= 2 * w.digits + 5 {
        let mut x = U256::of(hi.coef);
        x.times_pow10(diff);
        (x, U256::of(lo.coef), lo.exp)
    } else {
        // The smaller one lies wholly below where the answer is rounded,
        // so only "something, of this sign" of it counts: the larger is
        // widened to four digits past the width, and the smaller stands
        // in as one unit below that. Every value between the two rounds
        // the same way.
        let k = w.digits + 3 - hd;
        let mut x = U256::of(hi.coef);
        x.times_pow10(k + 1);
        (x, U256::of((lo.coef != 0) as u128), hi.exp - k as i32 - 1)
    };
    let (neg, sum) = if hi.neg == lo.neg {
        (hi.neg, big.add(&small))
    } else {
        match big.cmp(&small) {
            core::cmp::Ordering::Less => (lo.neg, small.sub(&big)),
            core::cmp::Ordering::Greater => (hi.neg, big.sub(&small)),
            // An exact zero is positive, rounding to nearest (IEEE 6.3).
            core::cmp::Ordering::Equal => (false, U256::of(0)),
        }
    };
    round(neg, sum, e, false, w)
}

fn mul(a: Dec, b: Dec, w: Width) -> Result<Dec, Stop> {
    let c = U256::mul(a.coef, b.coef);
    round(a.neg != b.neg, c, a.exp + b.exp, false, w)
}

/// `a div b`, with `b` not zero. An exact quotient takes the exponent
/// closest to `a`'s less `b`'s (IEEE's preferred exponent): `2.50 div 2`
/// is `1.25`, `10.0 div 4` is `2.5`.
fn div(a: Dec, b: Dec, w: Width) -> Result<Dec, Stop> {
    let neg = a.neg != b.neg;
    let pref = a.exp as i64 - b.exp as i64;
    if a.coef == 0 {
        let e = pref.clamp(w.qmin as i64, w.qmax as i64) as i32;
        return Ok(Dec { neg, coef: 0, exp: e });
    }
    // Long division, a digit at a time, until the quotient has a digit
    // past the width or divides exactly. Every partial remainder is below
    // the divisor, so ten of it fit 128 bits.
    let mut q = a.coef / b.coef;
    let mut rem = a.coef % b.coef;
    let mut s: i64 = 0;
    while rem != 0 && q < pow10(w.digits) {
        rem *= 10;
        q = q * 10 + rem / b.coef;
        rem %= b.coef;
        s += 1;
    }
    let e = pref - s;
    if e < i32::MIN as i64 / 2 || e > i32::MAX as i64 / 2 {
        return if e > 0 { Err(Stop::Over) } else { Ok(Dec { neg, coef: 0, exp: w.qmin }) };
    }
    round(neg, U256::of(q), e as i32, rem != 0, w)
}

fn compare(a: Dec, b: Dec) -> i32 {
    let (az, bz) = (a.coef == 0, b.coef == 0);
    if az && bz {
        return 0;
    }
    let sign = |d: &Dec, z: bool| if z { 0 } else if d.neg { -1 } else { 1 };
    let (sa, sb) = (sign(&a, az), sign(&b, bz));
    if sa != sb {
        return if sa < sb { -1 } else { 1 };
    }
    // Same sign, both non-zero: by magnitude, the leading digit's place
    // first.
    let la = digits128(a.coef) as i64 + a.exp as i64;
    let lb = digits128(b.coef) as i64 + b.exp as i64;
    let mag = if la != lb {
        if la < lb { -1 } else { 1 }
    } else {
        // The same leading place: lined up, each is at most 34 digits.
        let (mut x, mut y) = (a.coef, b.coef);
        if a.exp > b.exp {
            x *= pow10((a.exp - b.exp) as u32);
        } else {
            y *= pow10((b.exp - a.exp) as u32);
        }
        match x.cmp(&y) {
            core::cmp::Ordering::Less => -1,
            core::cmp::Ordering::Equal => 0,
            core::cmp::Ordering::Greater => 1,
        }
    };
    if sa < 0 {
        -mag
    } else {
        mag
    }
}

// ------------------------------------------------------ the C boundary

fn two(lo: u64, hi: u64) -> u128 {
    lo as u128 | (hi as u128) << 64
}

/// `+ - x div` on two `deci`s of `bits`: 0 and the answer in `out`; 1
/// when the answer is past the largest value; 2 on a division by zero.
///
/// # Safety
/// `out` must be writable.
#[no_mangle]
pub unsafe extern "C" fn cyborg_deci_arith(op: u32, alo: u64, ahi: u64, blo: u64, bhi: u64, bits: u32, out: *mut u128) -> u8 {
    let w = width(bits);
    let a = decode(two(alo, ahi), w);
    let mut b = decode(two(blo, bhi), w);
    let r = match op {
        0 => add(a, b, w),
        1 => {
            b.neg = !b.neg;
            add(a, b, w)
        }
        2 => mul(a, b, w),
        _ => {
            if b.coef == 0 {
                return 2;
            }
            div(a, b, w)
        }
    };
    match r {
        Ok(d) => {
            *out = encode(d, w);
            0
        }
        Err(Stop::Over) => 1,
        Err(Stop::ByZero) => 2,
    }
}

/// Compares two `deci`s by value: -1, 0 or 1. `1.50` and `1.5` are equal,
/// and so are `0` and `-0`.
#[no_mangle]
pub extern "C" fn cyborg_deci_compare(alo: u64, ahi: u64, blo: u64, bhi: u64, bits: u32) -> i32 {
    let w = width(bits);
    compare(decode(two(alo, ahi), w), decode(two(blo, bhi), w))
}

/// A whole number into a `deci`: 0 and the value; 1 when it has more
/// digits than the width holds (`too_many_digits`) — by its digits, not
/// by whether a rounding would happen to be exact.
///
/// # Safety
/// `out` must be writable.
#[no_mangle]
pub unsafe extern "C" fn cyborg_deci_from_int(lo: u64, hi: u64, bits: u32, out: *mut u128) -> u8 {
    let w = width(bits);
    let v = two(lo, hi) as i128;
    let m = v.unsigned_abs();
    if digits128(m) > w.digits {
        return 1;
    }
    *out = encode(Dec { neg: v < 0, coef: m, exp: 0 }, w);
    0
}

/// A decimal written out — digits and the power of ten after them — into
/// a `deci`, keeping the width's digits: `fill` pads a shorter one with
/// zeros to the full width (a bin's exact value is every digit it has).
fn from_digits(neg: bool, digits: &[u8], exp: i64, fill: bool, w: Width) -> Result<Dec, Stop> {
    // Leading zeros are no digits.
    let start = digits.iter().position(|&c| c != b'0').unwrap_or(digits.len());
    let digits = &digits[start..];
    if digits.is_empty() {
        return Ok(Dec { neg, coef: 0, exp: 0 });
    }
    let p = w.digits as usize;
    let mut coef: u128 = 0;
    let take = digits.len().min(p + 1);
    for &c in &digits[..take] {
        coef = coef * 10 + (c - b'0') as u128;
    }
    let sticky = digits[take..].iter().any(|&c| c != b'0');
    let mut e = exp + (digits.len() - take) as i64;
    if fill && take < p {
        let pad = (p - take) as u32;
        coef *= pow10(pad);
        e -= pad as i64;
    }
    if e > i32::MAX as i64 / 2 {
        return Err(Stop::Over);
    }
    if e < i32::MIN as i64 / 2 {
        return Ok(Dec { neg, coef: 0, exp: w.qmin });
    }
    round(neg, U256::of(coef), e as i32, sticky, w)
}

/// A bin into a `deci` (`types.md`): `useful` 0 is the exact value
/// rounded to the width, every digit of it; 64, 32 or 16 is the shortest
/// decimal that reads back as that bin, from a bin of that width. 0 and
/// the value; 1 past the largest; 2 for an infinity or NaN, which no bin
/// holds (owner, 2026-09-29) unless one got past its checks unchecked.
///
/// # Safety
/// `out` must be writable.
#[no_mangle]
pub unsafe extern "C" fn cyborg_deci_from_bin(x: f64, bits: u32, useful: u32, out: *mut u128) -> u8 {
    let w = width(bits);
    if !x.is_finite() {
        return 2;
    }
    let neg = x.is_sign_negative();
    if x == 0.0 {
        *out = encode(Dec { neg, coef: 0, exp: 0 }, w);
        return 0;
    }
    let mut digits = [0u8; 1080];
    let (n, exp) = if useful == 0 {
        let mut buf = [0u8; 1080];
        let len = expand(x.abs(), &mut buf);
        split_point(&buf[..len], &mut digits)
    } else {
        let mut buf = [0u8; 64];
        let len = shortest(x.abs(), useful, &mut buf);
        let (d, e10) = split_exponent(&buf[..len]);
        let (n, e) = split_point(d, &mut digits);
        (n, e + e10)
    };
    match from_digits(neg, &digits[..n], exp, useful == 0, w) {
        Ok(d) => {
            *out = encode(d, w);
            0
        }
        Err(_) => 1,
    }
}

/// `1234.5678` → the digits `12345678` and -4.
fn split_point(s: &[u8], digits: &mut [u8]) -> (usize, i64) {
    let mut n = 0;
    let mut after = 0i64;
    let mut seen = false;
    for &c in s {
        if c == b'.' {
            seen = true;
        } else if c.is_ascii_digit() {
            digits[n] = c;
            n += 1;
            if seen {
                after += 1;
            }
        }
    }
    (n, -after)
}

/// `1.5e-7` → `1.5` and -7.
fn split_exponent(s: &[u8]) -> (&[u8], i64) {
    match s.iter().position(|&c| c == b'e' || c == b'E') {
        None => (s, 0),
        Some(at) => {
            let (m, rest) = (&s[..at], &s[at + 1..]);
            let (neg, rest) = match rest.first() {
                Some(b'-') => (true, &rest[1..]),
                Some(b'+') => (false, &rest[1..]),
                _ => (false, rest),
            };
            let mut e = 0i64;
            for &c in rest {
                e = e * 10 + (c - b'0') as i64;
            }
            (m, if neg { -e } else { e })
        }
    }
}

struct Buf<'a> {
    b: &'a mut [u8],
    n: usize,
}

impl core::fmt::Write for Buf<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let s = s.as_bytes();
        if self.n + s.len() > self.b.len() {
            return Err(core::fmt::Error);
        }
        self.b[self.n..self.n + s.len()].copy_from_slice(s);
        self.n += s.len();
        Ok(())
    }
}

/// A bin's useful text (owner, 2026-09-30: `to.useful.str.utf8`): the
/// shortest decimal that reads back as `x` at its own width, written out
/// in full — `0.1`, `100`, `0.000001`, `123456789012345680` — as
/// `print.useful` shows it.
///
/// # Safety
/// `out` must be writable.
#[no_mangle]
pub unsafe extern "C" fn cyborg_bin_to_text_useful(out: *mut Text, x: f64, bits: u32) {
    if x.is_nan() || x.is_infinite() {
        let mut buf = [0u8; 1080];
        let n = expand(x, &mut buf);
        text_of(out, &buf[..n]);
        return;
    }
    // `d.ddde±N`: the digits and where the point goes.
    let mut sci = [0u8; 64];
    let n = shortest(x.abs(), bits, &mut sci);
    let s = &sci[..n];
    let e_at = s.iter().position(|&c| c == b'e').unwrap_or(n);
    let digits: [u8; 64] = {
        let mut d = [0u8; 64];
        let mut k = 0;
        for &c in &s[..e_at] {
            if c.is_ascii_digit() {
                d[k] = c;
                k += 1;
            }
        }
        d
    };
    let nd = s[..e_at].iter().filter(|c| c.is_ascii_digit()).count();
    let mut exp: i64 = 0;
    let mut neg_exp = false;
    for &c in &s[(e_at + 1).min(n)..] {
        match c {
            b'-' => neg_exp = true,
            b'0'..=b'9' => exp = exp * 10 + (c - b'0') as i64,
            _ => {}
        }
    }
    if neg_exp {
        exp = -exp;
    }
    let mut buf = [0u8; 400];
    let mut k = 0;
    let mut put = |c: u8, k: &mut usize| {
        buf[*k] = c;
        *k += 1;
    };
    if x.is_sign_negative() && x != 0.0 {
        put(b'-', &mut k);
    }
    // Where the point falls after the first digit.
    let point = exp + 1;
    if point <= 0 {
        put(b'0', &mut k);
        put(b'.', &mut k);
        for _ in 0..(-point) {
            put(b'0', &mut k);
        }
        for &c in &digits[..nd] {
            put(c, &mut k);
        }
    } else if point as usize >= nd {
        for &c in &digits[..nd] {
            put(c, &mut k);
        }
        for _ in 0..(point as usize - nd) {
            put(b'0', &mut k);
        }
    } else {
        for (i, &c) in digits[..nd].iter().enumerate() {
            if i == point as usize {
                put(b'.', &mut k);
            }
            put(c, &mut k);
        }
    }
    text_of(out, &buf[..k]);
}

/// The shortest decimal that reads back as `x` at `bits`, in scientific
/// form (`1.5e-7`).
fn shortest(x: f64, bits: u32, out: &mut [u8]) -> usize {
    use core::fmt::Write;
    let mut b = Buf { b: out, n: 0 };
    match bits {
        64 => {
            let _ = write!(b, "{:e}", x);
        }
        32 => {
            let _ = write!(b, "{:e}", x as f32);
        }
        _ => {
            // No `f16` or brain16 to format here: the fewest digits that
            // the reading at that width takes back to `x`.
            let width = if bits == crate::BRAIN { crate::BRAIN } else { 16 };
            for p in 0..17 {
                b.n = 0;
                let _ = write!(b, "{:.*e}", p, x);
                let mut back = 0.0f64;
                let status = unsafe { crate::cyborg_parse_bin(b.b.as_ptr(), b.n as u64, width, &mut back) };
                if status == 0 && back == x {
                    break;
                }
            }
        }
    }
    b.n
}

/// Text into a `deci`, by `bin`'s grammar (`types.md`): 0 and the value;
/// 1 not a number; 2 out of range (an exponent the width does not reach
/// without rounding); 3 more digits than the width holds.
///
/// # Safety
/// `p` must have `len` readable bytes, and `out` must be writable.
#[no_mangle]
pub unsafe extern "C" fn cyborg_deci_parse(p: *const u8, len: u64, bits: u32, out: *mut u128) -> u8 {
    let w = width(bits);
    let len = len & !STATIC;
    let b = if len == 0 { &[][..] } else { core::slice::from_raw_parts(p, len as usize) };
    let Some(parts) = bin_grammar(b) else { return 1 };
    let mut coef: u128 = 0;
    let mut count = 0u32;
    for &c in parts.int.iter().chain(parts.frac.iter()) {
        if count == 0 && c == b'0' {
            continue;
        }
        count += 1;
        if count > w.digits {
            return 3;
        }
        coef = coef * 10 + (c - b'0') as u128;
    }
    let q = parts.exp - parts.frac.len() as i64;
    let mut d = Dec { neg: parts.neg, coef, exp: 0 };
    if q < w.qmin as i64 {
        return 2;
    }
    if q > w.qmax as i64 {
        let need = q - w.qmax as i64;
        if coef == 0 || need > (w.digits - digits128(coef)) as i64 {
            return 2;
        }
        d.coef *= pow10(need as u32);
        d.exp = w.qmax;
    } else {
        d.exp = q as i32;
    }
    *out = encode(d, w);
    0
}

/// A `deci` into a whole number: 0 and the value in `out`; 1 when it is
/// not whole (`not_whole`); 2 when it is past what 128 bits hold, which is
/// past every integer type.
///
/// # Safety
/// `out` must be writable.
#[no_mangle]
pub unsafe extern "C" fn cyborg_deci_to_int(lo: u64, hi: u64, bits: u32, out: *mut i128) -> u8 {
    let w = width(bits);
    let d = decode(two(lo, hi), w);
    let mut c = d.coef;
    if c != 0 && d.exp < 0 {
        // A coefficient has at most 34 digits: past 38 places it is all
        // fraction.
        let k = (-d.exp) as u32;
        if k > 38 || c % pow10(k) != 0 {
            return 1;
        }
        c /= pow10(k);
    } else if c != 0 {
        for _ in 0..d.exp {
            match c.checked_mul(10) {
                Some(v) => c = v,
                None => return 2,
            }
        }
    }
    if c > i128::MAX as u128 {
        return 2;
    }
    *out = if d.neg { -(c as i128) } else { c as i128 };
    0
}

/// A `deci` into a bin, rounded to nearest (`types.md`: "to a `bin`
/// rounds, as `bin` math does"), at `to` bits: the value as a 64-bit
/// float, exact for a narrower one.
#[no_mangle]
pub extern "C" fn cyborg_deci_to_bin(lo: u64, hi: u64, bits: u32, to: u32) -> f64 {
    let w = width(bits);
    let d = decode(two(lo, hi), w);
    // Written out as `-<digits>e<exp>` and read by the bin reader, which
    // rounds correctly at every width.
    let mut buf = [0u8; 64];
    let mut n = 0;
    if d.neg {
        buf[0] = b'-';
        n = 1;
    }
    n += coef_digits(d.coef, &mut buf[n..]);
    buf[n] = b'e';
    n += 1;
    if d.exp < 0 {
        buf[n] = b'-';
        n += 1;
    }
    n += coef_digits(d.exp.unsigned_abs() as u128, &mut buf[n..]);
    let mut v = 0.0f64;
    match unsafe { crate::cyborg_parse_bin(buf.as_ptr(), n as u64, to, &mut v) } {
        // Past the bin's largest: the bin's own answer, an infinity.
        2 => {
            if d.neg {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            }
        }
        _ => v,
    }
}

/// The digits of `v` into the front of `buf`; how many.
fn coef_digits(v: u128, buf: &mut [u8]) -> usize {
    let len = digits128(v);
    let mut t = v;
    for i in (0..len as usize).rev() {
        buf[i] = b'0' + (t % 10) as u8;
        t /= 10;
    }
    len as usize
}

/// A `deci` as text (`types.md`): every digit its coefficient holds, the
/// point placed by its exponent, never an exponent written — `2.50` stays
/// `2.50`. `useful` drops the zeros after the point that carry nothing
/// (`2.5`, and `5` for `5.00`).
///
/// # Safety
/// `out` must be writable.
#[no_mangle]
pub unsafe extern "C" fn cyborg_deci_to_text(out: *mut Text, lo: u64, hi: u64, bits: u32, useful: u32) {
    let w = width(bits);
    let d = decode(two(lo, hi), w);
    // The longest: a sign, 34 digits and 6176 zeros before them, or 6111
    // after.
    let mut buf = [0u8; 6300];
    let n = write_dec(d, useful != 0, &mut buf);
    text_of(out, &buf[..n]);
}

fn write_dec(d: Dec, useful: bool, buf: &mut [u8]) -> usize {
    let mut digits = [0u8; 40];
    let len = coef_digits(d.coef, &mut digits);
    let mut n = 0;
    if d.neg {
        buf[0] = b'-';
        n = 1;
    }
    if d.exp >= 0 {
        buf[n..n + len].copy_from_slice(&digits[..len]);
        n += len;
        if d.coef != 0 {
            for _ in 0..d.exp {
                buf[n] = b'0';
                n += 1;
            }
        }
        return n;
    }
    let k = (-d.exp) as usize;
    if len > k {
        buf[n..n + len - k].copy_from_slice(&digits[..len - k]);
        n += len - k;
    } else {
        buf[n] = b'0';
        n += 1;
    }
    let point = n;
    buf[n] = b'.';
    n += 1;
    for _ in len..k {
        buf[n] = b'0';
        n += 1;
    }
    let from = len.saturating_sub(k);
    buf[n..n + (len - from)].copy_from_slice(&digits[from..len]);
    n += len - from;
    if useful {
        while n > point + 1 && buf[n - 1] == b'0' {
            n -= 1;
        }
        if n == point + 1 {
            n = point;
        }
    }
    n
}

/// A `deci` into a wider one, which holds its every value exactly: the
/// same sign, coefficient and exponent in the wider layout.
///
/// # Safety
/// `out` must be writable.
#[no_mangle]
pub unsafe extern "C" fn cyborg_deci_widen(lo: u64, hi: u64, from: u32, to: u32, out: *mut u128) {
    *out = encode(decode(two(lo, hi), width(from)), width(to));
}

/// A hash that agrees with `==` (`maps.md`: a key compares by its `op ==`,
/// and equal keys hash the same): taken over the value with its trailing
/// zeros dropped, so `1.50` and `1.5` hash alike, and every zero alike.
#[no_mangle]
pub extern "C" fn cyborg_deci_hash(lo: u64, hi: u64, bits: u32) -> u64 {
    let d = decode(two(lo, hi), width(bits));
    let (mut c, mut e) = (d.coef, d.exp as i64);
    if c == 0 {
        return crate::cyborg_hash_int(0);
    }
    while c % 10 == 0 {
        c /= 10;
        e += 1;
    }
    let h = crate::cyborg_hash_mix(crate::cyborg_hash_int(c as u64), crate::cyborg_hash_int((c >> 64) as u64));
    crate::cyborg_hash_mix(h, crate::cyborg_hash_int(((e as u64) << 1) | d.neg as u64))
}

// ------------------------------------------------ `mod`, `xx`, narrowing

/// A whole number of up to 2560 bits, least significant limb first: room
/// for a power's working digits squared, and a `mod`'s lined-up operands.
#[derive(Clone, Copy)]
struct Nat {
    l: [u64; 40],
}

impl Nat {
    fn of(v: u128) -> Nat {
        let mut l = [0u64; 40];
        l[0] = v as u64;
        l[1] = (v >> 64) as u64;
        Nat { l }
    }

    fn is_zero(&self) -> bool {
        self.l.iter().all(|&x| x == 0)
    }

    fn fits128(&self) -> bool {
        self.l[2..].iter().all(|&x| x == 0)
    }

    fn low128(&self) -> u128 {
        self.l[0] as u128 | (self.l[1] as u128) << 64
    }

    fn bits(&self) -> u32 {
        for i in (0..40).rev() {
            if self.l[i] != 0 {
                return i as u32 * 64 + 64 - self.l[i].leading_zeros();
            }
        }
        0
    }

    /// `self x k`; false when it no longer fits.
    fn mul_small(&mut self, k: u64) -> bool {
        let mut carry: u128 = 0;
        for limb in self.l.iter_mut() {
            let x = *limb as u128 * k as u128 + carry;
            *limb = x as u64;
            carry = x >> 64;
        }
        carry == 0
    }

    fn times_pow10(&mut self, mut n: u32) -> bool {
        while n > 0 {
            let s = n.min(19);
            if !self.mul_small(10u64.pow(s)) {
                return false;
            }
            n -= s;
        }
        true
    }

    fn div_small(&mut self, k: u64) -> u64 {
        let mut rem: u128 = 0;
        for limb in self.l.iter_mut().rev() {
            let cur = rem << 64 | *limb as u128;
            *limb = (cur / k as u128) as u64;
            rem = cur % k as u128;
        }
        rem as u64
    }

    /// `a x b`, or None when it does not fit.
    fn mul(a: &Nat, b: &Nat) -> Option<Nat> {
        let (na, nb) = (a.bits().div_ceil(64) as usize, b.bits().div_ceil(64) as usize);
        if na + nb > 40 {
            return None;
        }
        let mut out = [0u64; 40];
        for i in 0..na {
            let mut carry: u128 = 0;
            for j in 0..nb {
                let x = out[i + j] as u128 + a.l[i] as u128 * b.l[j] as u128 + carry;
                out[i + j] = x as u64;
                carry = x >> 64;
            }
            out[i + nb] = carry as u64;
        }
        Some(Nat { l: out })
    }

    fn cmp(&self, o: &Nat) -> core::cmp::Ordering {
        for i in (0..40).rev() {
            match self.l[i].cmp(&o.l[i]) {
                core::cmp::Ordering::Equal => {}
                c => return c,
            }
        }
        core::cmp::Ordering::Equal
    }

    fn sub(&mut self, o: &Nat) {
        let mut borrow = 0u64;
        for i in 0..40 {
            let (x, b1) = self.l[i].overflowing_sub(o.l[i]);
            let (y, b2) = x.overflowing_sub(borrow);
            self.l[i] = y;
            borrow = (b1 || b2) as u64;
        }
    }

    fn shl1(&mut self) {
        let mut carry = 0u64;
        for limb in self.l.iter_mut() {
            let next = *limb >> 63;
            *limb = *limb << 1 | carry;
            carry = next;
        }
    }

    /// `self / m` and `self mod m`, by shift and subtract; `m` not zero.
    fn divrem(&self, m: &Nat) -> (Nat, Nat) {
        let mut q = Nat::of(0);
        let mut r = Nat::of(0);
        for i in (0..self.bits()).rev() {
            r.shl1();
            r.l[0] |= (self.l[(i / 64) as usize] >> (i % 64)) & 1;
            if r.cmp(m) != core::cmp::Ordering::Less {
                r.sub(m);
                q.l[(i / 64) as usize] |= 1 << (i % 64);
            }
        }
        (q, r)
    }

    fn add_one(&mut self) {
        for limb in self.l.iter_mut() {
            let (v, carry) = limb.overflowing_add(1);
            *limb = v;
            if !carry {
                break;
            }
        }
    }

    /// `self mod m`, by shift and subtract; `m` not zero.
    fn rem(&self, m: &Nat) -> Nat {
        let mut r = Nat::of(0);
        for i in (0..self.bits()).rev() {
            r.shl1();
            r.l[0] |= (self.l[(i / 64) as usize] >> (i % 64)) & 1;
            if r.cmp(m) != core::cmp::Ordering::Less {
                r.sub(m);
            }
        }
        r
    }

    fn digits(&self) -> u32 {
        let mut t = *self;
        let mut n = 0;
        while !t.fits128() {
            t.div_small(10_000_000_000_000_000_000);
            n += 19;
        }
        n + digits128(t.low128())
    }

    /// Cut to its leading `keep` digits: the digits dropped, and whether any
    /// was not zero.
    fn cut(&mut self, keep: u32) -> (u32, bool) {
        let d = self.digits();
        if d <= keep {
            return (0, false);
        }
        let mut drop = d - keep;
        let n = drop;
        let mut lost = false;
        while drop > 0 {
            let s = drop.min(19);
            if self.div_small(10u64.pow(s)) != 0 {
                lost = true;
            }
            drop -= s;
        }
        (n, lost)
    }
}

/// How a whole quotient rounds, as the op codes name it.
#[derive(Clone, Copy, PartialEq)]
enum Whole {
    Floor,
    Ceil,
    Trunc,
}

/// `a mod.X b`, `b` not zero, what `a div.X b` leaves over (owner,
/// 2026-10-02): at the smaller of the two exponents, exact, then rounded
/// like the rest. `mod.floor` takes the divisor's sign — `-7.50 mod.floor
/// 2` is `0.50` — and so does its 0; `mod.ceil` the divisor's other sign,
/// `mod.trunc` the dividend's; their 0 is `+0`, as the oracle's.
fn modulo(a: Dec, b: Dec, w: Width, how: Whole) -> Result<Dec, Stop> {
    let e = a.exp.min(b.exp);
    // |a| mod |b|, both lined up at `e`.
    let r: u128 = if a.exp >= b.exp {
        // a's coefficient times 10^k, taken mod b's: 10^k by squaring.
        let m = Nat::of(b.coef);
        let mut k = (a.exp - b.exp) as u32;
        let mut acc = Nat::of(a.coef).rem(&m);
        let mut base = Nat::of(10).rem(&m);
        while k > 0 {
            if k & 1 == 1 {
                acc = Nat::mul(&acc, &base).map(|x| x.rem(&m)).unwrap_or(acc);
            }
            base = Nat::mul(&base, &base).map(|x| x.rem(&m)).unwrap_or(base);
            k >>= 1;
        }
        acc.low128()
    } else {
        // b lined up at a's exponent: once it has more digits than a can,
        // a is already the remainder.
        let k = (b.exp - a.exp) as u32;
        let mut m = Nat::of(b.coef);
        if k > 80 || !m.times_pow10(k) {
            a.coef
        } else {
            Nat::of(a.coef).rem(&m).low128()
        }
    };
    // |b| - r, at `e`: fits, being below |b| lined up.
    let other = || {
        let mut m = Nat::of(b.coef);
        m.times_pow10((b.exp - e) as u32);
        m.sub(&Nat::of(r));
        m.low128()
    };
    let (neg, coef) = match how {
        Whole::Floor if r == 0 => (b.neg, 0),
        Whole::Floor if a.neg == b.neg => (b.neg, r),
        Whole::Floor => (b.neg, other()),
        _ if r == 0 => (false, 0),
        Whole::Trunc => (a.neg, r),
        Whole::Ceil if a.neg == b.neg => (!b.neg, other()),
        Whole::Ceil => (!b.neg, r),
    };
    round(neg, U256::of(coef), e, false, w)
}

/// `a div.X b`, `b` not zero, `X` floor, ceil or trunc: the whole quotient,
/// exact, then rounded to the width like the rest, at exponent 0 when it
/// fits; a 0 is `+0`. `a` lined up at `b`'s exponent as far as 2560 bits
/// hold it: past that the quotient has some 700 digits more than the
/// width keeps, and what is past them only says it is not exact.
fn whole_div(a: Dec, b: Dec, w: Width, how: Whole) -> Result<Dec, Stop> {
    let neg = a.neg != b.neg;
    let k = a.exp as i64 - b.exp as i64;
    let mut n = Nat::of(a.coef);
    let mut d = Nat::of(b.coef);
    let mut shift: i64 = 0;
    let (mut q, inexact) = if k >= 0 {
        let j = k.min(700);
        n.times_pow10(j as u32);
        shift = k - j;
        let (q, r) = n.divrem(&d);
        (q, !r.is_zero())
    } else if -k > 700 || !d.times_pow10((-k) as u32) {
        // The divisor lined up is past every dividend: the quotient is 0.
        (Nat::of(0), a.coef != 0)
    } else {
        let (q, r) = n.divrem(&d);
        (q, !r.is_zero())
    };
    // One more, away from zero, where the rounding lies that way. Past the
    // 700 digits the step is far below what the width keeps.
    let away = match how {
        Whole::Floor => neg,
        Whole::Ceil => !neg,
        Whole::Trunc => false,
    };
    if away && inexact && shift == 0 {
        q.add_one();
    }
    if q.is_zero() {
        return Ok(Dec { neg: false, coef: 0, exp: 0 });
    }
    let (dropped, lost) = q.cut(w.digits + 3);
    let e = shift + dropped as i64;
    if e > i32::MAX as i64 / 2 {
        return Err(Stop::Over);
    }
    round(neg, U256::of(q.low128()), e as i32, lost || (shift > 0 && inexact), w)
}

/// `a xx n`, a whole exponent (owner, 2026-09-29): the coefficient's power
/// and the exponent times n, exact while it has at most `p + 40` digits,
/// then rounded like the rest; one over it for a negative n. Past that
/// many digits the power is carried at `p + 40`, cut toward zero, and
/// marked inexact: only an answer within 10^-40 of a tie could round
/// differently.
fn power(a: Dec, n: i64, w: Width) -> Result<Dec, Stop> {
    let m = n.unsigned_abs();
    let neg = a.neg && m & 1 == 1;
    if m == 0 {
        return Ok(Dec { neg: false, coef: 1, exp: 0 });
    }
    if a.coef == 0 {
        if n < 0 {
            return Err(Stop::ByZero);
        }
        let e = (a.exp as i64).saturating_mul(m as i64);
        return round(neg, U256::of(0), e.clamp(w.qmin as i64, w.qmax as i64) as i32, false, w);
    }
    let keep = w.digits + 40;
    // The trailing zeros stand apart: 1.000 xx 100000 is a power of 1.
    let (mut c, mut zeros) = (a.coef, 0u64);
    while c % 10 == 0 {
        c /= 10;
        zeros += 1;
    }
    let mut exp: i128 = a.exp as i128 * m as i128 + zeros as i128 * m as i128;
    // Square and multiply, each step cut to `keep` digits.
    let mut acc = Nat::of(1);
    let mut acc_e: i128 = 0;
    let mut base = Nat::of(c);
    let mut base_e: i128 = 0;
    let mut lost = false;
    let mut k = m;
    loop {
        if k & 1 == 1 {
            acc = Nat::mul(&acc, &base).ok_or(Stop::Over)?;
            acc_e += base_e;
            let (d, l) = acc.cut(keep);
            acc_e += d as i128;
            lost |= l;
        }
        k >>= 1;
        if k == 0 {
            break;
        }
        base = Nat::mul(&base, &base).ok_or(Stop::Over)?;
        base_e *= 2;
        let (d, l) = base.cut(keep);
        base_e += d as i128;
        lost |= l;
    }
    exp += acc_e;
    // The zeros set apart come back, as many as the width has room for:
    // the answer is `acc x 10^(zeros x m)` at `exp - zeros x m`.
    let back = (zeros as i128 * m as i128).min((keep.saturating_sub(acc.digits())) as i128) as u32;
    acc.times_pow10(back);
    exp -= back as i128;
    if n > 0 {
        if exp > i32::MAX as i128 / 2 {
            return Err(Stop::Over);
        }
        if exp < i32::MIN as i128 / 2 {
            return Ok(Dec { neg, coef: 0, exp: w.qmin });
        }
        let mut t = acc;
        let (d, l) = t.cut(w.digits + 3);
        return round(neg, U256::of(t.low128()), (exp + d as i128) as i32, lost || l, w);
    }
    // One over it: long division of 1 by the power, a digit at a time,
    // until a digit past the width or an exact quotient. The first
    // exponent that makes it exact is the one nearest `-exp`.
    let mut rem = Nat::of(1);
    let mut q: u128 = 0;
    let mut s: i128 = 0;
    while !rem.is_zero() && q < pow10(w.digits + 1) {
        rem.mul_small(10);
        let mut digit = 0u128;
        while rem.cmp(&acc) != core::cmp::Ordering::Less {
            rem.sub(&acc);
            digit += 1;
        }
        q = q * 10 + digit;
        s += 1;
    }
    let e = -exp - s;
    if e > i32::MAX as i128 / 2 {
        return Err(Stop::Over);
    }
    if e < i32::MIN as i128 / 2 {
        return Ok(Dec { neg, coef: 0, exp: w.qmin });
    }
    round(neg, U256::of(q), e as i32, lost || !rem.is_zero(), w)
}

/// `a mod.floor b` (op 4), `a xx n` (op 5, `n` a whole number in the second
/// pair), `a div.floor b`, `div.ceil`, `div.trunc` (6, 7, 8) and `a mod.ceil
/// b`, `mod.trunc` (9, 10): as `cyborg_deci_arith`, and 2 for a division
/// by zero or zero to a negative power.
///
/// # Safety
/// `out` must be writable.
#[no_mangle]
pub unsafe extern "C" fn cyborg_deci_arith2(op: u32, alo: u64, ahi: u64, blo: u64, bhi: u64, bits: u32, out: *mut u128) -> u8 {
    let w = width(bits);
    let a = decode(two(alo, ahi), w);
    let r = if op != 5 {
        let b = decode(two(blo, bhi), w);
        if b.coef == 0 {
            return 2;
        }
        match op {
            4 => modulo(a, b, w, Whole::Floor),
            6 => whole_div(a, b, w, Whole::Floor),
            7 => whole_div(a, b, w, Whole::Ceil),
            8 => whole_div(a, b, w, Whole::Trunc),
            9 => modulo(a, b, w, Whole::Ceil),
            _ => modulo(a, b, w, Whole::Trunc),
        }
    } else {
        let n = two(blo, bhi) as i128;
        power(a, n.clamp(i64::MIN as i128, i64::MAX as i128) as i64, w)
    };
    match r {
        Ok(d) => {
            *out = encode(d, w);
            0
        }
        Err(Stop::Over) => 1,
        Err(Stop::ByZero) => 2,
    }
}

/// A `deci` into a narrower one (owner, 2026-09-29): 0 and the value when
/// its digits fit; 1 when they do not (`too_many_digits`) — trailing zeros
/// are digits, as everywhere; 2 when it is past the narrower's largest.
/// A value below the narrower's smallest place rounds, as math does.
///
/// # Safety
/// `out` must be writable.
#[no_mangle]
pub unsafe extern "C" fn cyborg_deci_narrow(lo: u64, hi: u64, from: u32, to: u32, out: *mut u128) -> u8 {
    let (wf, wt) = (width(from), width(to));
    let d = decode(two(lo, hi), wf);
    if d.coef != 0 && digits128(d.coef) > wt.digits {
        return 1;
    }
    match round(d.neg, U256::of(d.coef), d.exp, false, wt) {
        Ok(r) => {
            *out = encode(r, wt);
            0
        }
        Err(_) => 2,
    }
}
