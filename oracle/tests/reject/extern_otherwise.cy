%extern.C proc f [$a: int.sign.32] -> int.sign.32;
!MAIN:
    !no_check let $x := f(1) otherwise 0;
!end of MAIN
