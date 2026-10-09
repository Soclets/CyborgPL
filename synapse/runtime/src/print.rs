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
    /// The target's output, one `uint8` a call.
    Byte = 1,
    /// The target's output, a `lent array[uint8]` a call: its address and
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

extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
}

/// A lent `array[uint8]` as a function receives one.
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
        0 => {
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
