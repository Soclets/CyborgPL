fn one [$a: i32] -> bool:
    ret true;
;fn one
fn apply [$f: fn[i32] -> i32] -> i32:
    ret $f(1);
;fn apply
MAIN:
    let $x := apply(one);
;MAIN
