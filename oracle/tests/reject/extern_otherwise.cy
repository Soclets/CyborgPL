%external.C procedure f [$a: integer.signed.32_bit] -> integer.signed.32_bit;
!MAIN:
    !no_check let $x := f(1) otherwise 0;
!end of MAIN
