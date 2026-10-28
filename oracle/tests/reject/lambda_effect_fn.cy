fn keep_if [let $n: i32, let $test: fn[i32] -> bool] -> bool:
    ret $test($n);
;fn keep_if

proc each [let $n: i32, let $act: proc[i32]]:
    $act($n);
;proc each

MAIN:
    let $big: fn[i32] -> bool := fn [let $s] = $s >= 90;
    let $say: proc[i32] := fn [let $s] = std::print($s, newln);
    std::print(keep_if(91, $big), newln);
    each(7, $say);
    each(8, fn [let $s] = $s >= 1);
;MAIN
