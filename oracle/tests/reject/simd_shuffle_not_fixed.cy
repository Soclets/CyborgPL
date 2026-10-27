# simd.shuffle's order is fixed numbers, known when compiling (2026-09-27)
!MAIN:
    let $v: simd[integer.signed.32_bit, count: 4] = simd.fill(value: 1);
    let $k := 2;
    let $r := simd.shuffle($v, [$k, 1, 3, 4]);
!end of MAIN
