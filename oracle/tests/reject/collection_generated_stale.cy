coll.fn c:
    for $n in [1, 2, 3]:
        named "t" concat to.stored.str.utf8($n);
        member [] -> int.sign.32:
            ret $n;
        end of member
    end of for
    #generated_2_lines_below
    member c.t1 [] -> int.sign.32:
        ret 1;
    end of member c.t1
    member c.t2 [] -> int.sign.32:
        ret 2;
    end of member c.t2
end of coll.fn c
!MAIN:
!end of MAIN
