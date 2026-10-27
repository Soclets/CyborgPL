op <+> [$a: int.sign.32, $b: int.sign.32] -> int.sign.32:
    ret $a plus $b;
end of op <+>
!MAIN:
    let $b := 2;
    let $w := 1 <+>-$b;
!end of MAIN
