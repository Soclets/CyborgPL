// fib: naive recursive Fibonacci.
// Hand-written AArch64 for Apple Silicon (macOS, Mach-O, Apple arm64 ABI).
        .section __TEXT,__text,regular,pure_instructions
        .p2align 2
// long fib(long n): n < 2 ? n : fib(n - 1) + fib(n - 2)
_fib:
        cmp     x0, #2
        b.lt    Lfib_base
        stp     x29, x30, [sp, #-32]!
        mov     x29, sp
        stp     x19, x20, [sp, #16]
        mov     x19, x0                 // keep n
        sub     x0, x19, #1
        bl      _fib
        mov     x20, x0                 // fib(n - 1)
        sub     x0, x19, #2
        bl      _fib
        add     x0, x0, x20
        ldp     x19, x20, [sp, #16]
        ldp     x29, x30, [sp], #32
Lfib_base:
        ret

        .globl  _main
        .p2align 2
_main:
        stp     x29, x30, [sp, #-16]!
        mov     x29, sp

        adrp    x0, Lname@PAGE          // n = getenv("N") ? atoll(...) : -1
        add     x0, x0, Lname@PAGEOFF
        bl      _getenv
        cbz     x0, Lbad
        bl      _atoll
        cmp     x0, #0
        b.lt    Lbad
        cmp     x0, #92
        b.gt    Lbad
        bl      _fib
        and     w0, w0, #0xff           // fib(n) mod 256
        b       Lret
Lbad:
        mov     w0, #2
Lret:
        ldp     x29, x30, [sp], #16
        ret

        .section __TEXT,__cstring,cstring_literals
Lname:  .asciz  "N"
.subsections_via_symbols
