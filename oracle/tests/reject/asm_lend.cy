# A lend satisfies a pointer in an asm binding (owner, 2026-09-30: a): it checks, then the oracle
# refuses to run assembly, as for any asm block.
MAIN {
    let $text := "Hello, world!" concat newln;
    let $fd: i64 = 1;
    let $len: i64 = 14;
    no_check inline.ASM {
        mov x16, #4
        svc #0x80
    } with [$fd -> x0, lend $text -> x1, $len -> x2, $n <- x0, clobber x16];
}
