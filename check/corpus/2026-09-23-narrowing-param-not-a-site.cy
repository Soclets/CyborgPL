# env CHK_A 300
# Finding 2026-09-23 (check claude, by: hand): passing a u64 to a u8
# parameter is not counted as a site at all.
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
extern.C fn strlen ['s': ptr[u8]] -> u64;
fn g ['v': u8] -> u8 { return 'v'; }
MAIN {
    let 'k' := "CHK_A";
    no_check let 'n': u64 = strlen(getenv(lend 'k'));
    let 'b': u8 = g('n');
    let 'hi' := "ran on";
    no_check write(1, lend 'hi', 6);
}
