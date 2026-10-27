# A branch `on.target` rules out is "not compiled at all" (owner, 2026-10-06), so a 256-bit vector
# there is not judged against this 128-bit target; the hand-written fallback runs (owner,
# 2026-10-07: a vector the target lacks is refused, and a fallback is written by hand).
!MAIN:
    on.target.chip x86_64:
        let $w: simd[integer.signed.32_bit, count: 8] = simd.fill(value: 1);
    end of on.target.chip
    else:
        let $v: simd[integer.signed.32_bit, count: 4] = simd.fill(value: 1);
        std::print.shortest(simd.sum($v), newline);
    end of else
!end of MAIN
