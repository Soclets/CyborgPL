# Finding 2026-10-19 (check run, seed 7284, a width probe with a typed literal; cut down by hand): a
# simd vector declared simd[int32, count 4], filled with a value of a narrower type. The oracle fills it,
# widening each lane, and prints ok; kayo builds simd[uint8, count 16] from the value and refuses: "expected
# a value of the declared type (simd[int32, count 4]), found a `simd[uint8, count 16]`". The same with
# to.uint8[5] as with 5: uint8. A plain `let 'r': int32 = 5: uint8;` is widened by both. Whether
# simd.fill takes its lanes from the declared type or from the value's type is not ruled that I can
# find (design/simd.md shows only a bare 0 under :=). Open: the owner's.
MAIN {
    let 'r': simd[int32, count 4] = simd.fill[value 5: uint8];
    let 's': simd[int32, count 4] = simd.fill[value to.uint8[5]];
    std::print.useful["ok", newln];
}
