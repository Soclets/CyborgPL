coll.fn say:
    labels [$a as first, $b as second];
    member say [$a: i32, $b: i32] -> i32:
        ret $a;
    ;member say
;coll.fn say
MAIN:
    let $x := say(second: 2, first: 1);
;MAIN
