# Finding 2026-09-23 (check claude, by: hand): as range-overshoot-u8-down, at: i64's top.
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
MAIN {
    let 'dot' := ".";
    for 'i': i64 [from: 9223372036854775806, to: 9223372036854775807, by: 2] {
        no_check write[1, lend 'dot', 1];
    }
}
