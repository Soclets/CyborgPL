# simd[T, N] (simd.md): fill, load, store, the lane-wise operators with
# their checks, and the folds all, any, max, sum. Each true case adds its bit.
# A store used to leave Precog believing the list unchanged, and an `if`
# about a stored element was folded to false.
extern.C fn exit ['code': i32];
fn is_ascii [lent 'bytes': array[u8]] -> bool {
    let mut 'acc': simd[u8, count: 16] = simd.fill[value: 0];
    for 'i' [from: 1, to: array.length[lend 'bytes'], by: 16] {
        change 'acc' = 'acc' or.bit simd.load[lend 'bytes', pos: 'i'];
    }
    return simd.max['acc'] < 128;
}
MAIN {
    let 'plain': array[u8] = [72, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100, 33, 33, 33, 33, 33];
    let 'thai': array[u8] = [72, 101, 108, 108, 111, 32, 224, 184, 129, 108, 100, 33, 33, 33, 33, 33];
    let mut 'got': i32 = 0;
    if is_ascii[lend 'plain'] { change 'got' = 'got' plus 1; }
    if not is_ascii[lend 'thai'] { change 'got' = 'got' plus 2; }
    let 'a': simd[i32, count: 4] = simd.fill[value: 10];
    let 'b': simd[i32, count: 4] = simd.fill[value: 3];
    if simd.sum['a'] equal_to 40 { change 'got' = 'got' plus 4; }
    if simd.max['a' times 'b'] equal_to 30 { change 'got' = 'got' plus 8; }
    let 'm' := 'a' > 'b';
    if simd.all['m'] { change 'got' = 'got' plus 16; }
    if not simd.any['b' > 'a'] { change 'got' = 'got' plus 32; }
    let mut 'out': array[i32] = [0, 0, 0, 0, 0, 0];
    simd.store[lend.mut 'out', pos: 2, value: 'a' plus 'b'];
    if 'out'[pos: 1] equal_to 0 and 'out'[pos: 2] equal_to 13 and 'out'[pos: 5] equal_to 13 and 'out'[pos: 6] equal_to 0 { change 'got' = 'got' plus 64; }
    no_check my.exit['got'];
}
