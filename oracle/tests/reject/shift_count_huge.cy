# a shift count is compared whole: 2^62 is not 0 (verification session, seed 1125)
!MAIN:
    let $v7: bool := (to.int.sign.64(0) shr 4611686018427387904) greater_than 0;
!end of MAIN
