fn keep_if [let $n: int.sign.32, let $test: fn[int.sign.32] -> bool] -> bool:
    ret $test($n);
end of fn keep_if

proc each [let $n: int.sign.32, let $act: proc[int.sign.32]]:
    $act($n);
end of proc each

!MAIN:
    let $big: fn[int.sign.32] -> bool := fn [let $s] = $s at_least 90;
    let $say: proc[int.sign.32] := proc [let $s] = std::print.shortest($s, newln);
    std::print.shortest(keep_if(91, $say), newln);
    each(7, $say);
    each(8, fn [let $s] = $s at_least 1);
!end of MAIN
