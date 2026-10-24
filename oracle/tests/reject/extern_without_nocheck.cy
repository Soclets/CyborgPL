extern.C fn write [$fd: i32, $buf: ptr[u8], $len: u64] -> i64;
MAIN {
    let $s := "x";
    let $n := write(1, lend $s, 1);
}
