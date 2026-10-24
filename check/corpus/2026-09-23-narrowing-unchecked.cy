# env CHK_A 300
# Finding 2026-09-23 (check claude, by: hand): a narrowing site reported
# "checked at run time" has no check; 300 became 44.
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
extern.C fn strlen ['s': ptr[u8]] -> u64;
MAIN {
    let 'k' := "CHK_A";
    no_check let 'n': u64 = strlen(getenv(lend 'k'));
    let 'b': u8 = 'n';
    let 'yes' := "b is 44";
    if 'b' equal_to 44 { no_check write(1, lend 'yes', 7); }
}
