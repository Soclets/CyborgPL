# Finding 2026-09-24 (generator seed 1086, shrunk): --assert-proofs takes this branch;
# 2^63 compared as a negative number. The oracle and the normal build do not.
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
MAIN {
    let 'v1': u16 = 0;
    let 'yes' := "taken";
    if 9223372036854775808 at_most 'v1' { no_check write(1, lend 'yes', 5); }
}
