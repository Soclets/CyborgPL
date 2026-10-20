# Sums over a `range` with a constant step: Precog bounds each total by its
# start plus the trips times the largest term (a bin's with room for its
# rounding), so both adds are proven and both loops run in slices.
# 300000 x 0.5 = 150000, and the i x i mod 7 terms total 599999: 600149 mod 256 = 85.
extern.C fn exit ['code': i32];
MAIN {
    let 'n': i64 = 300000;
    let mut 'half': f64 = 0.0;
    for 'i': i64 [from: 1, to: 'n', by: 1] {
        change 'half' = 'half' plus 0.5;
    }
    let mut 'acc': i64 = 0;
    for 'i': i64 [from: 1, to: 'n', by: 1] {
        change 'acc' = 'acc' plus (('i' times 'i') mod.floor 7);
    }
    let 'h': i32 = to.i32['half' div.near 1000.0] otherwise 1;
    no_check my.exit[to.i32[('acc' plus to.i64['h']) mod.floor 256]];
}
