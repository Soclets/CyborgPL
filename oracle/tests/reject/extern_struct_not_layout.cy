record P:
    x: integer.signed.32_bit;
end of record P
%external.C procedure f [$a: pointer[P]];
!MAIN:
!end of MAIN
