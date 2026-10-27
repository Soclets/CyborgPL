%external.C procedure malloc [$n: integer.unsigned.64_bit] -> pointer.mutable[integer.unsigned.8_bit];
!MAIN:
    !no_check let $p := malloc(2);
    !no_check let $q := pointer.deref[$p];
!end of MAIN
