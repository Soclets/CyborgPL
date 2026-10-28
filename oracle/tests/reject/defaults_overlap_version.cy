coll.fn say:
    labels [$t as value, $k as count];
    defaults [$k = 1];
    versions say:
        [let $n: i32, let $t: i32, let $k: i32] -> i32:
            ret 1;
        ;version
        [let $n: i32, let $t: i32] -> i32:
            ret 2;
        ;version
    ;versions say
;coll.fn say
MAIN:
;MAIN
