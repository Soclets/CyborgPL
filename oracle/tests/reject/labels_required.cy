coll.fn say:
    labels [$t as value];
    member say [let $n: i32, let $t: i32] -> i32:
        ret $t;
    ;member say
;coll.fn say
MAIN:
    let $x := say(1);
;MAIN
