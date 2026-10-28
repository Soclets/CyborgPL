coll.fn ov:
    versions ov.flip:
        [let $x: i32] -> i32:
            ret 1;
        ;version
        [let $x: i64] -> i32:
            ret 2;
        ;version
    ;versions ov
;coll.fn ov
MAIN:
    let $p := false;
    std::println("before");
    if $p:
        std::println(ov.flip(77));
    ;if
;MAIN
