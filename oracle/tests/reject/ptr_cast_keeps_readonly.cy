%external.C procedure getenv [$name: pointer[integer.unsigned.8_bit]] -> pointer[integer.unsigned.8_bit];
!MAIN:
    let $key := "HOME";
    !no_check let $home := getenv(lend $key);
    !no_check let $p := pointer.cast[integer.signed.8_bit]($home);
    !no_check pointer.write($p, value: 0);
!end of MAIN
