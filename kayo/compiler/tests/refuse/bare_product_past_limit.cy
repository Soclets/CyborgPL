# refuse: the value can be outside what an `integer` holds
# A group of bare numbers beside a typed operand is worked at its own size,
# and its value is in between: held to the 64-bit limit where it is used
# (check claude, seed 36150; the oracle stops at the sum).
fn v [] -> u64:
    ret 5;
;fn v
MAIN:
    let $x: u64 := ((6729433359276983393 x 17924805223061868988) + v()) mod.floor 7;
;MAIN
