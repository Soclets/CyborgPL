coll.fn say:
    labels [$t as value];
    member say [$t: i32] -> i32:
        ret $t;
    ;member say
;coll.fn say
MAIN:
    let $x := say(colour: 1);
;MAIN
