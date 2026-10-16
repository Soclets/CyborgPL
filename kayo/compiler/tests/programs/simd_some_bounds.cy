# `simd.load.some`'s k items must be in the list (owner, 2026-09-27: a): a
# count Precog cannot see is checked at run time, and past the end it traps.
extern.C fn getpid [] -> int32;
extern.C fn exit ['code': int32];
MAIN {
    let 'xs': list[int32] = [1, 2, 3];
    no_check let 'p': int32 = getpid[];
    let 'k': uint64 = to.uint64[('p' minus 'p') plus 3];
    let 'v': simd[int32, 4] = simd.load.some[lend 'xs', pos 2, count 'k'];
    no_check my.exit[simd.sum['v']];
}
