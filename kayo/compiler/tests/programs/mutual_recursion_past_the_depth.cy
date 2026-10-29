# Two functions calling each other past the call depth: the walk standing
# for the deeper calls walks each one on its own terms, so `od`'s divisor
# keeps the 0 its last call gives, and the check stays (check claude,
# 2026-10-17, corpus ab280a72: proven never zero from {2, 4, 6, 8}).
%extern.C proc exit [$code: i32];

fn ev [$n: u32[min: 0, max: 23], $acc: i64] -> i64:
    if $n == 0:
        ret $acc;
    ;if
    ret od($n - 1, ($acc + 20) mod.floor 3710);
;fn ev

fn od [$n: u32[min: 0, max: 23], $acc: i64] -> i64:
    if $n == 0:
        ret $acc;
    ;if
    ret ev($n - 1, ($acc x 3) mod.floor ($n - 1));
;fn od

MAIN:
    no_check my.exit(to.i32(ev(10, 50) mod.floor 100));
;MAIN
