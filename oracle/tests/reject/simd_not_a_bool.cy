!MAIN:
    let $a: simd[integer.signed.32_bit, count: 4] = simd.fill(value: 1); let $b: simd[integer.signed.32_bit, count: 4] = simd.fill(value: 2); if $a less_than $b:
    end of if
!end of MAIN
