# A short lambda (owner, 2026-10-24: "(c)"): one value after `=`, its parameters' types and its return type taken from
# where it goes, `keep`'s `fn[i32] -> bool` or a declared type; captures stay explicit with `with`.
fn keep [lent $xs: std::ls[int.sign.32], let $test: fn[int.sign.32] -> bool] -> std::ls[int.sign.32]:
    var $out: std::ls[int.sign.32] = std::ls.empty;
    for $x in lend $xs:
        if $test($x):
            std::ls.append(lend.mut $out, value: $x);
        end of if
    end of for
    ret give $out;
end of fn keep

!MAIN:
    let $scores: std::ls[int.sign.32] := [55, 72, 60, 91, 38];
    let $limit := 60;
    let $passing := keep(lend $scores, fn [let $s] with [lend $limit] = $s at_least $limit);
    let $evens := keep(lend $scores, fn [let $s] = ($s mod.floor 2) eq 0);
    std::print.shortest($passing, " ", $evens, newln);
    let $big: fn[int.sign.32] -> bool := fn [let $s] = $s at_least 90;
    std::print.shortest($big(91), " ", twice_if(5, fn [let $n] = $n at_most 9), newln);
!end of MAIN

fn twice_if [let $n: int.sign.32, let $test: fn[int.sign.32] -> bool] -> int.sign.32:
    if $test($n):
        ret $n times 2;
    end of if
    ret $n;
end of fn twice_if
