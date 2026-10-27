%extern.C proc write [$fd: int.sign.32, $buf: ptr[int.unsign.8], $len: int.unsign.64] -> int.sign.64;
!MAIN:
    let $s := "x";
    let $n := write(1, lend $s, 1);
!end of MAIN
