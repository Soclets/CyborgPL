fn one [$a: int.sign.32] -> int.sign.32:
    ret $a;
end of fn one
!MAIN:
    let $x := one(1, 2);
!end of MAIN
