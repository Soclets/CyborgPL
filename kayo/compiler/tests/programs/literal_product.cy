# Two bare numbers multiplied into a u64: an unsigned 64-bit multiply,
# in the ordinary build (folded) and with proofs asserted (computed).
%extern.C proc exit [$code: i32];
fn put_u [$n: u64] -> u64:
    ret $n;
;fn put_u
fn f3 [$p1: u32] -> u16:
    var $w6: i32 = 1;
    var $got: u64 = 0;
    while $w6 > 0:
        change $w6 to 0;
        change $got to put_u(2 x 7726793855274730819);
    ;while
    if $got == 15453587710549461638:
        ret 7;
    ;if
    ret 0;
;fn f3
MAIN:
    no_check my.exit(to.i32(f3(0)));
;MAIN
