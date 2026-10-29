# not.bit on an exact answer flips at the size of its values: 'v' + 1 with
# 'v' a u32 flips 32 bits (examples/widths.cpls), 4294967291. On an i8
# it is -x - 1, which layer 2 now knows as a range.
%extern.C proc exit [let $code: i32];
MAIN:
    let $v: u32 := 3;
    let $c: u32 := not.bit ($v + 1);
    let $x: i8 := -102;
    let $y: i8 := not.bit $x;
    no_check my.exit(to.i32(($c mod.floor 256) + to.u32($y)));
;MAIN
