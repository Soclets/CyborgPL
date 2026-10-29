# `no_cheat` — "no cheat" (owner, 2026-10-01): the work written really happens, so the compiler may not
# replace it with its answer (here n(n+1)/2), drop it, or compute look-alike calls once; SIMD, cores and
# inlining are still allowed. It goes in front of any statement or block, as `no_check` does, calls
# included (owner, 2026-10-18: "(b)"), and in front of a `for`, never inside it (owner: "(a)").
fn fib [$n: i64[min: 0, max: 92]] -> i64:
    if $n < 2:
        ret $n;
    ;if
    ret fib($n - 1) + fib($n - 2);
;fn fib

MAIN:
    var $acc: i64 = 0;
    no_skip for $i: i64 [from: 1, to: 1000, by: 1]:
        change $acc to $acc + $i;
    ;for
    var $more: i64 = 0;
    no_skip for $i: i64 [from: 1, to: 10, by: 1]:
        change $more to $more + $i;
    ;for
    no_skip let $f := fib(20);
    var $both: i64 = 0;
    no_skip:
        change $both to $acc + $more;
    ;no_skip
    # beside `no_check`, `no_check` comes first: the stronger word, and the unchecked part seen first (owner, 2026-10-19)
    var $last: i64 = 0;
    no_check no_skip change $last to $f;
    std::print($acc, " ", $more, " ", $f, " ", $both, " ", $last, newln);
;MAIN
