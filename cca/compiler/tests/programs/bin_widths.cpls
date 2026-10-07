# bin16/bin32 math rounds after every step, and a bare literal takes the
# typed side's width (owner, 2026-09-24: a): 2^24 + 1.0 at bin32 is 2^24.
# With a bin64 it is done at the wider (the oracle's pick). A literal in a
# comparison stays at 64 bits: a bin32 0.1 is a little more than 0.1.
extern.C fn exit ['code': int32];
MAIN {
    let 'x': bin32 = 16777216.0;
    let mut 'got': int32 = 0;
    let 'a' := 'x' plus 1.0;
    let 'na' := to.int32['a' minus 16777216.0] otherwise 99;
    if 'na' equal_to 0 { change 'got' = 'got' plus 1; }
    let 'b': bin32 = 'x' plus 1.0 plus 1.0;
    let 'nb' := to.int32['b' minus 16777216.0] otherwise 99;
    let 'w': bin64 = 1.0;
    let 'nw' := to.int32['x' plus 'w' minus 16777216.0] otherwise 99;
    if 'nb' equal_to 0 and 'nw' equal_to 1 { change 'got' = 'got' plus 2; }
    let 'c': bin32 = 0.1;
    if 'c' > 0.1 and not ('c' equal_to 0.1) { change 'got' = 'got' plus 4; }
    let 'd' := 'c' times 10.0;
    if 'd' > 0.99 and 'd' < 1.01 { change 'got' = 'got' plus 8; }
    let mut 'h': int32 = 0;
    check if 'x' > 0.0 { change 'h' = to.int32['x' div.near 2.0] otherwise 0; }
    if 'h' equal_to 8388608 { change 'got' = 'got' plus 16; }
    no_check my.exit['got'];
}
