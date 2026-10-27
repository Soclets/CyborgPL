coll.fn f:
    f [$a: int.sign.64] -> int.sign.64:
        ret 1;
    end of f
    f [$a: int.sign.64, $b: int.sign.64] -> int.sign.64:
        ret 2;
    end of f
end of coll.fn f
!MAIN:
!end of MAIN
