%external.C procedure malloc [$n: integer.unsigned.64_bit] -> pointer.mutable[integer.unsigned.8_bit];
!MAIN:
    !no_check let $p := malloc(4);
    let $b := pointer.read($p);
!end of MAIN
