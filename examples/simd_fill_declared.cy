# The declared type says what the lanes are, and `simd.fill`'s value is fitted into them, as a typed `let` fits
# its value (owner, 2026-10-19: "doesn't doing `let 'x': int32` say this var is int32?", then "yea").
MAIN {
    let 'b': uint8 = 5;
    let 'r': simd[int32, count 4] = simd.fill[value 'b'];
    let 'q': simd[int32, count 4] = simd.fill[value 7: uint8];
    std::print.useful[simd.sum['r'], " ", simd.sum['q'], newln];
}
