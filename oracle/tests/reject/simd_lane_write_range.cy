# a lane write outside the vector is a check site, as a list's is (owner, 2026-09-29: a)
!MAIN:
    variable $v: simd[integer.signed.32_bit, count: 4] = simd.fill(value: 0);
    change $v[position: 5] = 30;
!end of MAIN
