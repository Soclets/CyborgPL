!MAIN:
    let $v: simd[int.sign.32, count: 4] = simd.fill(value: 1200000000);
    let $s: int.sign.64 = simd.sum.lane($v);
!end of MAIN
