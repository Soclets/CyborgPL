# An unsigned lane narrower than 64 bits, widened (check claude): a lane's
# type was `nothing` below MIR, so it was widened by sign, and a `u8` of
# 200 read back as 2^64 - 56. 200 + 60000 + 4000000000 - 4000060000 = 200.
extern.C fn exit [$code: i32];
MAIN {
    let $b: simd[u8, count: 16] = simd.fill(value: 200);
    let $u: u64 = to.u64($b[pos: 1]);
    let $h: simd[u16, count: 8] = simd.fill(value: 60000);
    let $v: u64 = to.u64($h[pos: 8]);
    let $w: simd[u32, count: 4] = simd.fill(value: 4000000000);
    let $x: u64 = to.u64($w[pos: 2]);
    no_check my.exit(to.i32(($u plus $v plus $x minus 4000060000) mod.floor 256));
}
