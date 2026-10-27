coll.fn say:
    labels [$t as value];
    member say [let $n: int.sign.32, let $t: int.sign.32] -> int.sign.32:
        ret $t;
    end of member say
end of coll.fn say
!MAIN:
    let $x := say(value: 2, 1);
!end of MAIN
