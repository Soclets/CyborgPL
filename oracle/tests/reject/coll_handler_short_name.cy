fn half [let $n: i32] -> i32 or fail[odd]:
    if $n x 1 == 3:
        fail odd;
    ;if
    ret $n;
;fn half

coll.handler h:
    as_zero:
        case odd:
            ret 0;
        ;case odd
    ;as_zero
;coll.handler h

MAIN:
;MAIN
