MAIN { let 'a': simd[int32, count 4] = simd.fill[value 1]; let 'b': simd[int32, count 4] = simd.fill[value 2]; if 'a' < 'b' { } }
