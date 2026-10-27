!MAIN:
    let $xs: array[integer.unsigned.8_bit] = [1, 2, 3]; let $v := simd.load(lend $xs, position: 1);
!end of MAIN
