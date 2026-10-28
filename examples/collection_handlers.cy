# `coll.handler` (owner, 2026-10-13): named `otherwise` blocks. Each member holds the arms written
# after `otherwise` today, one per way the call can fail, and `otherwise half_failed.as_zero` reads them
# as if they were written at the call: an arm may `ret`, `break`, `continue` or `fail` from where it
# is used, and a member that misses a way the call can fail is refused there.
fn half [let $n: i32] -> i32 or fail[odd, negative: i32]:
    if $n < 0:
        fail negative[$n];
    ;if
    if ($n div.floor 2) x 2 != $n:
        fail odd;
    ;if
    ret $n div.floor 2;
;fn half

coll.handler half_failed:
    member half_failed.as_zero:
        case odd:
            ret 0;
        ;case odd
        case negative $v:
            std::print("negative: ", $v, newln); ret 0;
        ;case negative
    ;member half_failed.as_zero
    member half_failed.skip:
        case odd:
            continue;
        ;case odd
        case negative $v:
            continue;
        ;case negative
    ;member half_failed.skip
    member half_failed.stop:
        case odd:
            break;
        ;case odd
        case negative $v:
            break;
        ;case negative
    ;member half_failed.stop
;coll.handler half_failed

proc half_or_zero [let $n: i32] -> i32:
    let $h := half($n) otherwise half_failed.*as_zero;
    ret $h;
;proc half_or_zero

fn sum_of_halves [lent $xs: std::ls[i32], let $until_bad: bool] -> i32:
    var $sum = 0;
    for $x in lend $xs:
        if $until_bad:
            let $h := half($x) otherwise half_failed.*stop;
            change $sum to $sum + $h;
        else:
            let $h := half($x) otherwise half_failed.*skip;
            change $sum to $sum + $h;
        ;if
    ;for
    ret $sum;
;fn sum_of_halves

MAIN:
    std::print(half_or_zero(10), " ", half_or_zero(7), newln);
    std::print(half_or_zero(-4), newln);
    let $xs: std::ls[i32] := [2, 4, 5, 8, -2, 10];
    std::print(sum_of_halves(lend $xs, false), " ", sum_of_halves(lend $xs, true), newln);
;MAIN
