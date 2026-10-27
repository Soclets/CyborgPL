# an unsigned type's range cannot start below zero (owner, 2026-09-23)
!MAIN:
    let $v: integer.unsigned.8_bit[minimum: negative 1, maximum: 5] = 5;
!end of MAIN
