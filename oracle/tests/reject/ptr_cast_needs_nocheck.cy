%external.C procedure malloc [$bytes: integer.unsigned.64_bit] -> pointer.mutable[integer.unsigned.8_bit];
!MAIN:
    !no_check let $raw := malloc(4);
    let $p := pointer.cast[integer.signed.32_bit]($raw);
!end of MAIN
