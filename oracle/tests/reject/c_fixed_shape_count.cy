%external.C procedure p_one = %C.printf [$format: pointer[integer.unsigned.8_bit], ...$n: integer.signed.32_bit] -> integer.signed.32_bit;

!MAIN:
    let $f := %C.string.make("%d %d");
    !no_check p_one(lend $f, 1, 2);
!end of MAIN
