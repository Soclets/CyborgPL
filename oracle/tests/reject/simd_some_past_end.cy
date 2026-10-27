# simd.load.some's k items must be in the list (2026-09-27)
!MAIN:
    let $xs: std::list[integer.signed.32_bit] = [1, 2, 3];
    let $v: simd[integer.signed.32_bit, count: 4] = simd.load.some(lend $xs, position: 2, count: 3);
!end of MAIN
