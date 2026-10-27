coll.fn c:
    member c [$x: int.sign.32] -> int.sign.32:
        ret $x;
    end of member c
    member c.q [$nope = 1];
end of coll.fn c
!MAIN:
!end of MAIN
