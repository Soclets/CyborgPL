# `lend.mut 'x' -> x1` hands the assembly the address of 'x' to write
# through (owner, 2026-09-30: a). 'x' was 1 and is 100 after it, so
# `'x' x 3` is 300, which no u8 holds: the program stops. Precog must
# not keep what it knew of 'x' before the assembly.
%extern.C proc exit [let $code: i32];
MAIN:
    var $x: i64 = 1;
    no_check inline.ASM with [lend.mut $x -> x1, clobber x2]:
        mov x2, #100
        str x2, [x1]
    ;inline.ASM
    let $y: u8 := to.u8($x x 3);
    no_check my.exit(to.i32($y));
;MAIN
