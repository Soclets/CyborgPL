# Finding 2026-10-19 (check run, seed 7284, a width probe with a typed literal; cut down by hand),
# ruled the same day (owner: "(a)"; DECISIONS a1a982c4): simd.fill fits its value into the declared
# lanes, so simd.fill[value: 5: u8] into simd[i32, count: 4] fills four i32 lanes, as a typed
# let would, refused only when the value does not fit. The oracle did so; kayo had built
# simd[u8, 16] from the value and refused, until 3e5bf3c8. Both print ok now.
MAIN {
    let 'r': simd[i32, count: 4] = simd.fill(value: 5: u8);
    let 's': simd[i32, count: 4] = simd.fill(value: to.u8(5));
    std::print.useful("ok", newln);
}
