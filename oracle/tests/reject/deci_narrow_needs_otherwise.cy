# a d64 into a d32 says what happens when the digits do not fit (owner, 2026-09-29: a)
!MAIN:
    let $x: float.%decimal.64_bit = 1.5;
    let $y := to.float.%decimal.32_bit($x);
!end of MAIN
