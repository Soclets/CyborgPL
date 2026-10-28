union.tag S:
    a;
    b;
;union.tag S
MAIN:
    let $x := match S.a:
        case a:
            1
        ;case a
        case b:
            2
        ;case b
    ;match;
;MAIN
