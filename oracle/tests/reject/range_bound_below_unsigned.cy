# an unsigned type's range cannot start below zero (owner, 2026-09-23)
!MAIN:
    let $v: int.unsign.8[min: neg 1, max: 5] = 5;
!end of MAIN
