# A function value says its kind, as a declaration does (owner, 2026-10-27: (a)): a `fn[…]` holds only `fn`s, a
# `proc[…]` a `proc` or a `fn`; a lambda with effects is `proc [ … ] = …`. A `fn` may call a `fn[…]` it is handed and
# stay a `fn`; calling a `proc[…]` makes it a `proc`.
fn keep_if [let $n: i32, let $test: fn[i32] -> bool] -> bool:
    ret $test($n);
;fn keep_if

proc each [let $n: i32, let $act: proc[i32]]:
    $act($n);
;proc each

MAIN:
    let $big: fn[i32] -> bool := fn [let $s] = $s >= 90;
    let $say: proc[i32] := proc [let $s] = std::print($s, newln);
    std::print(keep_if(91, $big), newln);
    each(7, $say);
    each(8, fn [let $s] = $s + 1);
;MAIN
