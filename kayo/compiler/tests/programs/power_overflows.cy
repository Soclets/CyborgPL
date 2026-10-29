# 3 xx 50 is past any 64-bit type: the check in between stops it.
%extern.C proc exit [let $code: i32];
%extern.C proc strlen [let $s: ptr[u8]] -> u64;
MAIN:
    let $s := "ab";
    no_check let $two: i64 := to.i64(strlen(lend $s));
    let $p: i64 := ($two + 1) std::pow ($two x 25);
    no_check my.exit(to.i32($p mod.floor 256));
;MAIN
