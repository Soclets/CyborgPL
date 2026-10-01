// lists: append N numbers to a growable list, then sum it 10 times.
// Hand-written AArch64 for Apple Silicon (macOS, Mach-O, Apple arm64 ABI).
// The list is a buffer grown with realloc, doubling from 4.
        .section __TEXT,__text,regular,pure_instructions
        .globl  _main
        .p2align 2
_main:
        stp     x29, x30, [sp, #-64]!
        mov     x29, sp
        stp     x19, x20, [sp, #16]
        stp     x21, x22, [sp, #32]
        stp     x23, x24, [sp, #48]

        adrp    x0, Lname@PAGE          // n = getenv("N") ? atoll(...) : -1
        add     x0, x0, Lname@PAGEOFF
        bl      _getenv
        cbz     x0, Lbad
        bl      _atoll
        mov     x19, x0                 // x19 = n
        cmp     x19, #0
        b.lt    Lbad
        movz    x9, #0xe100             // 100 000 000 = 0x05f5e100
        movk    x9, #0x05f5, lsl #16
        cmp     x19, x9
        b.gt    Lbad

        mov     x20, #0                 // xs = NULL
        mov     x21, #0                 // len
        mov     x22, #0                 // cap
        mov     x23, #1                 // i
        mov     x24, #1000
Lfill:
        cmp     x23, x19
        b.gt    Lfilled
        cmp     x21, x22
        b.ne    Lstore
        lsl     x22, x22, #1            // cap = cap ? cap * 2 : 4
        cbnz    x22, 1f
        mov     x22, #4
1:      mov     x0, x20                 // xs = realloc(xs, cap * 8)
        lsl     x1, x22, #3
        bl      _realloc
        mov     x20, x0
Lstore:
        udiv    x9, x23, x24
        msub    x9, x9, x24, x23        // i mod 1000
        str     x9, [x20, x21, lsl #3]  // xs[len++] = ...
        add     x21, x21, #1
        add     x23, x23, #1
        b       Lfill
Lfilled:
        mov     x10, #0                 // acc
        mov     x11, #0                 // r
Lround:
        cmp     x11, #10
        b.ge    Lsummed
        mov     x12, #0                 // j
Lsum:
        cmp     x12, x21
        b.ge    Lnext
        ldr     x9, [x20, x12, lsl #3]
        add     x10, x10, x9
        add     x12, x12, #1
        b       Lsum
Lnext:
        add     x11, x11, #1
        b       Lround
Lsummed:
        mov     x19, x10                // keep acc across free
        mov     x0, x20
        bl      _free
        and     w0, w19, #0xff          // acc mod 256
        b       Lret
Lbad:
        mov     w0, #2
Lret:
        ldp     x23, x24, [sp, #48]
        ldp     x21, x22, [sp, #32]
        ldp     x19, x20, [sp, #16]
        ldp     x29, x30, [sp], #64
        ret

        .section __TEXT,__cstring,cstring_literals
Lname:  .asciz  "N"
.subsections_via_symbols
