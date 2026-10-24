# `->` in a signature and a function type, `<-` into `print.weird`: the short arrows are the only
# ones (owner, 2026-10-07); `-->` and `<--`, main from 2026-10-03, are refused, naming the short one.
fn double [$n: i32] -> i32 { return $n x 2; }
fn apply [$v: i32, $f: fn[i32] -> i32] -> i32 { return $f($v); }
MAIN {
    std::print.useful(double(4), " ", apply(5, double), newln);
    std::print.weird.useful <- "weird: " <- double(2) <- newln;
}
