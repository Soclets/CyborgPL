# `lend.mut 'buf' -> x1` puts the address of the list's elements in x1,
# as a lend does at an extern call (owner, 2026-09-30: a); the assembly
# writes 42 over the first, and the program exits with it.
%extern.C proc exit [let $code: i32];
MAIN:
    var $buf: std::ls[u8] = [7, 8,];
    no_check inline.ASM with [lend.mut $buf -> x1, clobber x2]:
        mov w2, #42
        strb w2, [x1]
    ;inline.ASM
    no_check my.exit(to.i32($buf[pos: 1]));
;MAIN
