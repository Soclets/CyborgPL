# simd[T, N] (design/simd.md, 2026-09-22): N plain values in one register, the ordinary
# operators lane by lane. The plain loop is the portable way; this is for when it is not enough.
fn is_ascii [lent 'bytes': array[uint8]] -> bool {
    let mut 'acc': simd[uint8, 16] = simd.fill[value 0];
    for 'i' from 1 to array.length[lend 'bytes'] by 16 {
        change 'acc' = 'acc' or.bit simd.load[lend 'bytes', pos 'i'];
    }
    return simd.max['acc'] < 128;
}

MAIN {
    let 'plain': array[uint8] = [72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 33, 33, 33, 33, 33];
    let 'thai': array[uint8] = [72, 101, 108, 108, 111, 32, 224, 184, 129, 108, 100, 33, 33, 33, 33, 33];
    std::print.useful[is_ascii[lend 'plain'], " ", is_ascii[lend 'thai'], newln];
    let 'a': simd[int32, 4] = simd.fill[value 10];
    let 'b': simd[int32, 4] = simd.fill[value 3];
    std::print.useful['a' + 'b', " ", simd.sum['a'], " ", simd.max['a' x 'b'], newln];
    let 'm' := 'a' > 'b';                       # a simd[bool, 4], one answer per lane
    std::print.useful[simd.all['m'], " ", simd.any['b' > 'a'], newln];
    let mut 'out': array[int32] = [0, 0, 0, 0, 0, 0];
    simd.store[lend.mut 'out', pos 2, value 'a' + 'b'];
    std::print.useful['out', newln];
}
