# A short lambda (owner, 2026-10-24: "(c)"): one value after `=`, its parameters' types and its return type taken from
# where it goes, `keep`'s `fn[i32] -> bool` or a declared type; captures stay explicit with `with`.
fn keep [lent $xs: std::list[i32], $test: fn[i32] -> bool] -> std::list[i32] {
    let mut $out: std::list[i32] = std::list.empty;
    for $x in lend $xs {
        if $test($x) { std::list.append(lend.mut $out, value: $x); }
    }
    return give $out;
}

MAIN {
    let $scores: std::list[i32] = [55, 72, 60, 91, 38];
    let $limit := 60;
    let $passing := keep(lend $scores, fn [$s] with [lend $limit] = $s at_least $limit);
    let $evens := keep(lend $scores, fn [$s] = ($s mod.floor 2) equal_to 0);
    std::print.useful($passing, " ", $evens, newln);
    let $big: fn[i32] -> bool = fn [$s] = $s at_least 90;
    std::print.useful($big(91), " ", twice_if(5, fn [$n] = $n at_most 9), newln);
}

fn twice_if [$n: i32, $test: fn[i32] -> bool] -> i32 {
    if $test($n) { return $n x 2; }
    return $n;
}
