# a range's bounds are values of its type: no u8 is 300 (owner, 2026-09-23)
!MAIN:
    let $v: int.unsign.8[min: 0, max: 300] = 5;
!end of MAIN
