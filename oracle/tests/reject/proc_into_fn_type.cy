fn keep_if [$n: i32, $test: fn[i32] -> bool] -> bool:
    ret $test($n);
;fn keep_if

proc each [$n: i32, $act: proc[i32]]:
    $act($n);
;proc each

MAIN:
    let $big: fn[i32] -> bool := fn [$s] = $s >= 90;
    let $say: proc[i32] := proc [$s] = std::print($s, newln);
    std::print(keep_if(91, $say), newln);
    each(7, $say);
    each(8, fn [$s] = $s >= 1);
;MAIN
