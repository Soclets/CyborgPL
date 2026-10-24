MAIN { let $xs: array[u8] = [1, 2, 3]; let $v := simd.load[lend $xs, pos: 1]; }
