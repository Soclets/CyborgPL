%external.C procedure getenv [$n: pointer[integer.unsigned.8_bit]] -> pointer[integer.unsigned.8_bit];
!MAIN:
    let $k := "CYBORG_NOPE_XYZ";
    !no_check let $p := getenv(lend $k);
    !no_check let $b := pointer.read($p);
!end of MAIN
