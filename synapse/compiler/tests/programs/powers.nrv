# Powers at run time: by squaring, each multiply checked only when the
# answer needs it; xx~ wraps at its values' size. Each true case adds a bit.
extern.C fn exit ['code': int32];
extern.C fn strlen ['s': ptr[uint8]] -> uint64;
MAIN {
    let 's' := "ab";
    no_check let 'two': int64 = to.int64[strlen[lend 's']];
    let mut 'got': int64 = 0;
    if 'two' pow 10 equal_to 1024 { change 'got' = 'got' plus 1; }
    if ('two' plus 1) pow 0 equal_to 1 { change 'got' = 'got' plus 2; }
    if (0 minus 'two') pow 3 equal_to -8 { change 'got' = 'got' plus 4; }
    let 'w': uint8 = to.uint8['two' plus 1];
    let 'e5': uint8 = to.uint8['two' plus 3];
    if 'w' pow.wrap 'e5' equal_to 243 { change 'got' = 'got' plus 8; }
    let 'e6': uint8 = to.uint8['two' plus 4];
    if 'w' pow.wrap 'e6' equal_to 217 { change 'got' = 'got' plus 16; }
    no_check my.exit[to.int32['got']];
}
