# Finding 2026-09-23 (generator seed 2, shrunk): a u64 sum above 2^63 is
# refused as if the type were i64.
MAIN {
    let 'v27': u64 = 0;
    change 'v27' = 0 plus 18446744073709551611;
}
