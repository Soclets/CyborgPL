# `simd[bool, N]` is a type you can write (owner, 2026-09-27: a): a mask, one yes/no per
# lane, as a vector comparison gives. It has as many lanes as the vectors it compares, and
# `not`/`and`/`or` work lane by lane. A function can take one.
fn count_on [$m: simd[bool, count: 4]] -> bool {
    return simd.all($m);
}
MAIN {
    let $xs: array[i32] = [5, 20, 30, 40];
    let $a: simd[i32, count: 4] = simd.load(lend $xs, pos: 1);
    let $m: simd[bool, count: 4] = $a > simd.fill(value: 10);
    let $all: simd[bool, count: 4] = simd.fill(value: true);
    let $both := $m and $all;
    let $flip := not $m;
    std::print.useful(simd.any($m), " ", simd.all($m), " ", count_on($both), " ", simd.any($flip), newln);
}
