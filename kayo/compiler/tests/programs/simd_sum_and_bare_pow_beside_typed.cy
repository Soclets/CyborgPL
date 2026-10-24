# Two findings of check claude's (corpus 12018c46). A group of bare numbers
# beside a typed operand is worked at the size its own values reach: the
# pow passes 400 bits in between (seed 32641; 92 as the oracle). A
# `simd.sum` gives way to a typed operand's width and otherwise works at its
# lane's (seed 32250): 5 xor 0 is 5 at i8, and two sums of i32 lanes
# xor at i32, 48 xor 47 = 31. 92 + 5 + 31 = 128.
extern.C fn exit [$code: i32];
fn three [] -> i32 { return 3; }
MAIN {
    let $p := three();
    let $x := to.i64($p) plus (((-656682024) pow 15) mod.floor 2147483647);
    let $a := 5;
    let $v: simd[i32, count: 4] = simd.fill(value: 1200000000);
    let $w: simd[i32, count: 4] = simd.fill(value: 12);
    let $y := to.i8($a) xor (simd.sum($v) minus 4800000000);
    let $z := (simd.sum($w) plus 0) xor (simd.sum($w) minus 1);
    no_check my.exit(to.i32(($x mod.floor 256) plus $y plus $z));
}
