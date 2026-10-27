fn one [let $a: int.sign.32] -> int.sign.32:
    ret $a;
end of fn one
fn apply [let $f: fn[int.sign.32, int.sign.32] -> int.sign.32] -> int.sign.32:
    ret $f(1, 2);
end of fn apply
!MAIN:
    let $x := apply(one);
!end of MAIN
