# A `check` that holds, inside two loops (check claude, seed 316): 'v2' is
# -102 to -96 on every trip, so `not.bit 'v2'` is 95 to 101 and never zero.
# Layer 2 had no range for `not.bit`, and widened at every busy block, so
# the loops lost the interval and the build refused the `check`. The
# quotient is exact now (owner, 2026-10-02) and a shift of an exact answer
# is open, so it is an i8 first, as it was.
%extern.C proc exit [$code: i32];
%extern.C proc strlen [$s: ptr[u8]] -> u64;
MAIN:
    let $s := "seven!!";
    no_check let $n: u64 := strlen(lend $s);
    var $v2: i8 = (to.i8($n mod.floor 100) mod.floor 7) - 102;
    var $w3: i16 = 7;
    while $w3 > 0:
        change $w3 to $w3 - 1;
        for $i4: u16 [from: 29, to: 3, by: -3]:
            check change $v2 to ((to.i8((126 or.bit $v2) div.floor (not.bit $v2)) shl 6) mod.floor 7) - 102;
        ;for
    ;while
    no_check my.exit(to.i32(0 - $v2));
;MAIN
