# A check that fails with no OS freezes the CPU on a trap, `brk` (owner,
# 2026-10-08: "yea, freeze there").
fn down [let $n: u8] -> u8:
    ret $n - 1;
;fn down

MAIN:
    var $k: u8 = 3;
    while true:
        change $k to down($k);
    ;while
;MAIN
