coll.fn c:
    for $n [from: 1, before: 3, by: 1]:
        named "t" concat to.stored.str.utf8($n);
        [] -> int.sign.32:
            ret $n;
        end of named
    end of for
    #generated_3_lines_below
    c.t1 [] -> int.sign.32:
        ret 1;
    end of c.t1
    c.t2 [] -> int.sign.32:
        ret 2;
    end of c.t2
    c.t3 [] -> int.sign.32:
        ret 3;
    end of c.t3
end of coll.fn c
!MAIN:
!end of MAIN
