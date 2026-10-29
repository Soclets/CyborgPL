# A deci divided by zero stops the program (the oracle: "division by
# zero"): there is no infinity to give.
%extern.C proc exit [$code: i32];
fn half [$d: %d64] -> %d64:
    ret 1.5 div.near $d;
;fn half
MAIN:
    let $z: %d64 := 0.00;
    let $h := half($z);
    no_check my.exit(7);
;MAIN
