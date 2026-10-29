# A board Neuron for QEMU's `virt` machine, for a program to name as its
# target's `output` and `exit` (owner, 2026-10-09: a function of the Neuron
# or of a board Neuron it uses): the serial port, a PL011 at 0x0900_0000,
# and power off through PSCI (SYSTEM_OFF, over `hvc`), which ends QEMU.

pub module serial:
    pub proc write [let $b: u8]:
        no_check inline.ASM with [$b -> x0, clobber x9]:
            mov x9, #0x09000000
            strb w0, [x9]
        ;inline.ASM
    ;proc write

    # The other form an output may take: a chunk a call.
    pub proc write_all [lent $bytes: arr[u8]]:
        for $b in lend $bytes:
            write($b);
        ;for
    ;proc write_all
;module serial

pub module std::pow:
    pub proc off [let $code: u8]:
        no_check inline.ASM with [$code -> x1, clobber x0]:
            movz x0, #0x0008
            movk x0, #0x8400, lsl #16
            hvc #0
        ;inline.ASM
    ;proc off
;module std::pow
