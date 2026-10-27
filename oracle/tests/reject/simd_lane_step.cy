# a vector's lanes are checked at every operation at the lane's type (owner, 2026-09-29: b)
!MAIN:
    let $a: simd[integer.unsigned.8_bit, count: 16] = simd.fill(value: 200);
    let $b: simd[integer.unsigned.8_bit, count: 16] = ($a plus $a) minus $a;
!end of MAIN
