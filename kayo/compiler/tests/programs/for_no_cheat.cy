# `for !cheat` (owner, 2026-10-01): every round really runs, in
# order, or in slices under `%para`, and gives what the loop in order gives.
# 1 + … + 300000 = 45000150000, mod 256 = 240; 1 + … + 10 = 55; 295 exits
# as 39, as the oracle gives.
%extern.C proc exit [let $code: i32];
MAIN:
    var $acc: i64 = 0;
    no_skip for $i: i64 [from: 1, to: 300_000, by: 1]:
        change $acc to $acc + $i;
    ;for
    var $more: i64 = 0;
    no_skip for $i: i64 [from: 1, to: 10, by: 1]:
        change $more to $more + $i;
    ;for
    no_check my.exit(to.i32(($acc mod.floor 256) + $more));
;MAIN
