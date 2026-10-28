union.tag S:
    a;
    b;
    c;
;union.tag S
MAIN:
    let $s := S.a;
    match $s:
        case a:
        ;case a
        case b:
        ;case b
    ;match
;MAIN
