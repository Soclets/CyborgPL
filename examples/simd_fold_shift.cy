# A vector folded to one lane gives a shift the lane's width (check claude, seed 4094): `simd.max($s)`
# of u8 lanes is a u8, so `shl 1` is an 8-bit shift.
!MAIN:
    let $s: simd[integer.unsigned.8_bit, count: 16] = simd.fill(value: 3);
    std::print.shortest(simd.max($s) shift_left 1, newline);
!end of MAIN
