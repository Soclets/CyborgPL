# `simd.sum` is worked out in full and checked where it lands, as any plain number; `simd.sum.lane`
# checks it at the lane's type, and `simd.sum.wrap` wraps it there as the chip's own sum does, never a
# check (owner, 2026-10-07: "There should be a way to do the other option too", then "yea").
!MAIN:
    let $v: simd[integer.signed.32_bit, count: 4] = simd.fill(value: 1200000000);
    let $s: integer.signed.64_bit = simd.sum($v);
    let $w: integer.signed.32_bit = simd.sum.wrap($v);
    let $u: simd[integer.signed.32_bit, count: 4] = simd.fill(value: negative 500000000);
    let $l: integer.signed.32_bit = simd.sum.lane($u);
    std::print.shortest($s, " ", $w, " ", $l, newline);
!end of MAIN
