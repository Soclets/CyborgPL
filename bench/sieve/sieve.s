// sieve: count the primes up to N (sieve of Eratosthenes).
// Hand-written AArch64 for Apple Silicon (macOS, Mach-O, Apple arm64 ABI).
// One byte per number, from calloc.
        .section __TEXT,__text,regular,pure_instructions
        .globl  _main
        .p2align 2
_main:
        stp     x29, x30, [sp, #-48]!
        mov     x29, sp
        stp     x19, x20, [sp, #16]
        stp     x21, x22, [sp, #32]

        adrp    x0, Lname@PAGE          // n = getenv("N") ? atoll(...) : -1
        add     x0, x0, Lname@PAGEOFF
        bl      _getenv
        cbz     x0, Lbad
        bl      _atoll
        mov     x19, x0                 // x19 = n
        cmp     x19, #0
        b.lt    Lbad
        movz    x9, #0xc200             // 200 000 000 = 0x0bebc200
        movk    x9, #0x0beb, lsl #16
        cmp     x19, x9
        b.gt    Lbad

        add     x0, x19, #1             // comp = calloc(n + 1, 1)
        mov     x1, #1
        bl      _calloc
        mov     x20, x0
        mov     x21, #0                 // count
        mov     x22, #2                 // p
        mov     w13, #1
Lp:
        cmp     x22, x19
        b.gt    Ldone
        ldrb    w9, [x20, x22]
        cbnz    w9, Lnextp              // composite: skip
        add     x21, x21, #1            // count++
        mul     x10, x22, x22           // m = p * p
Lm:
        cmp     x10, x19
        b.gt    Lnextp
        strb    w13, [x20, x10]         // comp[m] = true
        add     x10, x10, x22
        b       Lm
Lnextp:
        add     x22, x22, #1
        b       Lp
Ldone:
        mov     x0, x20
        bl      _free
        and     w0, w21, #0xff          // count mod 256
        b       Lret
Lbad:
        mov     w0, #2
Lret:
        ldp     x21, x22, [sp, #32]
        ldp     x19, x20, [sp, #16]
        ldp     x29, x30, [sp], #48
        ret

        .section __TEXT,__cstring,cstring_literals
Lname:  .asciz  "N"
.subsections_via_symbols
