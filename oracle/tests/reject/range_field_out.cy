record P:
    v: integer.unsigned.8_bit[minimum: 0, maximum: 100];
end of record P
!MAIN:
    let $p := P[v = 120];
!end of MAIN
