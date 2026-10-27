%extern.C proc write [let $fd: int.sign.32, let $buf: ptr[int.unsign.8], let $len: int.unsign.64] -> int.sign.64;
!MAIN:
    let $s := "x";
    let $n := write(1, lend $s, 1);
!end of MAIN
