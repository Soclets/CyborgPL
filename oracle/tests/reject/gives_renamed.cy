# `gives` is spelled `giveS`, the capital S keeping it apart from `give` (owner, 2026-09-29)
union.tag S:
    a;
    b;
;union.tag S
MAIN:
    let $x: i32 := match S.a:
        case a:
            gives 1;
        ;case a
        case b:
            gives 2;
        ;case b
    ;match;
;MAIN
