# refuse: the value can be outside what an `integer` holds
# A group of bare numbers beside a typed operand is worked at its own size,
# and its value is in between: held to the 64-bit limit where it is used
# (check claude, seed 36150; the oracle stops at the sum).
fn v [] -> uint64 { return 5; }
MAIN {
    let 'x': uint64 = ((6729433359276983393 times 17924805223061868988) plus v[]) mod.floor 7;
}
