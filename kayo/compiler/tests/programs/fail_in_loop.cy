# A failing call in a loop, twenty million times. Its outcome had a stack
# slot made where the call was, which LLVM reads as a new allocation each
# trip: the stack grew until the program died (exit 139). Every slot is
# now made once, in the function's first block. The sum of 1 to 10^7 is
# 50000005000000; 0 mod 1000, and 7 is the exit.
%extern.C proc exit [let $code: i32];
fn half [let $n: i64] -> i64 or fail[odd]:
    if ($n mod.floor 2) == 1:
        fail odd;
    ;if
    ret $n div.floor 2;
;fn half
MAIN:
    var $acc: i64 = 0;
    for $i: i64 [from: 1, to: 20000000, by: 1]:
        let $h := half($i) ?? 0;
        change $acc to ($acc + $h) mod.floor 1000;
    ;for
    no_check my.exit(to.i32($acc mod.floor 100) + 7);
;MAIN
