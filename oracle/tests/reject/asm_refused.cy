!MAIN:
    let $fd := 1;
    !no_check inline.ASM with [$fd -> x0, $n <- x0, clobber x16]:
        mov x16, #4
        svc 0
    end of inline.ASM
!end of MAIN
