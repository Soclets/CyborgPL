# env CHK_A 8
# Finding 2026-09-23 (check claude, by hand): a shl by a run-time count past
# the width was reported "checked at run time" but ran on. Fixed in c36cfce.
extern.C fn write ['fd': int32, 'buf': ptr[uint8], 'len': uint64] -> int64;
extern.C fn getenv ['name': ptr[uint8]] -> ptr[uint8];
extern.C fn strlen ['s': ptr[uint8]] -> uint64;
MAIN {
    let 'k' := "CHK_A";
    no_check let 'n': uint64 = strlen[getenv[lend 'k']];
    let 'v': uint8 = 200;
    let 's': uint8 = to.uint8['n'];
    let 'r': uint8 = 'v' shl 's';
    let 'hi' := "ran on";
    no_check write[1, lend 'hi', 6];
}
