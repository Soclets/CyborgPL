# `<-` feeds `print.weird` one value at a time. The arrows are short (owner, 2026-10-07: "Short,
# the neck & eye movement is less"); the long `<--`, main from 2026-10-03, is now refused.
fn double [$n: i32] -> i32 { return $n x 2; }
MAIN {
    std::print.weird.useful <- double(3) <- newln;
}
