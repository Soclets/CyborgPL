# The value in `simd.fill` lands in a lane, so the lane's 8 bits are its width (check claude, seed 4106).
MAIN {
    let $c: i8 = 9;
    let $v: simd[i8, count: 16] = simd.fill[value: ((-53) shr $c) mod.floor 23];
}
