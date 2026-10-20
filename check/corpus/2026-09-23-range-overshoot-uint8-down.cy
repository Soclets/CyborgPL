# Finding 2026-09-23 (check claude, by: hand): CCA refuses a for whose last step
# would pass the counter type's edge; 'i' only takes the value 1.
# design/control-flow.md: only the counter's values must fit; compare before stepping.
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
MAIN {
    let 'dot' := ".";
    for 'i': u8 [from: 1, to: 0, by: -2] {
        no_check write[1, lend 'dot', 1];
    }
}
