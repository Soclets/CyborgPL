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
    /// Where the oldest print still held was written, as its stop says it
    /// (`error[write_failed] at …`, owner 2026-10-24), and where the print
    /// being filled is.
    pub at: *const u8,
    pub at_len: u64,
    pub cur: *const u8,
    pub cur_len: u64,
    /// A print kept whole (owner, 2026-10-24: the writer's default): when
    /// one does not fit, what came before it is sent and it is kept
    /// together. 0 for `.split` and for kayo's own buffer.
    pub whole: u32,
    /// Where the print being filled starts in the buffer.
    pub mark: u32,
}

#[cfg(not(target_os = "none"))]
extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn fflush(stream: *mut u8) -> i32;
    fn exit(code: i32) -> !;
}

/// The one buffer.
static mut SHARED_BYTES: [u8; CAP] = [0; CAP];
static mut SHARED: Out = Out { to: 1, sink: 0, len: 0, cap: CAP as u32, failed: 0, bytes: core::ptr::null_mut(), at: core::ptr::null(), at_len: 0, cur: core::ptr::null(), cur_len: 0, whole: 0, mark: 0 };

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
        stopped_writing(o);
    }
}

/// Where the print about to fill `o` is written: the oldest held, when the
/// buffer holds nothing yet.
///
/// # Safety
/// `o` is a buffer; `at` has `len` bytes that live as long as the program.
#[no_mangle]
pub unsafe extern "C" fn cyborg_out_at(o: *mut Out, at: *const u8, len: u64) {
    (*o).cur = at;
    (*o).cur_len = len;
    (*o).mark = (*o).len;
    if (*o).len == 0 || (*o).at.is_null() {
        (*o).at = at;
        (*o).at_len = len;
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

/// A write the OS refused, with no `otherwise`: the program stops (owner,
/// 2026-10-23: "default to stop"). How a stop reads is not decided; until
/// it is, it says so on stderr, as the oracle does, and ends with 1.
#[cfg(not(target_os = "none"))]
unsafe fn stopped_writing(o: *const Out) -> ! {
    // `error[write_failed]` and where the print is (owner, 2026-10-24: "(a)").
    if (*o).at.is_null() {
        let say = b"error[write_failed]\n";
        write(2, say.as_ptr(), say.len());
    } else {
        write(2, (*o).at, (*o).at_len as usize);
    }
    exit(1)
}

/// The program's start: a write to a pipe whose reader has gone fails as a
/// write, which `otherwise` handles or which stops the program (owner,
/// 2026-10-23), rather than the OS ending the program on its own.
///
/// # Safety
/// Once, before the program's code.
#[cfg(not(target_os = "none"))]
#[no_mangle]
pub unsafe extern "C" fn cyborg_program_start() {
    extern "C" {
        fn signal(sig: i32, handler: usize) -> usize;
    }
    // SIGPIPE is 13 and SIG_IGN is 1 on macOS and Linux alike.
    signal(13, 1);
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
pub unsafe extern "C" fn cyborg_writer_new(cap: u64, sink: u32, to: usize, whole: u32) -> *mut Out {
    let cap = cap.clamp(1, u32::MAX as u64);
    let o = crate::cyborg_alloc(core::mem::size_of::<Out>() as u64) as *mut Out;
    let bytes = crate::cyborg_alloc(cap);
    o.write(Out { to, sink, len: 0, cap: cap as u32, failed: 0, bytes, at: core::ptr::null(), at_len: 0, cur: core::ptr::null(), cur_len: 0, whole, mark: 0 });
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
    // A writer's text goes out after what kayo's own buffer holds: grouping
    // prints is speed, never order (design claude, 2026-10-24; check
    // claude's flushed-writer finding).
    if o != shared() && (*(&raw const SHARED)).len > 0 {
        cyborg_print_flush();
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
    // What is still to come is the print being filled.
    if (*o).failed == 0 {
        (*o).at = (*o).cur;
        (*o).at_len = (*o).cur_len;
    }
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
        // A format's scratch grows rather than being sent.
        #[cfg(not(target_os = "none"))]
        if room == 0 && (*o).sink == GROW {
            let cap = ((*o).cap as u64 * 2).max(64);
            (*o).bytes = crate::cyborg_realloc((*o).bytes, cap);
            (*o).cap = cap as u32;
            continue;
        }
        if room == 0 {
            // Kept whole: what came before this print goes, and the print
            // starts the buffer; one longer than the buffer goes out alone.
            let mark = (*o).mark as usize;
            if (*o).whole != 0 && mark > 0 {
                let keep = (*o).len as usize - mark;
                (*o).len = mark as u32;
                cyborg_out_flush(o);
                core::ptr::copy((*o).bytes.add(mark), (*o).bytes, keep);
                (*o).len = keep as u32;
            } else {
                cyborg_out_flush(o);
            }
            (*o).mark = 0;
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

// ------------------------------------------------------------ formatting
//
// `std::print.format` and `std::format` (owner, 2026-09-22; `{}` holes since
// 2026-10-01, `examples/print_format.cy`): each hole is written into a
// scratch buffer that grows, reshaped there (`.places`, `e`), then padded and
// aligned into the print's own.

/// A scratch buffer's sink: it grows and is never sent.
const GROW: u32 = 3;

/// A hole's scratch buffer.
///
/// # Safety
/// Freed by `cyborg_fmt_put` or `cyborg_fmt_text`.
#[cfg(not(target_os = "none"))]
#[no_mangle]
pub unsafe extern "C" fn cyborg_fmt_scratch() -> *mut Out {
    let o = crate::cyborg_alloc(core::mem::size_of::<Out>() as u64) as *mut Out;
    let bytes = crate::cyborg_alloc(64);
    o.write(Out { to: 0, sink: GROW, len: 0, cap: 64, failed: 0, bytes, at: core::ptr::null(), at_len: 0, cur: core::ptr::null(), cur_len: 0, whole: 0, mark: 0 });
    o
}

unsafe fn scratch_bytes<'a>(s: *mut Out) -> &'a [u8] {
    core::slice::from_raw_parts((*s).bytes, (*s).len as usize)
}

/// A whole number in base 16 (`x`, `X` upper), 8 or 2, a `-` before it.
///
/// # Safety
/// `s` is a buffer.
#[no_mangle]
pub unsafe extern "C" fn cyborg_fmt_int_base(s: *mut Out, lo: u64, hi: u64, signed: bool, base: u32, upper: bool) {
    let v = ((hi as u128) << 64) | lo as u128;
    let (neg, mut m) = if signed && (v as i128) < 0 { (true, (v as i128).unsigned_abs()) } else { (false, v) };
    let mut digits = [0u8; 130];
    let mut n = 0;
    let set: &[u8] = if upper { b"0123456789ABCDEF" } else { b"0123456789abcdef" };
    loop {
        digits[n] = set[(m % base as u128) as usize];
        n += 1;
        m /= base as u128;
        if m == 0 {
            break;
        }
    }
    if neg {
        put(s, b"-");
    }
    while n > 0 {
        n -= 1;
        put(s, &digits[n..n + 1]);
    }
}

/// The decimal text in `s` (a number's stored or readable form), reshaped:
/// `places` (or -1) digits after the point, rounded half to even on the
/// exact digits; `sci`, scientific, `1.25e1`.
///
/// # Safety
/// `s` is a scratch buffer.
#[cfg(not(target_os = "none"))]
#[no_mangle]
pub unsafe extern "C" fn cyborg_fmt_decimal(s: *mut Out, places: i32, sci: bool) {
    // Past 80 places a decimal shows nothing more a reader can use.
    let places = places.min(80);
    let text = scratch_bytes(s);
    let (neg, body) = match text.first() {
        Some(b'-') => (true, &text[1..]),
        _ => (false, text),
    };
    // Not a plain decimal (`inf`, `NaN`): left as it is.
    if body.is_empty() || !body.iter().all(|c| c.is_ascii_digit() || *c == b'.') {
        return;
    }
    let (int_part, frac_part) = match body.iter().position(|c| *c == b'.') {
        Some(p) => (&body[..p], &body[p + 1..]),
        None => (body, &b""[..]),
    };
    // All the digits, and where the point sits among them.
    let mut digits: [u8; 1200] = [0; 1200];
    let mut n = 0;
    for &c in int_part.iter().chain(frac_part.iter()) {
        if n < digits.len() {
            digits[n] = c - b'0';
            n += 1;
        }
    }
    let mut point = int_part.len() as i64;
    // Leading zeros out, so the first digit is the first that counts.
    let mut lead = 0;
    while lead + 1 < n && digits[lead] == 0 {
        lead += 1;
    }
    let all_zero = digits[lead..n].iter().all(|d| *d == 0);
    let mut out: [u8; 1300] = [0; 1300];
    let mut w = 0;
    let mut emit = |c: u8, w: &mut usize| {
        if *w < out.len() {
            out[*w] = c;
            *w += 1;
        }
    };
    // Round the digits to `keep` of them, half to even; true when a carry
    // made a new first digit.
    let round = |d: &mut [u8], len: usize, keep: usize| -> bool {
        if keep >= len {
            return false;
        }
        let next = d[keep];
        let rest_zero = d[keep + 1..len].iter().all(|x| *x == 0);
        let last_odd = keep > 0 && d[keep - 1] % 2 == 1;
        let up = next > 5 || (next == 5 && (!rest_zero || last_odd));
        for x in d[keep..len].iter_mut() {
            *x = 0;
        }
        if !up {
            return false;
        }
        let mut i = keep;
        while i > 0 {
            i -= 1;
            if d[i] == 9 {
                d[i] = 0;
            } else {
                d[i] += 1;
                return false;
            }
        }
        true
    };
    if neg && !(all_zero) {
        emit(b'-', &mut w);
    }
    if sci {
        if all_zero {
            emit(b'0', &mut w);
            if places > 0 {
                emit(b'.', &mut w);
                for _ in 0..places {
                    emit(b'0', &mut w);
                }
            }
            for &c in b"e0" {
                emit(c, &mut w);
            }
        } else {
            let mut d: [u8; 1201] = [0; 1201];
            let len = n - lead;
            d[..len].copy_from_slice(&digits[lead..n]);
            let mut exp = point - lead as i64 - 1;
            let shown = if places >= 0 {
                let keep = places as usize + 1;
                if round(&mut d, len.max(keep), keep) {
                    d.copy_within(0..keep, 1);
                    d[0] = 1;
                    exp += 1;
                }
                keep
            } else {
                // The readable digits, trailing zeros dropped.
                let mut k = len;
                while k > 1 && d[k - 1] == 0 {
                    k -= 1;
                }
                k
            };
            emit(b'0' + d[0], &mut w);
            if shown > 1 {
                emit(b'.', &mut w);
                for &x in &d[1..shown] {
                    emit(b'0' + x, &mut w);
                }
            }
            emit(b'e', &mut w);
            if exp < 0 {
                emit(b'-', &mut w);
            }
            let mut e = exp.unsigned_abs();
            let mut ed = [0u8; 20];
            let mut en = 0;
            loop {
                ed[en] = b'0' + (e % 10) as u8;
                en += 1;
                e /= 10;
                if e == 0 {
                    break;
                }
            }
            while en > 0 {
                en -= 1;
                emit(ed[en], &mut w);
            }
        }
    } else {
        let places = places.max(0) as usize;
        // Digits up to `places` after the point, rounded.
        let mut d: [u8; 1300] = [0; 1300];
        let keep_total = (point.max(0) as usize) + places;
        let len = n.max(keep_total);
        d[..n].copy_from_slice(&digits[..n]);
        if round(&mut d, len, keep_total) {
            d.copy_within(0..keep_total, 1);
            d[0] = 1;
            point += 1;
        }
        let int_len = point.max(0) as usize;
        let mut first = 0;
        while first + 1 < int_len && d[first] == 0 {
            first += 1;
        }
        if int_len == 0 {
            emit(b'0', &mut w);
        }
        for &x in &d[first..int_len] {
            emit(b'0' + x, &mut w);
        }
        if places > 0 {
            emit(b'.', &mut w);
            for &x in &d[int_len..int_len + places] {
                emit(b'0' + x, &mut w);
            }
        }
    }
    (*s).len = 0;
    cyborg_out_bytes(s, out.as_ptr(), w as u64);
}

/// A hole's text, from its scratch buffer into the print's, padded to
/// `width` characters with `fill` (`align`: 0 the kind's own, numbers right
/// and the rest left; 1 `<`, 2 `^`, 3 `>`), `+` before a number that has no
/// `-`, and `0` padding after its sign. The scratch buffer is freed.
///
/// # Safety
/// `o` a buffer, `s` a scratch buffer from `cyborg_fmt_scratch`.
#[cfg(not(target_os = "none"))]
#[no_mangle]
pub unsafe extern "C" fn cyborg_fmt_put(o: *mut Out, s: *mut Out, width: u32, unit: u32, align: u32, fill: u32, plus: bool, zero: bool, numeric: bool) {
    let text = scratch_bytes(s);
    let (sign, rest): (&[u8], &[u8]) = match text.first() {
        Some(b'-') if numeric => (b"-", &text[1..]),
        _ if numeric && plus => (b"+", text),
        _ => (b"", text),
    };
    // The width in its unit (owner, 2026-10-25); a fill is one unit wide.
    let taken = sign.len() + measure(rest, unit);
    let pad = (width as usize).saturating_sub(taken);
    let mut fill_bytes = [0u8; 4];
    let fill_len = crate::char_utf8(if fill == 0 { b' ' as u32 } else { fill }, &mut fill_bytes);
    let pad_with = |o: *mut Out, k: usize, b: &[u8]| {
        for _ in 0..k {
            put(o, b);
        }
    };
    if zero && numeric && align == 0 {
        put(o, sign);
        pad_with(o, pad, b"0");
        put(o, rest);
    } else {
        let (left, right) = match align {
            1 => (0, pad),
            2 => (pad / 2, pad - pad / 2),
            3 => (pad, 0),
            _ if numeric => (pad, 0),
            _ => (0, pad),
        };
        pad_with(o, left, &fill_bytes[..fill_len]);
        put(o, sign);
        put(o, rest);
        pad_with(o, right, &fill_bytes[..fill_len]);
    }
    crate::cyborg_free((*s).bytes);
    crate::cyborg_free(s as *mut u8);
}

/// How wide `text` is in a format width's unit (owner, 2026-10-25): 0
/// screen columns, 1 the same with East Asian Ambiguous characters 2, 2
/// graphemes, 3 points, 4 bytes.
fn measure(text: &[u8], unit: u32) -> usize {
    use unicode_segmentation::UnicodeSegmentation;
    let Ok(s) = core::str::from_utf8(text) else { return text.len() };
    match unit {
        2 => s.graphemes(true).count(),
        3 => s.chars().count(),
        4 => text.len(),
        _ => s.graphemes(true).map(|g| grapheme_cols(g, unit == 1)).sum(),
    }
}

/// A grapheme's screen columns: its widest character's, and 2 when U+FE0F
/// follows an emoji in it (it asks the emoji to be drawn as a picture).
fn grapheme_cols(g: &str, east_asian: bool) -> usize {
    use crate::unicode_width::EMOJI;
    let mut widest = 0;
    let mut after_emoji = false;
    for c in g.chars() {
        if c == '\u{FE0F}' && after_emoji {
            return 2;
        }
        after_emoji = within(&EMOJI, c as u32);
        widest = widest.max(char_cols(c as u32, east_asian));
    }
    widest
}

/// One character's screen columns: 0 for a mark, a format or control
/// character and the Hangul jamo that join the syllable before; 2 for an
/// East Asian Wide or Fullwidth one, or one drawn as an emoji; an East Asian
/// Ambiguous one 1, or 2 under `cols.east_asian`; 1 otherwise.
fn char_cols(c: u32, east_asian: bool) -> usize {
    use crate::unicode_width::{AMBIGUOUS, EMOJI_PRESENTATION, WIDE, ZERO};
    if within(&ZERO, c) {
        0
    } else if within(&WIDE, c) || within(&EMOJI_PRESENTATION, c) {
        2
    } else if east_asian && within(&AMBIGUOUS, c) {
        2
    } else {
        1
    }
}

fn within(t: &[(u32, u32)], c: u32) -> bool {
    t.binary_search_by(|&(a, b)| if c < a { core::cmp::Ordering::Greater } else if c > b { core::cmp::Ordering::Less } else { core::cmp::Ordering::Equal }).is_ok()
}

/// `std::format(…)`: the scratch buffer's bytes as a new text; the buffer
/// is freed.
///
/// # Safety
/// `out` a text to write, `s` a scratch buffer.
#[cfg(not(target_os = "none"))]
#[no_mangle]
pub unsafe extern "C" fn cyborg_fmt_text(out: *mut crate::Text, s: *mut Out) {
    let n = (*s).len as u64;
    let p = crate::cyborg_alloc(n.max(1));
    core::ptr::copy_nonoverlapping((*s).bytes, p, n as usize);
    (*out).ptr = p;
    (*out).len = n;
    crate::cyborg_free((*s).bytes);
    crate::cyborg_free(s as *mut u8);
}
