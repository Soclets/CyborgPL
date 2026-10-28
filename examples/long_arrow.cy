# `->` in a signature and a function type, `<-` into `print.arrows`: the short arrows are the only
# ones (owner, 2026-10-07); `-->` and `<--`, main from 2026-10-03, are refused, naming the short one.
fn double [let $n: i32] -> i32:
    ret $n x 2;
;fn double
fn apply [let $v: i32, let $f: fn[i32] -> i32] -> i32:
    ret $f($v);
;fn apply
MAIN:
    std::print(double(4), " ", apply(5, double), newln);
    std::print.arrows.shortest <- "weird: " <- double(2) <- newln;
;MAIN
