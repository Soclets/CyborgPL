# A deci whose power's coefficient is 1, to a negative power, is exact at
# the exponent one over it gives: 1 pow -1 is 1, not 1.0 (check claude,
# 2026-10-15, corpus ffe48e05). Each text as the oracle prints it counts
# one: 8 in all.
extern.C fn exit ['code': i32];

MAIN {
    let mut 'got': i32 = 0;
    let 'a': d32 = to.d32[1] pow (-1);
    if to.stored.str.utf8['a'] equal_to "1" { change 'got' = 'got' plus 1; }
    let 'b': d64 = to.d64[1] pow (-2);
    if to.stored.str.utf8['b'] equal_to "1" { change 'got' = 'got' plus 1; }
    let 'c': d32 = to.d32[-1] pow (-1);
    if to.stored.str.utf8['c'] equal_to "-1" { change 'got' = 'got' plus 1; }
    let 'h1': d32 = 0.01;
    let 'e': d32 = 'h1' pow (-1);
    if to.stored.str.utf8['e'] equal_to "100" { change 'got' = 'got' plus 1; }
    let 'g': d64 = to.d64[10] pow (-3);
    if to.stored.str.utf8['g'] equal_to "0.001" { change 'got' = 'got' plus 1; }
    let 'h': d64 = to.d64[4] pow (-1);
    if to.stored.str.utf8['h'] equal_to "0.25" { change 'got' = 'got' plus 1; }
    let 'h3': d64 = -0.1;
    let 'i': d64 = 'h3' pow (-3);
    if to.stored.str.utf8['i'] equal_to "-1000" { change 'got' = 'got' plus 1; }
    let 'j': d128 = to.d128[-1] pow (-7);
    if to.stored.str.utf8['j'] equal_to "-1" { change 'got' = 'got' plus 1; }
    no_check my.exit['got'];
}
