# The value in `simd.fill` lands in a lane, so the lane's 8 bits are its width (check claude, seed 4106).
!MAIN:
    let $c: integer.signed.8_bit = 9;
    let $v: simd[integer.signed.8_bit, count: 16] = simd.fill(value: ((negative 53) shift_right $c) modulo.floor 23);
!end of MAIN
