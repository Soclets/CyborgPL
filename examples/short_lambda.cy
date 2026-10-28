# A short lambda (owner, 2026-10-24: "(c)"): one value after `=`, its parameters' types and its return type taken from
# where it goes, `keep`'s `fn[i32] -> bool` or a declared type; captures stay explicit with `with`.
fn keep [lent $xs: std::ls[i32], let $test: fn[i32] -> bool] -> std::ls[i32]:
    var $out: std::ls[i32] = std::ls.empty;
    for $x in lend $xs:
        if $test($x):
            std::ls.append(lend.mut $out, value: $x);
        ;if
    ;for
    ret give $out;
;fn keep

MAIN:
    let $scores: std::ls[i32] := [55, 72, 60, 91, 38];
    let $limit := 60;
    let $passing := keep(lend $scores, fn [let $s] with [lend $limit] = $s >= $limit);
    let $evens := keep(lend $scores, fn [let $s] = ($s mod.floor 2) == 0);
    std::print($passing, " ", $evens, newln);
    let $big: fn[i32] -> bool := fn [let $s] = $s >= 90;
    std::print($big(91), " ", twice_if(5, fn [let $n] = $n <= 9), newln);
;MAIN

fn twice_if [let $n: i32, let $test: fn[i32] -> bool] -> i32:
    if $test($n):
        ret $n x 2;
    ;if
    ret $n;
;fn twice_if
