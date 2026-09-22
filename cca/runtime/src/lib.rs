//! The runtime linked into every CyborgPL binary. `no_std`: it is the
//! floor under the language, so it stands on the OS and nothing else.
//!
//! Today it holds one thing — what happens when a check fires. In the
//! working build an unproven site keeps its check, and a check that fires
//! stops the program and says where (`core-features.md`: "the program runs
//! and fails loudly where a guardrail is missing"; `diagnostics.md`: "the
//! trap at run time names the site").
#![no_std]

extern "C" {
    fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    fn exit(code: i32) -> !;
    fn malloc(bytes: usize) -> *mut u8;
    fn realloc(p: *mut u8, bytes: usize) -> *mut u8;
    fn free(p: *mut u8);
    fn memcpy(dst: *mut u8, src: *const u8, bytes: usize) -> *mut u8;
    fn memmove(dst: *mut u8, src: *const u8, bytes: usize) -> *mut u8;
}

/// A check fired. `msg` is the site's own words, built at compile time, so
/// nothing has to be formatted here.
///
/// # Safety
/// `msg` must point at `len` readable bytes. The compiler only ever passes
/// a constant it emitted itself.
#[no_mangle]
pub unsafe extern "C" fn cyborg_check_failed(msg: *const u8, len: usize) -> ! {
    write(2, msg, len);
    exit(101)
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { exit(102) }
}


// ----------------------------------------------------------------- memory
//
// `ffi.md`: the language has no raw-allocation primitive — "bytes you can
// name are a `list[uint8]`". So allocating is the runtime's job, and it
// stands on the C allocator the platform already has rather than carrying
// one of its own. A list, a map and a built text all come through here.

/// Out of memory is not a check firing and not a failure the program can
/// handle: there is nothing to hand back a failure *in*. It stops, saying
/// so.
unsafe fn out_of_memory() -> ! {
    let msg = b"out of memory\n";
    write(2, msg.as_ptr(), msg.len());
    exit(103)
}

/// `bytes` of memory, or the program stops. Zero bytes is a null pointer:
/// an empty list owns nothing, so there is nothing to free.
#[no_mangle]
pub unsafe extern "C" fn cyborg_alloc(bytes: u64) -> *mut u8 {
    if bytes == 0 {
        return core::ptr::null_mut();
    }
    let p = malloc(bytes as usize);
    if p.is_null() {
        out_of_memory();
    }
    p
}

/// Grow (or shrink) an allocation, keeping what was in it.
#[no_mangle]
pub unsafe extern "C" fn cyborg_realloc(p: *mut u8, bytes: u64) -> *mut u8 {
    if bytes == 0 {
        cyborg_free(p);
        return core::ptr::null_mut();
    }
    let q = if p.is_null() { malloc(bytes as usize) } else { realloc(p, bytes as usize) };
    if q.is_null() {
        out_of_memory();
    }
    q
}

/// Give memory back. A null pointer is an empty value and is not an error.
#[no_mangle]
pub unsafe extern "C" fn cyborg_free(p: *mut u8) {
    if !p.is_null() {
        free(p);
    }
}

/// Copy bytes. The regions never overlap: every caller is building a fresh
/// allocation.
#[no_mangle]
pub unsafe extern "C" fn cyborg_copy(dst: *mut u8, src: *const u8, bytes: u64) {
    if bytes != 0 {
        memcpy(dst, src, bytes as usize);
    }
}

/// Move bytes that may overlap: `list.remove` slides the tail of a list
/// down over the element that left.
#[no_mangle]
pub unsafe extern "C" fn cyborg_move(dst: *mut u8, src: *const u8, bytes: u64) {
    if bytes != 0 {
        memmove(dst, src, bytes as usize);
    }
}
