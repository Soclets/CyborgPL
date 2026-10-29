coll.fn f:
    versions f:
        [$a: i64] -> i64:
            ret 1;
        ;version
        [$b: i64] -> i64:
            ret 2;
        ;version
    ;versions f
;coll.fn f
MAIN:
    std::print(f(1), newln);
;MAIN
