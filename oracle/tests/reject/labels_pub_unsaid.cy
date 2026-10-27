pub coll.fn say:
    labels [$t as value];
    member say [let $t: int.sign.32] -> int.sign.32:
        ret $t;
    end of member say
end of coll.fn say
!MAIN:
!end of MAIN
