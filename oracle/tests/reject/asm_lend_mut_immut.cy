!MAIN:
    let $buf: str.utf8 := "hi";
    !no_check inline.ASM with [lend.mut $buf -> x1]:
        strb wzr, [x1]
    end of inline.ASM
!end of MAIN
