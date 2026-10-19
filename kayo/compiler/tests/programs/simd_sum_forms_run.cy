# `simd.sum` worked out in full, checked where it lands (owner, 2026-10-07:
# "b"); `.lane` checked at the lane's type, `.wrap` wrapped there ("yea").
# 4800000000 into an int64; 505032704 wrapped; -2000000000 at int32; seed
# 30022's shape: the whole sum, then `mod.floor`.
extern.C fn exit ['code': int32];
fn big [] -> int32 { return 1200000000; }
MAIN {
    let 'v': simd[int32, count 4] = simd.fill[value big[]];
    let 's': int64 = simd.sum['v'];
    let 'w': int32 = simd.sum.wrap['v'];
    let 'u': simd[int32, count 4] = simd.fill[value -500000000];
    let 'l': int32 = simd.sum.lane['u'];
    let 'm' := simd.sum['v'] mod.floor 7;
    let mut 'got': int32 = 0;
    if 's' equal_to 4800000000 { change 'got' = 'got' plus 1; }
    if 'w' equal_to 505032704 { change 'got' = 'got' plus 2; }
    if 'l' equal_to -2000000000 { change 'got' = 'got' plus 4; }
    if 'm' equal_to (4800000000 mod.floor 7) { change 'got' = 'got' plus 8; }
    let 'b': simd[uint8, count 16] = simd.fill[value 200];
    if simd.sum['b'] equal_to 3200 and simd.sum.wrap['b'] equal_to 128 { change 'got' = 'got' plus 16; }
    no_check my.exit['got'];
}
