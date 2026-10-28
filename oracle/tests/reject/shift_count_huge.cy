# a shift count is compared whole: 2^62 is not 0 (verification session, seed 1125)
MAIN:
    let $v7: bool := (to.i64(0) shr 4611686018427387904) > 0;
;MAIN
