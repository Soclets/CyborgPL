//! `cord::print`'s output (owner, 2026-10-09: "`b`, not the entire
//! fucking neuron"): a fixed buffer the generated code keeps on its stack,
//! filled by these functions and handed on whole when full and when the
//! print ends. Nothing here uses the heap, so a chip with no OS prints the
//! same way: what differs is only where the bytes go — a file descriptor
//! on an OS, the target's `output` with none (one byte a call, or a chunk
//! a call, owner: "both").

use crate::deci::{bin_useful, deci_text};
use crate::{char_utf8, expand, int_digits};

/// How full a buffer gets before it is handed on.
pub const CAP: usize = 512;

/// Where the bytes go.
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sink {
    /// `write` to a file descriptor: 1 or 2.
    Fd = 0,
    /// The target's output, one `u8` a call.
    Byte = 1,
    /// The target's output, a `lent array[u8]` a call: its address and
    /// its length, as a lent array is passed.
    Chunk = 2,
}

/// The buffer, laid out as the generated code lays it out on its stack.
#[repr(C)]
pub struct Out {
    /// The descriptor, or the output's address.
    pub to: usize,
    pub sink: u32,
    pub len: u32,
    pub bytes: [u8; CAP],
}

#[cfg(not(target_os = "none"))]
extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn fflush(stream: *mut u8) -> i32;
}

/// A lent `array[u8]` as a function receives one.
#[repr(C)]
struct Array {
    ptr: *const u8,
    len: u64,
}

/// # Safety
/// `o` must point at an `Out`; `to` is a descriptor or an output's address
/// of the kind `sink` says.
#[no_mangle]
pub unsafe extern "C" fn cyborg_out_init(o: *mut Out, sink: u32, to: usize) {
    (*o).to = to;
    (*o).sink = sink;
    (*o).len = 0;
}

/// Hand on what the buffer holds.
///
/// # Safety
/// `o` must have been set up by `cyborg_out_init`.
#[no_mangle]
pub unsafe extern "C" fn cyborg_out_flush(o: *mut Out) {
    let n = (*o).len as usize;
    if n == 0 {
        return;
    }
    let p = (*o).bytes.as_ptr();
    match (*o).sink {
        // A file descriptor is the OS's; with none there is never one.
        #[cfg(not(target_os = "none"))]
        0 => {
            // What C's own stdio holds goes first (`printf` through
            // `= C.printf`, owner 2026-10-19): the output reads in the
            // order the program wrote it.
            fflush(core::ptr::null_mut());
            let mut done = 0;
            while done < n {
                let w = write((*o).to as i32, p.add(done), n - done);
                if w <= 0 {
                    break;
                }
                done += w as usize;
            }
        }
        1 => {
            let f: extern "C" fn(u8) = core::mem::transmute((*o).to);
            for i in 0..n {
                f(*p.add(i));
            }
        }
        _ => {
            let f: extern "C" fn(*const Array) = core::mem::transmute((*o).to);
            let a = Array { ptr: p, len: n as u64 };
            f(&a);
        }
    }
    (*o).len = 0;
}

/// Bytes into the buffer, handing it on whenever it fills.
///
/// # Safety
/// `o` as for `cyborg_out_flush`; `p` must have `n` readable bytes.
#[no_mangle]
pub unsafe extern "C" fn cyborg_out_bytes(o: *mut Out, p: *const u8, n: u64) {
    let mut p = p;
    let mut left = n as usize;
    while left > 0 {
        let room = CAP - (*o).len as usize;
        if room == 0 {
            cyborg_out_flush(o);
            continue;
        }
        let k = room.min(left);
        core::ptr::copy_nonoverlapping(p, (*o).bytes.as_mut_ptr().add((*o).len as usize), k);
        (*o).len += k as u32;
        p = p.add(k);
        left -= k;
    }
}

unsafe fn put(o: *mut Out, s: &[u8]) {
    cyborg_out_bytes(o, s.as_ptr(), s.len() as u64);
}

/// A whole number of up to 128 bits, as two halves, signed or not.
///
/// # Safety
/// As `cyborg_out_bytes`.
#[no_mangle]
pub unsafe extern "C" fn cyborg_out_int(o: *mut Out, lo: u64, hi: u64, signed: bool) {
    let mut buf = [0u8; 41];
    let at = int_digits(lo, hi, signed, &mut buf);
    put(o, &buf[at..]);
}

/// A bin (any width, widened to 64 bits, which is exact): its full
/// expansion, or with `useful` the shortest decimal that reads back at its
/// own width.
///
/// # Safety
/// As `cyborg_out_bytes`.
#[no_mangle]
pub unsafe extern "C" fn cyborg_out_bin(o: *mut Out, x: f64, bits: u32, useful: bool) {
    let mut buf = [0u8; 1080];
    let n = if useful { bin_useful(x, bits, &mut buf) } else { expand(x, &mut buf) };
    put(o, &buf[..n]);
}

