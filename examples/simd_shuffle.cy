# `simd.shuffle` rearranges lanes (owner, 2026-09-27: a): the vector, then its new order as
# lane numbers, which must be fixed numbers known when compiling. Reversing, swapping and
# copying one lane everywhere are all just orders.
MAIN {
    let $xs: array[i32] = [10, 20, 30, 40];
    let $v: simd[i32, count: 4] = simd.load[lend $xs, pos: 1];
    let $r := simd.shuffle($v, [4, 3, 2, 1]);         # reversed
    let $pairs := simd.shuffle($v, [2, 1, 4, 3]);     # neighbours swapped
    let $first := simd.shuffle($v, [1, 1, 1, 1]);     # lane 1 everywhere
    std::print.useful($r, newln, $pairs, newln, $first, newln);
}
