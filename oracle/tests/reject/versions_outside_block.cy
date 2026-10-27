coll.fn f:
    member f [$a: int.sign.64] -> int.sign.64:
        ret 1;
    end of member f
    member f [$a: int.sign.64, $b: int.sign.64] -> int.sign.64:
        ret 2;
    end of member f
end of coll.fn f
!MAIN:
!end of MAIN
