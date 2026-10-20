# env CHK_A 8
# Finding 2026-09-23 (check claude, by: hand): a shl by a run-time count past
# the width was reported "checked at run time" but ran on. Fixed in c36cfce.
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
extern.C fn strlen ['s': ptr[u8]] -> u64;
MAIN {
    let 'k' := "CHK_A";
    no_check let 'n': u64 = strlen[getenv[lend 'k']];
    let 'v': u8 = 200;
    let 's': u8 = to.u8['n'];
    let 'r': u8 = 'v' shl 's';
    let 'hi' := "ran on";
    no_check write[1, lend 'hi', 6];
}
