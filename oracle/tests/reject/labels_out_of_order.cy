coll.fn say:
    labels [$a as first, $b as second];
    member say [let $a: i32, let $b: i32] -> i32:
        ret $a;
    ;member say
;coll.fn say
MAIN:
    let $x := say(second: 2, first: 1);
;MAIN
