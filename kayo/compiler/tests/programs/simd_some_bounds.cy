# `simd.load.some`'s k items must be in the list (owner, 2026-09-27: a): a
# count Precog cannot see is checked at run time, and past the end it traps.
extern.C fn getpid [] -> i32;
extern.C fn exit ['code': i32];
MAIN {
    let 'xs': std::list[i32] = [1, 2, 3];
    no_check let 'p': i32 = getpid[];
    let 'k': u64 = to.u64[('p' minus 'p') plus 3];
    let 'v': simd[i32, count: 4] = simd.load.some[lend 'xs', pos: 2, count: 'k'];
    no_check my.exit[simd.sum['v']];
}
