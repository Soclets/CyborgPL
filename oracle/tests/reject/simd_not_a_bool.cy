!MAIN:
    let $a: simd[int.sign.32, count: 4] = simd.fill(value: 1); let $b: simd[int.sign.32, count: 4] = simd.fill(value: 2); if $a less_than $b:
    end of if
!end of MAIN
