# env CHK_A 300
# Finding 2026-09-23 (check claude, by hand): a narrowing site reported
# "checked at run time" has no check; 300 became 44.
extern.C fn write ['fd': int32, 'buf': ptr[uint8], 'len': uint64] -> int64;
extern.C fn getenv ['name': ptr[uint8]] -> ptr[uint8];
extern.C fn strlen ['s': ptr[uint8]] -> uint64;
MAIN {
    let 'k' := "CHK_A";
    no_check let 'n': uint64 = strlen[getenv[lend 'k']];
    let 'b': uint8 = 'n';
    let 'yes' := "b is 44";
    if 'b' equal_to 44 { no_check write[1, lend 'yes', 7]; }
}
