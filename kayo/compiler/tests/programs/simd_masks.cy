# `simd[bool, N]` is a type you can write (owner, 2026-09-27: a): a function
# takes one, `simd.fill[true]` makes one, and `not`/`and`/`or` work lane by
# lane. The answers are examples/simd_mask.cpls', read out as an exit code.
extern.C fn exit [$code: i32];

fn all_on [$m: simd[bool, count: 4]] -> bool {
    return simd.all($m);
}

fn flag [$b: bool, $v: i32] -> i32 {
    if $b { return $v; }
    return 0;
}

MAIN {
    let $xs: array[i32] = [5, 20, 30, 40];
    let $a: simd[i32, count: 4] = simd.load(lend $xs, pos: 1);
    let $m: simd[bool, count: 4] = $a > simd.fill(value: 10);
    let $all: simd[bool, count: 4] = simd.fill(value: true);
    let $both := $m and $all;
    let $flip := not $m;
    let $either := $flip or $m;
    let $few: simd[bool, count: 16] = simd.fill(value: false);
    no_check my.exit(flag(simd.any($m), 1) plus flag(simd.all($m), 2) plus flag(all_on($both), 4) plus flag(simd.any($flip), 8) plus flag(simd.all($either), 16) plus flag(simd.any($few), 32));   # 1 + 8 + 16 = 25
}
