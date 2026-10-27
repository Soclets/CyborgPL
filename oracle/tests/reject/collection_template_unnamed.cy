coll.fn c:
    for $n in [1, 2]:
        member [] -> int.sign.32:
            ret $n;
        end of member
    end of for
end of coll.fn c
!MAIN:
!end of MAIN
