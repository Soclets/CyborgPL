coll.fn c:
    for $n in [1, 2]:
        named "t" ++ to.stored.str.utf8($n);
        member [] -> i32:
            ret $n;
        ;member
    ;for
;coll.fn c
MAIN:
;MAIN
