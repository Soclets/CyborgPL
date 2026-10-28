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
    std::print("before", newln);
    if $p:
        std::print(ov.flip(77), newln);
    ;if
;MAIN
