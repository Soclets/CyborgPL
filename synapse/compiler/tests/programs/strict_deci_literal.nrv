# A deci literal into a deci range is decided from its digits (check
# claude, seed 8049): 0.79 is inside [0.0, 1.0]. Precog follows no deci
# value, so a strict file refused it; now it is proven, and 3 comes back.
extern.C fn exit ['code': int32];
fn f [] -> int32 {
    let immut 'd': deci64[min 0.0, max 1.0] = 0.79;
    return 3;
}
MAIN {
    let immut 'x': int32 = f[];
    no_check my.exit['x'];
}
