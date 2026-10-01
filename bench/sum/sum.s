// sum: Σ (i × i mod 7) for i = 1..N, then mod 1 000 000 007 once.
// Hand-written AArch64 for Apple Silicon (macOS, Mach-O, Apple arm64 ABI).
        .section __TEXT,__text,regular,pure_instructions
        .globl  _main
        .p2align 2
_main:
        stp     x29, x30, [sp, #-32]!
        mov     x29, sp
        stp     x19, x20, [sp, #16]

        adrp    x0, Lname@PAGE          // n = getenv("N") ? atoll(...) : -1
        add     x0, x0, Lname@PAGEOFF
        bl      _getenv
        cbz     x0, Lbad
        bl      _atoll
        mov     x19, x0                 // x19 = n
        cmp     x19, #0
        b.lt    Lbad
        movz    x9, #0xca00             // 1 000 000 000 = 0x3b9aca00
        movk    x9, #0x3b9a, lsl #16
        cmp     x19, x9
        b.gt    Lbad

        mov     x20, #0                 // acc
        mov     x10, #1                 // i
        mov     x11, #7
Lloop:
        cmp     x10, x19
        b.gt    Ldone
        mul     x12, x10, x10           // i * i
        udiv    x13, x12, x11
        msub    x12, x13, x11, x12      // (i * i) mod 7
        add     x20, x20, x12
        add     x10, x10, #1
        b       Lloop
Ldone:
        movz    x9, #0xca07             // 1 000 000 007 = 0x3b9aca07
        movk    x9, #0x3b9a, lsl #16
        udiv    x13, x20, x9
        msub    x0, x13, x9, x20        // acc mod 1 000 000 007
        and     w0, w0, #0xff
        b       Lret
Lbad:
        mov     w0, #2
Lret:
        ldp     x19, x20, [sp, #16]
        ldp     x29, x30, [sp], #32
        ret

        .section __TEXT,__cstring,cstring_literals
Lname:  .asciz  "N"
.subsections_via_symbols
