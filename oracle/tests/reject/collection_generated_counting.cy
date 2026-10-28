coll.fn c:
    for $n [from: 1, before: 3, by: 1]:
        named "t" concat to.stored.str.utf8($n);
        member [] -> i32:
            ret $n;
        ;member
    ;for
    #generated_3_lines_below
    member c.t1 [] -> i32:
        ret 1;
    ;member c.t1
    member c.t2 [] -> i32:
        ret 2;
    ;member c.t2
    member c.t3 [] -> i32:
        ret 3;
    ;member c.t3
;coll.fn c
MAIN:
;MAIN
