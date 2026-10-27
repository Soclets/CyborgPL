op dot [lent $a: int.sign.32, lent $b: int.sign.32] -> int.sign.32:
    ret $a times $b;
end of op dot
!MAIN:
    let $x := dot(1, 2);
!end of MAIN
