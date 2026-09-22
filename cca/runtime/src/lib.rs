//! The runtime linked into every CyborgPL binary. `no_std`; nothing here yet.
#![no_std]

#[cfg(not(test))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
