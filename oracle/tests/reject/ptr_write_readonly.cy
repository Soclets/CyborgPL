%external.C procedure getenv [$n: pointer[integer.unsigned.8_bit]] -> pointer[integer.unsigned.8_bit];
!MAIN:
    let $k := "PATH";
    !no_check let $p := getenv(lend $k);
    !no_check pointer.write($p, value: 1);
!end of MAIN
