# `op -` subtracts; negation as a value is `op neg` (owner, 2026-09-29)
!MAIN:
    let $k: fn[int.sign.32] -> int.sign.32 = op minus;
!end of MAIN
