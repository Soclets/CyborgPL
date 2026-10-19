# Finding 2026-10-19 (check run, seed 7284, a width probe with a typed literal; cut down by hand),
# ruled the same day (owner: "(a)"; DECISIONS a1a982c4): simd.fill fits its value into the declared
# lanes, so simd.fill[value 5: uint8] into simd[int32, count 4] fills four int32 lanes, as a typed
# let would, refused only when the value does not fit. The oracle did so; kayo had built
# simd[uint8, 16] from the value and refused, until 3e5bf3c8. Both print ok now.
MAIN {
    let 'r': simd[int32, count 4] = simd.fill[value 5: uint8];
    let 's': simd[int32, count 4] = simd.fill[value to.uint8[5]];
    std::print.useful["ok", newln];
}
