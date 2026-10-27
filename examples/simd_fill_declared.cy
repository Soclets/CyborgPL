# The declared type says what the lanes are, and `simd.fill`'s value is fitted into them, as a typed `let` fits
# its value (owner, 2026-10-19: "doesn't doing `let $x: i32` say this var is i32?", then "yea").
!MAIN:
    let $b: integer.unsigned.8_bit = 5;
    let $r: simd[integer.signed.32_bit, count: 4] = simd.fill(value: $b);
    let $q: simd[integer.signed.32_bit, count: 4] = simd.fill(value: 7: integer.unsigned.8_bit);
    std::print.shortest(simd.sum($r), " ", simd.sum($q), newline);
!end of MAIN
