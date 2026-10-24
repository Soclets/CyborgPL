# The declared type says what the lanes are, and `simd.fill`'s value is fitted into them, as a typed `let` fits
# its value (owner, 2026-10-19: "doesn't doing `let $x: i32` say this var is i32?", then "yea").
MAIN {
    let $b: u8 = 5;
    let $r: simd[i32, count: 4] = simd.fill(value: $b);
    let $q: simd[i32, count: 4] = simd.fill(value: 7: u8);
    std::print.useful(simd.sum($r), " ", simd.sum($q), newln);
}
