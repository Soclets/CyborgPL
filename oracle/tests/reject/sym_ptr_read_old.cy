%external.C procedure malloc [$bytes: integer.unsigned.64_bit] -> pointer.mutable[integer.unsigned.8_bit];
!MAIN:
    !no_check let $p := malloc(4);
    !no_check let $c := *$p;
!end of MAIN
