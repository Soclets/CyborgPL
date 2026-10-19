# An unsigned lane narrower than 64 bits, widened (check claude): a lane's
# type was `nothing` below MIR, so it was widened by sign, and a `uint8` of
# 200 read back as 2^64 - 56. 200 + 60000 + 4000000000 - 4000060000 = 200.
extern.C fn exit ['code': int32];
MAIN {
    let 'b': simd[uint8, count 16] = simd.fill[value 200];
    let 'u': uint64 = to.uint64['b'[pos 1]];
    let 'h': simd[uint16, count 8] = simd.fill[value 60000];
    let 'v': uint64 = to.uint64['h'[pos 8]];
    let 'w': simd[uint32, count 4] = simd.fill[value 4000000000];
    let 'x': uint64 = to.uint64['w'[pos 2]];
    no_check my.exit[to.int32[('u' plus 'v' plus 'x' minus 4000060000) mod.floor 256]];
}
