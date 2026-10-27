# a shift count is compared whole: 2^62 is not 0 (verification session, seed 1125)
!MAIN:
    let $v7: boolean = (to.integer.signed.64_bit(0) shift_right 4611686018427387904) greater_than 0;
!end of MAIN
