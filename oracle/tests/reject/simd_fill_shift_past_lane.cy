# The value in `simd.fill` lands in a lane, so the lane's 8 bits are its width (check claude, seed 4106).
!MAIN:
    let $c: int.sign.8 := 9;
    let $v: simd[int.sign.8, count: 16] := simd.fill(value: ((neg 53) shr $c) mod.floor 23);
!end of MAIN
