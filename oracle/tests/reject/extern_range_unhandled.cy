%external.C procedure strlen [$s: pointer[integer.unsigned.8_bit]] -> integer.unsigned.64_bit[minimum: 0, maximum: 10];
!MAIN:
    let $s := "x"; !no_check let $a := strlen(lend $s);
!end of MAIN
