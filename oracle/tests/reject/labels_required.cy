coll.fn say:
    labels [$t as value];
    say [$n: int.sign.32, $t: int.sign.32] -> int.sign.32:
        ret $t;
    end of say
end of coll.fn say
!MAIN:
    let $x := say(1);
!end of MAIN
