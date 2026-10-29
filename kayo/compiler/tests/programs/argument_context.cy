# An argument takes its parameter's type as its context, as a declaration's
# value does (owner, 2026-09-29: a): show[0 -~ 1] wraps at 8 bits to 255,
# wide[2 shl 3] is a 32-bit shift (16), wide[not.bit 0] is 4294967295, and
# list.append's value takes the element type (0 -~ 2 is 254).
# examples/argument_context.cpls as an exit code: one bit per right answer.
%extern.C proc exit [$code: i32];
fn show [$b: u8] -> u8:
    ret $b;
;fn show
fn wide [$w: u32] -> u32:
    ret $w;
;fn wide
fn flag [$b: bool, $v: i32] -> i32:
    if $b:
        ret $v;
    ;if
    ret 0;
;fn flag
MAIN:
    let $a := show(0 minus.wrap 1);
    let $v: u64 := to.u64(wide(2 shl 3));
    let $n := wide(not.bit 0);
    var $xs: std::ls[u8] = [1,];
    std::ls.append(lend.mut $xs, value: 0 minus.wrap 2);
    no_check my.exit(flag($a == 255, 1) + flag($v == 16, 2) + flag($n == 4294967295, 4) + flag($xs[pos: 2] == 254, 8));
;MAIN
