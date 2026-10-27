op <+> [$a: int.sign.32, $b: int.sign.32] -> int.sign.32:
    ret $a plus $b;
end of op <+>
!MAIN:
    let $f := <+>;
!end of MAIN
