/* QEMU's `virt` machine: RAM from 0x4000_0000 (128 MiB by default); the
   program from 0x4008_0000, past the device tree QEMU puts at the start,
   then zeroed memory and a 64 KiB stack. A Neuron's own script (owner,
   2026-10-09: "`c` now"). This one leaves out `__stack_top`, which
   Synapse's entry needs: refused, naming it. */
ENTRY(_start)
SECTIONS {
    . = 0x40080000;
    .text : { KEEP(*(.text._start)) *(.text .text.*) }
    .rodata : ALIGN(16) { *(.rodata .rodata.*) }
    .data : ALIGN(16) { *(.data .data.*) }
    .bss (NOLOAD) : ALIGN(16) {
        __bss_start = .;
        *(.bss .bss.* COMMON)
        . = ALIGN(16);
        __bss_end = .;
    }
    .stack (NOLOAD) : ALIGN(16) {
        . += 0x10000;
    }
    /DISCARD/ : { *(.comment) *(.note .note.*) *(.eh_frame) }
}
