module a:
    record P:
        x: integer.signed.32_bit;
    end of record P
end of module a
!MAIN:
    let $p := a.P[x = 1];
!end of MAIN
