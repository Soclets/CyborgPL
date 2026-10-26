# A deci divided by zero stops the program (the oracle: "division by
# zero"): there is no infinity to give.
%extern.C fn exit [$code: i32];
fn half [$d: %d64] -> %d64 {
    return 1.5 div.near $d;
}
MAIN {
    let $z: %d64 = 0.00;
    let $h := half($z);
    no_check my.exit(7);
}
