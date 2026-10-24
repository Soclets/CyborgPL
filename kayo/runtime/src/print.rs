//! `std::print`'s output (owner, 2026-10-09: "`b`, not the entire
//! fucking neuron"): one buffer every print fills (owner, 2026-10-23: "yes,
//! and there should also be a manual way"), sent when it is full and
//! wherever the order could matter — at the end of the program, before a C
//! call, before a print to the other stream, and before the program stops
//! on a check (design claude's list). The generated code takes it at a
//! print's start and gives it back at its end. Nothing here uses the heap,
//! so a chip with no OS prints the same way: what differs is only where the
//! bytes go — a file descriptor on an OS, the target's `output` with none
//! (one byte a call, or a chunk a call, owner: "both").

use crate::deci::{bin_useful, deci_text};
use crate::{char_utf8, expand, int_digits};

/// How full the buffer gets before it is handed on.
pub const CAP: usize = 4096;

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

/// A buffer: kayo's one, or a writer the program made (owner,
/// 2026-10-23: "both, bro").
#[repr(C)]
pub struct Out {
    /// The descriptor, or the output's address.
    pub to: usize,
    pub sink: u32,
    pub len: u32,
    pub cap: u32,
    /// A write the OS refused since this was last asked.
    pub failed: u32,
    pub bytes: *mut u8,
}

#[cfg(not(target_os = "none"))]
extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn fflush(stream: *mut u8) -> i32;
    fn exit(code: i32) -> !;
}

/// The one buffer.
static mut SHARED_BYTES: [u8; CAP] = [0; CAP];
static mut SHARED: Out = Out { to: 1, sink: 0, len: 0, cap: CAP as u32, failed: 0, bytes: core::ptr::null_mut() };

/// kayo's buffer, its bytes in place.
unsafe fn shared() -> *mut Out {
    let o = &raw mut SHARED;
    if (*o).bytes.is_null() {
        (*o).bytes = (&raw mut SHARED_BYTES).cast();
    }
    o
}

/// A write the OS refused, at a print with no `otherwise` or a flush kayo
/// put in itself: the program stops (owner, 2026-10-23: "default to stop").
unsafe fn stop_if_failed(o: *mut Out) {
    if (*o).failed != 0 {
        (*o).failed = 0;
        #[cfg(not(target_os = "none"))]
        stopped_writing();
    }
}

/// Held by the print filling the buffer: a task's print never runs into
/// another's. A chip with no OS has one thread, and its memory, the MMU
/// off, takes no exclusive access, so it has no lock.
#[cfg(not(target_os = "none"))]
static TAKEN: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

fn take() {
    #[cfg(not(target_os = "none"))]
    while TAKEN.compare_exchange_weak(false, true, core::sync::atomic::Ordering::Acquire, core::sync::atomic::Ordering::Relaxed).is_err() {
        core::hint::spin_loop();
    }
}

fn give_back() {
    #[cfg(not(target_os = "none"))]
    TAKEN.store(false, core::sync::atomic::Ordering::Release);
}

/// A write the OS refused, at a flush kayo put in itself: the program
/// stops (owner, 2026-10-23: "default to stop"). How a stop reads is not
/// decided; until it is, it ends with 1.
#[cfg(not(target_os = "none"))]
unsafe fn stopped_writing() -> ! {
    exit(1)
}

/// A print's start: the buffer, sent first if it holds another stream's
/// bytes.
///
/// # Safety
/// `to` is a descriptor or an output's address of the kind `sink` says;
/// `cyborg_print_end` gives the buffer back.
#[no_mangle]
pub unsafe extern "C" fn cyborg_print_begin(sink: u32, to: usize) -> *mut Out {
    take();
    let o = shared();
    if (*o).len > 0 && ((*o).sink != sink || (*o).to != to) {
        cyborg_out_flush(o);
        if (*o).failed != 0 {
            give_back();
            stop_if_failed(o);
        }
    }
    (*o).sink = sink;
    (*o).to = to;
    o
}

/// A print's end: the buffer kept for the next.
///
/// # Safety
/// `o` is what `cyborg_print_begin` handed out.
#[no_mangle]
pub unsafe extern "C" fn cyborg_print_end(o: *mut Out) {
    give_back();
    stop_if_failed(o);
}

