# env CHK_A 1
# Finding 2026-09-23 (check claude, by: hand): with n = 1, m = 10^19 fits
# u64, yet CCA's binary finds m < n.
%extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
%extern.C fn getenv ['name': ptr[u8]] -> ptr[u8];
%extern.C fn strlen ['s': ptr[u8]] -> u64;
MAIN {
    let 'k' := "CHK_A";
    no_check let 'n': u64 = strlen(getenv(lend 'k'));
    let 'm': u64 = 'n' times 10000000000000000000;
    let 'small' := "m is smaller than n";
    if 'm' < 'n' { no_check write(1, lend 'small', 19); }
}
