record P:
    n: string.utf8; x: integer.signed.32_bit;
end of record P
!MAIN:
    let $p := P[n = "a", x = 2];
    let $t := give $p.n;
    std::print.shortest($p.x);
!end of MAIN
