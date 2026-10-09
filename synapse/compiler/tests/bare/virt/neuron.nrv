# A board Neuron for QEMU's `virt` machine, for a program to name as its
# target's `output` and `exit` (owner, 2026-10-09: a function of the Neuron
# or of a board Neuron it uses): the serial port, a PL011 at 0x0900_0000,
# and power off through PSCI (SYSTEM_OFF, over `hvc`), which ends QEMU.
bare.never [fn];

pub module serial {
    pub fn write ['b': uint8] {
        no_check inline.ASM {
            mov x9, #0x09000000
            strb w0, [x9]
        } with ['b' -> x0, clobber x9];
    }

    # The other form an output may take: a chunk a call.
    pub fn write_all [lent 'bytes': array[uint8]] {
        for 'b' in lend 'bytes' {
            write['b'];
        }
    }
}

pub module power {
    pub fn off ['code': uint8] {
        no_check inline.ASM {
            movz x0, #0x0008
            movk x0, #0x8400, lsl #16
            hvc #0
        } with ['code' -> x1, clobber x0];
    }
}
