# `no_cheat` — "no cheat" (owner, 2026-10-01): the work written really happens, so the compiler may not
# replace it with its answer (here n(n+1)/2), drop it, or compute look-alike calls once; SIMD, cores and
# inlining are still allowed. It goes in front of any statement or block, as `no_check` does, calls
# included (owner, 2026-10-18: "(b)"), and in front of a `for`, never inside it (owner: "(a)").
function fib [$n: integer.signed.64_bit[minimum: 0, maximum: 92]] -> integer.signed.64_bit:
    if $n less_than 2:
        return $n;
    end of if
    return fib($n minus 1) plus fib($n minus 2);
end of function fib

!MAIN:
    variable $acc: integer.signed.64_bit = 0;
    no_skip for $i: integer.signed.64_bit [from: 1, to: 1000, by: 1]:
        change $acc = $acc plus $i;
    end of for
    variable $more: integer.signed.64_bit = 0;
    no_skip for sequential $i: integer.signed.64_bit [from: 1, to: 10, by: 1]:
        change $more = $more plus $i;
    end of for
    no_skip let $f := fib(20);
    variable $both: integer.signed.64_bit = 0;
    no_skip:
        change $both = $acc plus $more;
    end of no_skip
    # beside `no_check`, `no_check` comes first: the stronger word, and the unchecked part seen first (owner, 2026-10-19)
    variable $last: integer.signed.64_bit = 0;
    !no_check no_skip change $last = $f;
    std::print.shortest.auto_newline($acc, " ", $more, " ", $f, " ", $both, " ", $last);
!end of MAIN
