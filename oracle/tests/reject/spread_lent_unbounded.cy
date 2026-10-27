fn sum [...$nums: int.sign.32] -> int.sign.32:
    ret 0;
end of fn sum
fn f [lent $xs: std::ls[int.sign.32]] -> int.sign.32:
    ret sum(...$xs);
end of fn f
!MAIN:
    let $a := [1, 2]; let $t := f(lend $a);
!end of MAIN
