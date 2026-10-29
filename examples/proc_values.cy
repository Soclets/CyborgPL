# A function value says its kind, as a declaration does (owner, 2026-10-27: (a)): a `fn[…]` holds only `fn`s, a
# `proc[…]` a `proc` or a `fn`; a lambda with effects is `proc [ … ] = …`. A `fn` may call a `fn[…]` it is handed and
# stay a `fn`; calling a `proc[…]` makes it a `proc`.
fn keep_if [$n: i32, $test: fn[i32] -> bool] -> bool:
    ret $test($n);
;fn keep_if

proc each [$n: i32, $act: proc[i32]]:
    $act($n);
;proc each

MAIN:
    let $big: fn[i32] -> bool := fn [$s] = $s >= 90;
    let $say: proc[i32] := proc [$s] = std::print($s, newln);
    std::print(keep_if(91, $big), newln);
    each(7, $say);
    each(8, fn [$s] = $s + 1);
;MAIN
