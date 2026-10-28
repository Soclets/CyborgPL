coll.fn c:
    for $n in [1, 2, 3]:
        named "t" concat to.stored.str.utf8($n);
        member [] -> i32:
            ret $n;
        ;member
    ;for
    #generated_2_lines_below
    member c.t1 [] -> i32:
        ret 1;
    ;member c.t1
    member c.t2 [] -> i32:
        ret 2;
    ;member c.t2
;coll.fn c
MAIN:
;MAIN