/// A percent's digits (owner, 2026-10-17): the number's own, its point two
/// places on, so nothing is rounded (`0.07` is `7`, `1e-7` is `0.00001`).
/// `src` is the number as `bin_useful` or `deci_text` writes it.
fn percent_digits(src: &[u8], dst: &mut [u8]) -> usize {
    let (neg, s) = match src.first() {
        Some(b'-') => (true, &src[1..]),
        _ => (false, src),
    };
    // The digits, where the point sat among them, and an exponent.
    let mut digits = [0u8; 1200];
    let (mut nd, mut point, mut exp, mut seen_point, mut i) = (0usize, None, 0i64, false, 0usize);
    while i < s.len() {
        match s[i] {
            d @ b'0'..=b'9' if nd < digits.len() => {
                digits[nd] = d;
                nd += 1;
            }
            b'.' => {
                point = Some(nd);
                seen_point = true;
            }
            b'e' | b'E' => {
                let (mut sign, mut v, mut j) = (1i64, 0i64, i + 1);
                if j < s.len() && (s[j] == b'-' || s[j] == b'+') {
                    if s[j] == b'-' {
                        sign = -1;
                    }
                    j += 1;
                }
                while j < s.len() && s[j].is_ascii_digit() {
                    v = v * 10 + (s[j] - b'0') as i64;
                    j += 1;
                }
                exp = sign * v;
                break;
            }
            _ => {}
        }
        i += 1;
    }
    let _ = seen_point;
    let at = point.unwrap_or(nd) as i64 + exp + 2;
    let mut n = 0usize;
    let mut push = |dst: &mut [u8], b: u8| {
        if n < dst.len() {
            dst[n] = b;
            n += 1;
        }
    };
    // The whole part, without leading zeros, and the rest, without
    // trailing ones.
    let digit = |k: i64| if k >= 0 && (k as usize) < nd { digits[k as usize] } else { b'0' };
    let mut whole_start = 0i64;
    while whole_start < at - 1 && digit(whole_start) == b'0' {
        whole_start += 1;
    }
    let mut frac_end = nd as i64;
    while frac_end > at && digit(frac_end - 1) == b'0' {
        frac_end -= 1;
    }
    let zero = (whole_start..at.max(0)).all(|k| digit(k) == b'0') && (at.max(0)..frac_end).all(|k| digit(k) == b'0');
    if neg && !zero {
        push(dst, b'-');
    }
    if at <= 0 {
        push(dst, b'0');
    } else {
        for k in whole_start..at {
            push(dst, digit(k));
        }
    }
    if frac_end > at {
        push(dst, b'.');
        for k in at.min(0)..0 {
            let _ = k;
            push(dst, b'0');
        }
        for k in at.max(0)..frac_end {
            push(dst, digit(k));
        }
    }
    n
}

/// A percent of a bin, as `print.useful` shows it: `7%` (owner, 2026-10-17).
///
/// # Safety
/// As `cyborg_out_bytes`.
#[no_mangle]
pub unsafe extern "C" fn cyborg_out_bin_percent(o: *mut Out, x: f64, bits: u32, min: u32) {
    let mut buf = [0u8; 1080];
    let n = bin_useful(x, bits, &mut buf);
    let mut out = [0u8; 1300];
    let m = percent_digits(&buf[..n], &mut out[..1199]);
    let m = pad_places(&mut out, m, min as usize);
    out[m] = b'%';
    put(o, &out[..m + 1]);
}

/// At least `min` places after the point, as `min_places` asks (owner,
/// 2026-10-17): `5` is `5.00`. Room is left for the zeros and one byte more.
fn pad_places(buf: &mut [u8], n: usize, min: usize) -> usize {
    if min == 0 {
        return n;
    }
    let (mut n, have) = match buf[..n].iter().position(|&b| b == b'.') {
        Some(p) => (n, n - p - 1),
        None => {
            buf[n] = b'.';
            (n + 1, 0)
        }
    };
    for _ in have..min {
        if n + 1 >= buf.len() {
            break;
        }
        buf[n] = b'0';
        n += 1;
    }
    n
}

/// A deci in `print.useful`, padded to `min_places`.
///
/// # Safety
/// As `cyborg_out_bytes`.
#[no_mangle]
pub unsafe extern "C" fn cyborg_out_deci_padded(o: *mut Out, lo: u64, hi: u64, bits: u32, min: u32) {
    let mut buf = [0u8; 6300];
    let n = deci_text(lo, hi, bits, true, &mut buf);
    let mut out = [0u8; 6400];
    out[..n].copy_from_slice(&buf[..n]);
    let m = pad_places(&mut out, n, min as usize);
    put(o, &out[..m]);
}

/// A percent of a deci, as `print.useful` shows it: `12.25%`.
///
/// # Safety
/// As `cyborg_out_bytes`.
#[no_mangle]
pub unsafe extern "C" fn cyborg_out_deci_percent(o: *mut Out, lo: u64, hi: u64, bits: u32, min: u32) {
    let mut buf = [0u8; 6300];
    let n = deci_text(lo, hi, bits, true, &mut buf);
    let mut out = [0u8; 6500];
    let m = percent_digits(&buf[..n], &mut out[..6399]);
    let m = pad_places(&mut out, m, min as usize);
    out[m] = b'%';
    put(o, &out[..m + 1]);
}

/// A character: the UTF-8 of its code point.
///
/// # Safety
/// As `cyborg_out_bytes`.
#[no_mangle]
pub unsafe extern "C" fn cyborg_out_char(o: *mut Out, code: u32) {
    let mut buf = [0u8; 4];
    let n = char_utf8(code, &mut buf);
    put(o, &buf[..n]);
}

/// A deci, as two halves at `bits`.
///
/// # Safety
/// As `cyborg_out_bytes`.
#[no_mangle]
pub unsafe extern "C" fn cyborg_out_deci(o: *mut Out, lo: u64, hi: u64, bits: u32, useful: bool) {
    let mut buf = [0u8; 6300];
    let n = deci_text(lo, hi, bits, useful, &mut buf);
    put(o, &buf[..n]);
}
