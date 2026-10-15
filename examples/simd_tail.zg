# The leftover end of a SIMD loop (owner, 2026-09-27: a): `simd.load.some` loads k items and
# leaves the other lanes zero; `simd.store.some` writes only the first k lanes. So a list of
# any length is one vector loop. k is 0 to N, and the k items must be in the list (a check
# site the prover proves, as for `simd.load`).
MAIN {
    let mut 'xs': list[int32] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let 'n' := list.length[lend 'xs'];
    let mut 'acc': simd[int32, 4] = simd.fill[value 0];
    let mut 'i': uint64 = 1;
    while 'i' at_most 'n' {
        let 'left' := 'n' minus 'i' plus 1;
        if 'left' at_least 4 {
            let 'v': simd[int32, 4] = simd.load[lend 'xs', pos 'i'];
            change 'acc' = 'acc' plus 'v';
        } else {
            let 'v': simd[int32, 4] = simd.load.some[lend 'xs', pos 'i', count 'left'];
            change 'acc' = 'acc' plus 'v';
        }
        change 'i' = 'i' plus 4;
    }
    let 'twice': simd[int32, 4] = simd.fill[value 2];
    let 'last': simd[int32, 4] = simd.load[lend 'xs', pos 7];
    simd.store.some[lend.mut 'xs', pos 9, count 2, value 'last' times 'twice'];
    std::print.useful[simd.sum['acc'], " ", 'xs', newln];
}
