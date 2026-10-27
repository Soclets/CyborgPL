record P:
    x: integer.signed.64_bit;
end of record P
!MAIN:
    let $p := P(x = 1);
!end of MAIN
