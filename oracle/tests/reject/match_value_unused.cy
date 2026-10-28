# a case that gives a value needs a match used as one (2026-09-29)
union.tag S:
    a;
    b;
;union.tag S
MAIN:
    match S.a:
        case a:
            is 1;
        ;case a
        case b:
            is 2;
        ;case b
    ;match
;MAIN
