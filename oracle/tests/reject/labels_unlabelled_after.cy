coll.fn say:
    labels [$t as value];
    member say [$n: i32, $t: i32] -> i32:
        ret $t;
    ;member say
;coll.fn say
MAIN:
    let $x := say(value: 2, 1);
;MAIN
