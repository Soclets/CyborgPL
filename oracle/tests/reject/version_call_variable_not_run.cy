coll.fn mx:
    versions mx:
        [let $a: i32, let $b: i32] -> i32:
            ret $a + $b;
        ;version
        [let $a: i32, let $b: i64] -> i32:
            ret $a - to.i32($b);
        ;version
    ;versions mx
;coll.fn mx
MAIN:
    let $v: i32 := 4;
    let $p: bool := to.i64(1) == to.i64(2);
    std::println("before");
    if $p:
        std::println(mx($v, 5));
    ;if
    std::println("after");
;MAIN
