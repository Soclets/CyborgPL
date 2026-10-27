record P:
    let id: integer.signed.32_bit; x: integer.signed.32_bit;
end of record P
!MAIN:
    variable $p := P[id = 1, x = 2];
    change $p.id = 5;
!end of MAIN
