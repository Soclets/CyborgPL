# a trailing `,` is allowed (owner, 2026-10-14)
rec P:
    x: int.sign.32;
    y: int.sign.32;
end of rec P
union.tag S:
    a;
    b;
end of union.tag S
!MAIN:
    let $p := P[x = 1, y = 2];
    std::print.shortest($p.x plus $p.y, newln);
!end of MAIN
