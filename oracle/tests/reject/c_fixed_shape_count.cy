%extern.C proc p_one = %C.printf [$format: ptr[int.unsign.8], ...$n: int.sign.32] -> int.sign.32;

!MAIN:
    let $f := %C.str.make("%d %d");
    !no_check p_one(lend $f, 1, 2);
!end of MAIN
