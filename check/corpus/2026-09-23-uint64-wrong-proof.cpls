# env CHK_A 2
# Finding 2026-09-23 (check claude, by hand): 'n' x 10^19 reported proven
# (layer 2); with n = 2 it overflows and CCA's binary runs on.
extern.C fn write ['fd': int32, 'buf': ptr[uint8], 'len': uint64] -> int64;
extern.C fn getenv ['name': ptr[uint8]] -> ptr[uint8];
extern.C fn strlen ['s': ptr[uint8]] -> uint64;
MAIN {
    let 'k' := "CHK_A";
    no_check let 'n': uint64 = strlen[getenv[lend 'k']];
    let 'm': uint64 = 'n' times 10000000000000000000;
    let 'small' := "m is smaller than n";
    if 'm' < 'n' { no_check write[1, lend 'small', 19]; }
}
