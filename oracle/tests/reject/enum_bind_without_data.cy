union.tag S:
    a;
    b;
;union.tag S
MAIN:
    let $s := S.a;
    match $s:
        case a $x:
        ;case a
        case b:
        ;case b
    ;match
;MAIN
