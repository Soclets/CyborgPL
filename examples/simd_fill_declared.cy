# The declared type says what the lanes are, and `simd.fill`'s value is fitted into them, as a typed `let` fits
# its value (owner, 2026-10-19: "doesn't doing `let $x: i32` say this var is i32?", then "yea").
!MAIN:
    let $b: int.unsign.8 := 5;
    let $r: simd[int.sign.32, count: 4] := simd.fill(value: $b);
    let $q: simd[int.sign.32, count: 4] := simd.fill(value: 7: int.unsign.8);
    std::print.shortest(simd.sum($r), " ", simd.sum($q), newln);
!end of MAIN
