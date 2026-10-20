extern.C fn write ['fd': i32, 'buf': ptr[u8], 'len': u64] -> i64;

MAIN {
    let 't' := "hi";
    let 'n' := write[1, lend 't', 2];
}
