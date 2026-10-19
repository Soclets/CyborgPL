# One lane of a vector (simd.md: `'v'[1]`, 1-based; check claude found it
# gave garbage): the lane read had no shape in the generated code, the
# assignment was dropped without a word, and the variable was read unset.
# Lanes of each width, a position worked out at run time, and a load whose
# lanes differ, to see the order: 4 + 4 + (200 - 190) + (-5 + 10) +
# 2.5 x 2 + 30 = 58.
extern.C fn exit ['code': int32];
MAIN {
    let 'a': simd[int32, count 4] = simd.fill[value 4];
    let 'b': simd[uint8, count 16] = simd.fill[value 200];
    let 'c': simd[int64, count 2] = simd.fill[value -5];
    let 'w': simd[bin64, count 2] = simd.fill[value 2.5];
    let 'xs': list[int32] = [10, 20, 30, 40];
    let 'v': simd[int32, count 4] = simd.load[lend 'xs', pos 1];
    let 'k': int64 = 3;
    let 'x1': int32 = 'a'[pos 1];
    let 'x2': uint8 = 'b'[pos 16];
    let 'x3': int64 = 'c'[pos 2];
    let 'x4': int32 = 'a'[pos 'k' minus 1];
    let 'x5': bin64 = 'w'[pos 2];
    let 'x6': int32 = 'v'[pos 'k'];
    let 'half': int32 = to.int32['x5' times 2.0] otherwise 0;
    no_check my.exit['x1' plus 'x4' plus (to.int32['x2'] minus 190) plus to.int32['x3' plus 10] plus 'half' plus 'x6'];
}
