# The both-ways binding, `<->` (owner, 2026-10-07: the arrows are short
# again; the other eight spellings are old ones), nine times; each adds 1 in
# x0, so 1 becomes 10.
%extern.C proc exit [$code: i32];
MAIN:
    var $v: i64 = 1;
    no_check inline.ASM with [$v <-> x0]:
        add x0, x0, #1
    ;inline.ASM
    no_check inline.ASM with [$v <-> x0]:
        add x0, x0, #1
    ;inline.ASM
    no_check inline.ASM with [$v <-> x0]:
        add x0, x0, #1
    ;inline.ASM
    no_check inline.ASM with [$v <-> x0]:
        add x0, x0, #1
    ;inline.ASM
    no_check inline.ASM with [$v <-> x0]:
        add x0, x0, #1
    ;inline.ASM
    no_check inline.ASM with [$v <-> x0]:
        add x0, x0, #1
    ;inline.ASM
    no_check inline.ASM with [$v <-> x0]:
        add x0, x0, #1
    ;inline.ASM
    no_check inline.ASM with [$v <-> x0]:
        add x0, x0, #1
    ;inline.ASM
    no_check inline.ASM with [$v <-> x0]:
        add x0, x0, #1
    ;inline.ASM
    no_check my.exit(to.i32($v));
;MAIN
