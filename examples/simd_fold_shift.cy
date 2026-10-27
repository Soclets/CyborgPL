# A vector folded to one lane gives a shift the lane's width (check claude, seed 4094): `simd.max($s)`
# of u8 lanes is a u8, so `shl 1` is an 8-bit shift.
!MAIN:
    let $s: simd[int.unsign.8, count: 16] := simd.fill(value: 3);
    std::print.shortest(simd.max($s) shl 1, newln);
!end of MAIN
