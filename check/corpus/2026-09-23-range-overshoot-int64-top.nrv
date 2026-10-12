# Finding 2026-09-23 (check claude, by hand): as range-overshoot-uint8-down, at int64's top.
extern.C fn write ['fd': int32, 'buf': ptr[uint8], 'len': uint64] -> int64;
MAIN {
    let 'dot' := ".";
    for 'i': int64 from 9223372036854775806 to 9223372036854775807 by 2 {
        no_check write[1, lend 'dot', 1];
    }
}
