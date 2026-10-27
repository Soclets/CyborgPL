coll.fn mx:
    versions mx:
        [$a: int.sign.32, $b: int.sign.32] -> int.sign.32:
            ret $a plus $b;
        end of version
        [$a: int.sign.32, $b: int.sign.64] -> int.sign.32:
            ret $a minus to.int.sign.32($b);
        end of version
    end of versions mx
end of coll.fn mx
!MAIN:
    let $v: int.sign.32 = 4;
    let $p: bool = to.int.sign.64(1) eq to.int.sign.64(2);
    std::print.shortest("before", newln);
    if $p:
        std::print.shortest(mx($v, 5), newln);
    end of if
    std::print.shortest("after", newln);
!end of MAIN