/// The end of a print with `otherwise` (owner, 2026-10-23): it flushes
/// there, so its handler covers everything printed so far; 1 when a write
/// failed.
///
/// # Safety
/// `o` is what `cyborg_print_begin` handed out.
#[no_mangle]
pub unsafe extern "C" fn cyborg_print_end_status(o: *mut Out) -> u8 {
    cyborg_out_flush(o);
    let f = (*o).failed;
    (*o).failed = 0;
    give_back();
    (f != 0) as u8
}

/// `std::print.writer(size: N bytes)` (owner, 2026-10-23): the program's
/// own buffer for a stream.
///
/// # Safety
/// As `cyborg_print_begin`'s `sink` and `to`.
#[cfg(not(target_os = "none"))]
#[no_mangle]
pub unsafe extern "C" fn cyborg_writer_new(cap: u64, sink: u32, to: usize) -> *mut Out {
    let cap = cap.clamp(1, u32::MAX as u64);
    let o = crate::cyborg_alloc(core::mem::size_of::<Out>() as u64) as *mut Out;
    let bytes = crate::cyborg_alloc(cap);
    o.write(Out { to, sink, len: 0, cap: cap as u32, failed: 0, bytes });
    o
}

/// A print into a writer has ended: a write that failed when it filled
/// stops the program, with no `otherwise`.
///
/// # Safety
/// `o` is a writer.
#[no_mangle]
pub unsafe extern "C" fn cyborg_writer_end(o: *mut Out) {
    stop_if_failed(o);
}

/// A print into a writer with `otherwise`: 1 when a write failed as it
/// filled.
///
/// # Safety
/// `o` is a writer.
#[no_mangle]
pub unsafe extern "C" fn cyborg_writer_status(o: *mut Out) -> u8 {
    let f = (*o).failed;
    (*o).failed = 0;
    (f != 0) as u8
}

/// `std::print.flush(lend.mut $out)`: 1 when a write failed.
///
/// # Safety
/// `o` is a writer.
#[no_mangle]
pub unsafe extern "C" fn cyborg_writer_flush(o: *mut Out) -> u8 {
    cyborg_out_flush(o);
    cyborg_writer_status(o)
}

/// `std::print.flush(lend.mut $out)` with no `otherwise`.
///
/// # Safety
/// `o` is a writer.
#[no_mangle]
pub unsafe extern "C" fn cyborg_writer_flush_or_stop(o: *mut Out) {
    cyborg_out_flush(o);
    stop_if_failed(o);
}

/// A writer going away; what it held was flushed (kayo refuses one that
/// was not).
///
/// # Safety
/// `o` is a writer, used no more.
#[cfg(not(target_os = "none"))]
#[no_mangle]
pub unsafe extern "C" fn cyborg_writer_free(o: *mut Out) {
    crate::cyborg_free((*o).bytes);
    crate::cyborg_free(o as *mut u8);
}

/// What the buffer holds, sent now: before a C call, at the end of the
/// program, and before it stops.
///
/// # Safety
/// Any time; it waits for a print being filled to end.
#[no_mangle]
pub unsafe extern "C" fn cyborg_print_flush() {
    if (*(&raw const SHARED)).len == 0 {
        return;
    }
    take();
    let o = shared();
    cyborg_out_flush(o);
    give_back();
    stop_if_failed(o);
}

/// Before the program stops on a check: what the buffer holds, unless a
/// print is being filled on this very stop (then the lock never comes
/// back, and what it holds is left).
///
/// # Safety
/// Any time.
pub unsafe fn flush_before_stop() {
    #[cfg(not(target_os = "none"))]
    {
        if (*(&raw const SHARED)).len == 0 {
            return;
        }
        for _ in 0..1_000_000 {
            if TAKEN.compare_exchange_weak(false, true, core::sync::atomic::Ordering::Acquire, core::sync::atomic::Ordering::Relaxed).is_ok() {
                cyborg_out_flush(shared());
                give_back();
                return;
            }
            core::hint::spin_loop();
        }
    }
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
    let p = (*o).bytes as *const u8;
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
                    // What is held goes; whoever asked says what happens.
                    (*o).failed = 1;
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
        let room = (*o).cap as usize - (*o).len as usize;
        if room == 0 {
            cyborg_out_flush(o);
            continue;
        }
        let k = room.min(left);
        core::ptr::copy_nonoverlapping(p, (*o).bytes.add((*o).len as usize), k);
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
