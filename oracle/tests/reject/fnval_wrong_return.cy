fn one [$a: int.sign.32] -> bool:
    ret true;
end of fn one
fn apply [$f: fn[int.sign.32] -> int.sign.32] -> int.sign.32:
    ret $f(1);
end of fn apply
!MAIN:
    let $x := apply(one);
!end of MAIN
