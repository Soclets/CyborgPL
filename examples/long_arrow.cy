# `->` in a signature and a function type, `<-` into `print.weird`: the short arrows are the only
# ones (owner, 2026-10-07); `-->` and `<--`, main from 2026-10-03, are refused, naming the short one.
fn double [let $n: int.sign.32] -> int.sign.32:
    ret $n times 2;
end of fn double
fn apply [let $v: int.sign.32, let $f: fn[int.sign.32] -> int.sign.32] -> int.sign.32:
    ret $f($v);
end of fn apply
!MAIN:
    std::print.shortest(double(4), " ", apply(5, double), newln);
    std::print.arrows.shortest <- "weird: " <- double(2) <- newln;
!end of MAIN
