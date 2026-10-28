fn sum [let ...$nums: i32] -> i32:
    ret 0;
;fn sum
fn f [lent $xs: std::ls[i32]] -> i32:
    ret sum(...$xs);
;fn f
MAIN:
    let $a := [1, 2]; let $t := f(lend $a);
;MAIN
