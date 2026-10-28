coll.fn f:
    member f [let $a: i64] -> i64:
        ret 1;
    ;member f
    member f [let $a: i64, let $b: i64] -> i64:
        ret 2;
    ;member f
;coll.fn f
MAIN:
;MAIN
