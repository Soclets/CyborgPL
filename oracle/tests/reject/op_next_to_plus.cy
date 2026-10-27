op <+> [lent $a: int.sign.32, lent $b: int.sign.32] -> int.sign.32:
    ret $a plus $b;
end of op <+>
!MAIN:
    let $w := 1 plus 2 <+> 3;
!end of MAIN
