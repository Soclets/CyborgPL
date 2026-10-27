record P:
    x: integer.signed.32_bit;
end of record P
function poke [lent $q: P]:
    change $q.x = 9;
end of function poke
!MAIN:
    let $p := P[x = 1];
    poke(lend $p);
!end of MAIN
