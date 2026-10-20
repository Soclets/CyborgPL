# Finding 2026-09-23 (check claude, by: hand): as range-overshoot-u8-down, upwards.
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
MAIN {
    let 'dot' := ".";
    for 'i': u8 [from: 250, to: 255, by: 3] {
        no_check write[1, lend 'dot', 1];
    }
}
