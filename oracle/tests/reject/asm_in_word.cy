!MAIN:
    let $fd := 1;
    !no_check inline.ASM with [in $fd -> x0]:
        nop
    end of inline.ASM
!end of MAIN
