# Functions that take any number of arguments, typed or not.
fn sum [let ...$nums] -> f64:
    var $total: f64 = 0;
    for $n in lend $nums:
        change $total += to.f64($n);
    ;for
    ret $total;
;fn sum

fn count_ints [let ...$nums: i32] -> u64:
    ret std::ls.len(lend $nums);
;fn count_ints

proc show [lent $label: str.utf8, lent ...$parts]:
    std::print($label, ": ");
    for $p in lend $parts:
        std::print($p, " ");
    ;for
    std::print.stored(newln);
;proc show

MAIN:
    std::print(sum(1, 2.5, 3), newln);
    std::print(sum(), newln);
    std::print(count_ints(4, 5, 6), newln);
    let $name := "Ann";
    show("mixed", lend $name, 7, true, `x`);
    show("none");
;MAIN
