# a trailing `,` is allowed (owner, 2026-10-14)
rec P:
    var x: i32;
    var y: i32;
;rec P
union.tag S:
    a;
    b;
;union.tag S
MAIN:
    let $p := P[x = 1, y = 2];
    std::println($p.x + $p.y);
;MAIN
