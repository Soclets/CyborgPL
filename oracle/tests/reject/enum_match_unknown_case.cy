union.tag S:
    a;
    b;
;union.tag S
MAIN:
    let $s := S.a;
    match $s:
        case a:
        ;case a
        case b:
        ;case b
        case z:
        ;case z
    ;match
;MAIN
