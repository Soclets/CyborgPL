!MAIN:
    let $v: simd[integer.signed.32_bit, count: 4] = simd.fill(value: 1200000000);
    let $s: integer.signed.64_bit = simd.sum.lane($v);
!end of MAIN
