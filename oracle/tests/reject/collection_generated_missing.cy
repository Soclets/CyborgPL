coll.fn c:
    for $n in [1, 2]:
        named "t" concat to.stored.str.utf8($n);
        [] -> int.sign.32:
            ret $n;
        end of named
    end of for
end of coll.fn c
!MAIN:
!end of MAIN
