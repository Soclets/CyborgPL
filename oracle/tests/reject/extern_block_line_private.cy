
module text:
    %extern.C:
        pub proc strlen [$s: ptr[u8]] -> u64;
        proc write [$fd: i32, $buf: ptr[u8], $len: u64] -> i64;
    ;%extern.C
;module text
MAIN:
    let $msg := "hi";
    no_check let $n := text.write(1, lend $msg, 2);
;MAIN
