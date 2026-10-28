
module text:
    %extern.C:
        pub proc strlen [let $s: ptr[u8]] -> u64;
        proc write [let $fd: i32, let $buf: ptr[u8], let $len: u64] -> i64;
    ;%extern.C
;module text
MAIN:
    let $msg := "hi";
    no_check let $n := text.write(1, lend $msg, 2);
;MAIN
