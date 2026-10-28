# `simd.sum` is worked out in full and checked where it lands, as any plain number; `simd.sum.lane`
# checks it at the lane's type, and `simd.sum.wrap` wraps it there as the chip's own sum does, never a
# check (owner, 2026-10-07: "There should be a way to do the other option too", then "yea").
MAIN:
    let $v: simd[i32, count: 4] := simd.fill(value: 1200000000);
    let $s: i64 := simd.sum($v);
    let $w: i32 := simd.sum.wrap($v);
    let $u: simd[i32, count: 4] := simd.fill(value: -500000000);
    let $l: i32 := simd.sum.lane($u);
    std::print($s, " ", $w, " ", $l, newln);
;MAIN
