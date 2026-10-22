# A branch `on.target` rules out is "not compiled at all" (owner, 2026-10-06), so a 256-bit vector
# there is not judged against this 128-bit target; the hand-written fallback runs (owner,
# 2026-10-07: a vector the target lacks is refused, and a fallback is written by hand).
MAIN {
    on.target.chip x86_64 {
        let 'w': simd[i32, count: 8] = simd.fill[value: 1];
    } else {
        let 'v': simd[i32, count: 4] = simd.fill[value: 1];
        std::print.useful[simd.sum['v'], newln];
    }
}
