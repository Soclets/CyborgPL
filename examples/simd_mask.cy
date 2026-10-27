# `simd[bool, N]` is a type you can write (owner, 2026-09-27: a): a mask, one yes/no per
# lane, as a vector comparison gives. It has as many lanes as the vectors it compares, and
# `not`/`and`/`or` work lane by lane. A function can take one.
function count_on [$m: simd[boolean, count: 4]] -> boolean:
    return simd.all($m);
end of function count_on
!MAIN:
    let $xs: array[integer.signed.32_bit] = [5, 20, 30, 40];
    let $a: simd[integer.signed.32_bit, count: 4] = simd.load(lend $xs, position: 1);
    let $m: simd[boolean, count: 4] = $a greater_than simd.fill(value: 10);
    let $all: simd[boolean, count: 4] = simd.fill(value: true);
    let $both := $m and $all;
    let $flip := not $m;
    std::print.shortest(simd.any($m), " ", simd.all($m), " ", count_on($both), " ", simd.any($flip), newline);
!end of MAIN
