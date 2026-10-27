# `simd.sum` is worked out in full and checked where it lands, as any plain number; `simd.sum.lane`
# checks it at the lane's type, and `simd.sum.wrap` wraps it there as the chip's own sum does, never a
# check (owner, 2026-10-07: "There should be a way to do the other option too", then "yea").
!MAIN:
    let $v: simd[int.sign.32, count: 4] = simd.fill(value: 1200000000);
    let $s: int.sign.64 = simd.sum($v);
    let $w: int.sign.32 = simd.sum.wrap($v);
    let $u: simd[int.sign.32, count: 4] = simd.fill(value: neg 500000000);
    let $l: int.sign.32 = simd.sum.lane($u);
    std::print.shortest($s, " ", $w, " ", $l, newln);
!end of MAIN
