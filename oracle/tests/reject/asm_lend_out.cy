MAIN:
    let $text := "hi";
    no_check inline.ASM with [lend $text <- x1]:
        ldr x0, [x1]
    ;inline.ASM
;MAIN
