%extern.C proc strlen [$s: ptr[int.unsign.8]] -> int.unsign.64[min: 0, max: 10];
!MAIN:
    let $s := "x"; !no_check let $a := strlen(lend $s);
!end of MAIN
