# A shift on an exact answer works at the width of its typed operands (owner,
# 2026-09-23, confirmed 2026-10-03): ('v' plus 1) shl 'k' shifts as 32 bits.
# Worked at 65 bits and then cut, a shift past 32 bits went unseen. 'v' is 3
# and 'k' 29 at run time: (3 mod 16) shl 2 is 12, 4 shl 29 is 2^31 and shr 24
# of it 128, 4 shr 1 is 2: 12 + 128 + 2 = 142.
%extern.C proc exit [let $code: i32];
%extern.C proc strlen [let $s: ptr[u8]] -> u64;
MAIN:
    let $s := "xyz";
    no_check let $n: u64 := strlen(lend $s);
    let $v: u32 := to.u32($n);
    let $k: i32 := to.i32($n) + 26;
    let $b: u32 := ($v mod.floor 16) shl 2;
    let $c: u32 := ($v + 1) shl $k;
    let $d: u32 := ($v + 1) shr 1;
    no_check my.exit(to.i32(($b + ($c shr 24) + $d) mod.floor 256));
;MAIN
