# `simd.fill` of an answer of plain math (check claude, 163 of 300
# programs): the answer is exact, and was made the lane type — 65-bit lanes,
# one to a register, `simd[integer, count 1]` — so a `simd[i32, count 4]` refused it.
# It now lands in every lane of the type the vector gets, and is asked there.
# And a vector nothing gives a lane width (`:=` of bare numbers) has the
# natural one, 64 bits, two lanes; it had none. 6 + 2 + 25 + 5 + 6 = 44.
extern.C fn exit [$code: i32];
MAIN {
    let $x: i32 = 5;
    let $a: simd[i32, count: 4] = simd.fill(value: $x plus 1);
    let $b: simd[i32, count: 4] = simd.fill(value: ($x mod.floor 7) minus 3);
    let $c: simd[i16, count: 8] = simd.fill(value: $x pow 2);
    let $d := simd.fill(value: 5);
    let $e := simd.fill(value: $x plus 1);
    let $d2: i64 = $d[pos: 2];
    let $e1: i64 = $e[pos: 1];
    no_check my.exit($a[pos: 4] plus $b[pos: 1] plus to.i32($c[pos: 8]) plus to.i32($d2) plus to.i32($e1));
}
