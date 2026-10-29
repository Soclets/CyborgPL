# An argument that cannot pass its own narrowing never reaches the callee
# (check claude, 2026-09-29, seed 1027): the -1 stops at the call, which
# traps when it runs, and Precog no longer walks it into `half`, where it
# made the division look wrong every time.
%extern.C proc exit [let $code: i32];
fn half [let $n: u64] -> u64:
    ret ($n div.floor 10) + 1;
;fn half
MAIN:
    var $k: i32 = 2;
    var $total: u64 = 0;
    while $k > 0:
        change $k to $k - 1;
        if $k == 0:
            change $total to half(0 - 1);
        ;if
    ;while
    let $c: i32 := to.i32($total);
    no_check my.exit($c);
;MAIN
