# Bare numbers alone in a fill are worked out exactly (check claude, seed
# 5120): 15059671820752541943 div 3 was left with an open width, done at a
# signed 64 bits where the literal is negative, and the asserted build
# stopped on it. 5019890606917513981 mod 256 is 253.
extern.C fn exit [$code: i32];
MAIN {
    let $s: simd[u64, count: 2] = simd.fill(value: 15059671820752541943 div.floor 3);
    let $x: u64 = $s[pos: 1];
    no_check my.exit(to.i32($x mod.floor 256));
}
