# `<-` feeds `print.arrows` one value at a time. The arrows are short (owner, 2026-10-07: "Short,
# the neck & eye movement is less"); the long `<--`, main from 2026-10-03, is now refused.
fn double [$n: i32] -> i32:
    ret $n x 2;
;fn double
MAIN:
    std::print.arrows <- double(3) <- newln;
;MAIN
