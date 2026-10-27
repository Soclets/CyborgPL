fn one [let $a: int.sign.32] -> int.sign.32:
    ret $a;
end of fn one
!MAIN:
    let $xs := [1, 2]; let $t := one(...$xs);
!end of MAIN
