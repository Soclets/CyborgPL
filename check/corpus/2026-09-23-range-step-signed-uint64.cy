# Owner's ruling 2026-09-23: a range's step is its own signed number, so a
# u64 counter may count down. The oracle refused this before 3502e6c.
extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;
MAIN {
    let 'hi' := "hi";
    for 'i': u64 [from: 3, to: 1, by: -1] {
        no_check write(1, lend 'hi', 2);
    }
}
