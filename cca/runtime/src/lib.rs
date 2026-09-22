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
