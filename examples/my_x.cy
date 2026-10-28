# `x` is multiplication between two values; something may still be named `x`, reached by its path as
# `my.x(…)`, the way a function named `true` is (owner, 2026-10-28).
fn x [let $n: i32] -> i32:
    ret $n x 10;
;fn x

MAIN:
    std::println(my.x(4) x 2);
;MAIN
