%external.C procedure write [$fd: integer.signed.32_bit, $buf: pointer[integer.unsigned.8_bit], $len: integer.unsigned.64_bit] -> integer.signed.64_bit;
!MAIN:
    let $s := "x";
    let $n := write(1, lend $s, 1);
!end of MAIN
