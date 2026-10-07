# A deci divided by zero stops the program (the oracle: "division by
# zero"): there is no infinity to give.
extern.C fn exit ['code': int32];
fn half ['d': deci64] -> deci64 {
    return 1.5 div.near 'd';
}
MAIN {
    let 'z': deci64 = 0.00;
    let 'h' := half['z'];
    no_check my.exit[7];
}
