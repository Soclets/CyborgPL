# A call's answer of a range type, checked where it was made: the caller
# tells LLVM its range (owner, 2026-09-29: c), so a loop bounded by it is
# worked out at 64 bits. `pick` gives 3 here (no SYNAPSE_NO_SUCH_VARIABLE),
# and the sum of 1 to 3 is 6.
%extern.C proc exit [let $code: i32];
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
proc pick [] -> i64[min: 0, max: 1000] or fail[missing]:
    let $key := "SYNAPSE_NO_SUCH_VARIABLE";
    no_check let $p := getenv(lend $key);
    if ptr.is_null($p):
        ret 3;
    ;if
    fail missing;
;proc pick
MAIN:
    let $n := pick() ?? 0;
    var $sum: i64 = 0;
    var $i: i64 = 1;
    while $i <= $n:
        change $sum to $sum + $i;
        change $i to $i + 1;
    ;while
    no_check my.exit(to.i32($sum));
;MAIN
