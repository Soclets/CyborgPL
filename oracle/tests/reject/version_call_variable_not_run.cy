coll.fn mx:
    versions mx:
        [$a: i32, $b: i32] -> i32:
            ret $a + $b;
        ;version
        [$a: i32, $b: i64] -> i32:
            ret $a - to.i32($b);
        ;version
    ;versions mx
;coll.fn mx
MAIN:
    let $v: i32 := 4;
    let $p: bool := to.i64(1) == to.i64(2);
    std::print("before", newln);
    if $p:
        std::print(mx($v, 5), newln);
    ;if
    std::print("after", newln);
;MAIN
