%extern.C proc write [let $fd: i32, let $buf: ptr[u8], let $len: u64] -> i64;
MAIN:
    let $s := "x";
    let $n := write(1, lend $s, 1);
;MAIN
