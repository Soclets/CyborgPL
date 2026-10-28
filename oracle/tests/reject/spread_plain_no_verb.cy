fn sum [let ...$n: i32] -> i32:
    ret 0;
;fn sum
MAIN:
    let $x := [1, 2]; let $t := sum(...copy $x);
;MAIN
