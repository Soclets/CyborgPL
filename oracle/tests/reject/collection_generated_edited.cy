coll.fn c:
    for $n in [1, 2]:
        named "t" concat to.stored.str.utf8($n);
        [] -> int.sign.32:
            ret $n;
        end of named
    end of for
    #generated_2_lines_below
    c.t1 [] -> int.sign.32:
        ret 1;
    end of c.t1
    c.t2 [] -> int.sign.32:
        ret 20;
    end of c.t2
end of coll.fn c
!MAIN:
!end of MAIN
