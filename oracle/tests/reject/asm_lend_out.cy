!MAIN:
    let $text := "hi";
    !no_check inline.ASM with [lend $text <- x1]:
        ldr x0, [x1]
    end of inline.ASM
!end of MAIN
