# env CHK_A 300
# Finding 2026-09-23 (check claude, by hand): passing a uint64 to a uint8
# parameter is not counted as a site at all.
extern.C fn write ['fd': int32, 'buf': ptr[uint8], 'len': uint64] -> int64;
extern.C fn getenv ['name': ptr[uint8]] -> ptr[uint8];
extern.C fn strlen ['s': ptr[uint8]] -> uint64;
fn g ['v': uint8] -> uint8 { return 'v'; }
MAIN {
    let 'k' := "CHK_A";
    no_check let 'n': uint64 = strlen[getenv[lend 'k']];
    let 'b': uint8 = g['n'];
    let 'hi' := "ran on";
    no_check write[1, lend 'hi', 6];
}
