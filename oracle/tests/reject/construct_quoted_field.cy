record P:
    x: integer.signed.32_bit;
end of record P
!MAIN:
    let $p := P[$x = 1];
!end of MAIN
