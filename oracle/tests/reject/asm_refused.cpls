MAIN {
    let 'fd' := 1;
    no_check inline.ASM {
        mov x16, #4
        svc 0
    } with ['fd' -> x0, 'n' <- x0, clobber x16];
}
