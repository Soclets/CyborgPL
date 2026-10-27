# `->` in a signature and a function type, `<-` into `print.weird`: the short arrows are the only
# ones (owner, 2026-10-07); `-->` and `<--`, main from 2026-10-03, are refused, naming the short one.
function double [$n: integer.signed.32_bit] -> integer.signed.32_bit:
    return $n times 2;
end of function double
function apply [$v: integer.signed.32_bit, $f: function[integer.signed.32_bit] -> integer.signed.32_bit] -> integer.signed.32_bit:
    return $f($v);
end of function apply
!MAIN:
    std::print.shortest(double(4), " ", apply(5, double), newline);
    std::print.arrows.shortest <- "weird: " <- double(2) <- newline;
!end of MAIN
