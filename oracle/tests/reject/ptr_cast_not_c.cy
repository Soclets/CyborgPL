%external.C procedure malloc [$bytes: integer.unsigned.64_bit] -> pointer.mutable[integer.unsigned.8_bit];
record Plain:
    x: integer.signed.32_bit;
end of record Plain
!MAIN:
    !no_check let $raw := malloc(4);
    !no_check let $p := pointer.cast[Plain]($raw);
!end of MAIN
