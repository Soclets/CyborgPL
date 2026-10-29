coll.fn say:
    labels [$t as value, $k as count];
    defaults [$k = 1];
    versions say:
        [$n: i32, $t: i32, $k: i32] -> i32:
            ret 1;
        ;version
        [$n: i32, $t: i32] -> i32:
            ret 2;
        ;version
    ;versions say
;coll.fn say
MAIN:
;MAIN
