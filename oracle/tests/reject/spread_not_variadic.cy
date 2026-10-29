fn one [$a: i32] -> i32:
    ret $a;
;fn one
MAIN:
    let $xs := [1, 2]; let $t := one(...$xs);
;MAIN
