MAIN {
    let 'fd': int64 = 1;
    no_check inline.ASM {
        mov x16, #4
        svc #0x80
    } with ['fd' -> x0, 'n' <- x0, clobber x16];
}
