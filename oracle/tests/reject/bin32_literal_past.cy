# a literal just past f32's rounding limit does not fit (2026-09-24)
!MAIN:
    let $x: float.bin.32 := 340282356779733661637539395458142568448.0;
!end of MAIN
