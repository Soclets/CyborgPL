record P:
    x: integer.signed.32_bit;
end of record P
!MAIN:
    let $p := P[x = "no"];
!end of MAIN
