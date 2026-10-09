# Plain math is exact, checked where it lands (owner, 2026-09-23): the
# cases of examples/overflow_sites.cpls, each adding its bit to the exit.
# The last line also pins a wrapping answer kept at its own width when
# stored wider: layer 1 used to fold 8000000000 for 'h'.
extern.C fn exit ['code': int32];
extern.C fn strlen ['s': ptr[uint8]] -> uint64;
MAIN {
    let 'z' := "ab";
    no_check let 'two': int64 = to.int64[strlen[lend 'z']];
    let 'small': int16 = to.int16['two'] plus 3;
    let 'big': int64 = (20000 plus 20000) plus 'small';
    let 's': int16 = 30000;
    let 'twice': int64 = 's' plus 's';
    let 'tiny': int8 = (100 plus 100) minus 150;
    let 'v': int8 = 112;
    let 'u': uint8 = 5;
    let 'below' := 'u' minus 10;
    let 'x': uint32 = 4000000000;
    let 'h': uint64 = 'x' plus.wrap 'x';
    let mut 'got': int64 = 0;
    if 'big' equal_to 40005 { change 'got' = 'got' plus 1; }
    if 'twice' equal_to 60000 { change 'got' = 'got' plus 2; }
    if 'tiny' equal_to 50 { change 'got' = 'got' plus 4; }
    if not ('v' > ('v' plus 'v')) { change 'got' = 'got' plus 8; }
    if 'below' equal_to -5 { change 'got' = 'got' plus 16; }
    if 'h' equal_to 3705032704 { change 'got' = 'got' plus 32; }
    if 'v' < 200 { change 'got' = 'got' plus 64; }
    no_check my.exit[to.int32['got']];
}
