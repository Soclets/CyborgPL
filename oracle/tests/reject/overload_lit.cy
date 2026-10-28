coll.fn f:
    versions f:
        [let $a: i64] -> i64:
            ret 1;
        ;version
        [let $b: i32] -> i64:
            ret 2;
        ;version
    ;versions f
;coll.fn f
MAIN:
    std::println(f(1));
;MAIN
