# a range's bounds are values of its type: no u8 is 300 (owner, 2026-09-23)
!MAIN:
    let $v: integer.unsigned.8_bit[minimum: 0, maximum: 300] = 5;
!end of MAIN
