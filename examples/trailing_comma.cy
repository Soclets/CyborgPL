# a trailing `,` is allowed (owner, 2026-10-14)
record P:
    x: integer.signed.32_bit;
    y: integer.signed.32_bit;
end of record P
union.tagged S:
    a; b;
end of union.tagged S
!MAIN:
    let $p := P[x = 1, y = 2];
    std::print.shortest($p.x plus $p.y, newline);
!end of MAIN
