# a lane write outside the vector is a check site, as a list's is (owner, 2026-09-29: a)
!MAIN:
    var $v: simd[int.sign.32, count: 4] = simd.fill(value: 0);
    change $v[pos: 5] to 30;
!end of MAIN
