/* QEMU's `virt` machine: RAM from 0x4000_0000 (128 MiB by default); the
   program from 0x4008_0000, past the device tree QEMU puts at the start,
   then zeroed memory and a 64 KiB stack. A kit names its own script on the
   target (owner, 2026-10-09: "`c` now"); kayo's entry, `_start`, reads
   `__stack_top`, `__bss_start` and `__bss_end`. */
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
        __stack_top = .;
    }
    /DISCARD/ : { *(.comment) *(.note .note.*) *(.eh_frame) }
}
