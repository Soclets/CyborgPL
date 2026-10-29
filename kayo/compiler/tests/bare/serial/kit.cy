# QEMU virt's serial port, a PL011 at 0x0900_0000: a byte written to its
# data register goes out. The target names it as `std::print`'s output,
# and `stop` as `std::exit`'s (owner, 2026-10-09).
module serial:
    pub.kit proc write [let $b: u8]:
        no_check inline.ASM with [$b -> x0, clobber x9]:
            mov x9, #0x09000000
            strb w0, [x9]
        ;inline.ASM
    ;proc write
    pub.kit fn stop [let $code: u8]:
    ;fn stop
;module serial
MAIN:
;MAIN
