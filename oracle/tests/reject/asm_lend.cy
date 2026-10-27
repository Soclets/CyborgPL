# A lend satisfies a pointer in an asm binding (owner, 2026-09-30: a): it checks, then the oracle
# refuses to run assembly, as for any asm block.
!MAIN:
    let $text := "Hello, world!" concatenate newline;
    let $fd: integer.signed.64_bit = 1;
    let $len: integer.signed.64_bit = 14;
    !no_check inline.ASM with [$fd -> x0, lend $text -> x1, $len -> x2, $n <- x0, clobber x16]:
        mov x16, #4
        svc #0x80
    end of inline.ASM
!end of MAIN
