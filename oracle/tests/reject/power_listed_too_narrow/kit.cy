%extern.C proc write [let $fd: i32, let $buf: ptr[u8], let $len: u64] -> i64;

MAIN:
    let $t := "hi";
    let $n := write(1, lend $t, 2);
;MAIN
