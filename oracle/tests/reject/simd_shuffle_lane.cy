# a shuffle names lanes that exist (2026-09-27)
!MAIN:
    let $v: simd[integer.signed.32_bit, count: 4] = simd.fill(value: 1);
    let $r := simd.shuffle($v, [5, 1, 3, 4]);
!end of MAIN
