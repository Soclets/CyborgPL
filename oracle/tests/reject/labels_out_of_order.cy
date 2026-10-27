coll.fn say:
    labels [$a as first, $b as second];
    member say [let $a: int.sign.32, let $b: int.sign.32] -> int.sign.32:
        ret $a;
    end of member say
end of coll.fn say
!MAIN:
    let $x := say(second: 2, first: 1);
!end of MAIN
