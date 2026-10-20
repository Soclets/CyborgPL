# A whole bin or deci too big for the integer fails `out_of_range`, beside
# `not_whole` (owner, 2026-09-30: a): one `otherwise` handles both, as the
# oracle's examples/too_big_to_int.cpls prints "-1 -2 42 99". 42, then 100
# for -1, 50 for -2 and 20 for 99: 212.
extern.C fn exit ['code': i32];
MAIN {
    let 'b': f64 = 10000000000.0;
    let 'd': d128 = 99999999999999999999.0;
    let 'small': d64 = 42.0;
    let 'x' := to.i32['b'] otherwise -1;
    let 'y' := to.i64['d' times 1000] otherwise -2;
    let 'z' := to.i64['small'] otherwise -3;
    let 'w' := to.i32['d'] otherwise 99;
    let mut 'code': i64 = 'z';
    if 'x' equal_to -1 { change 'code' = 'code' plus 100; }
    if 'y' equal_to -2 { change 'code' = 'code' plus 50; }
    if 'w' equal_to 99 { change 'code' = 'code' plus 20; }
    no_check my.exit[to.i32['code']];
}
