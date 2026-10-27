# `mod.floor` on a bin takes the divisor's sign, as on integers and decis (owner, 2026-10-02: a), so an
# angle stays 0 to 360; C's `fmod` (the dividend's sign) is `mod.trunc`, by name.
!MAIN:
    let $angle: float.bin.64 = neg 370.0;
    let $turn: float.bin.64 = 360.0;
    let $a: float.bin.64 = 7.5;
    let $b: float.bin.64 = neg 2.0;
    let $h: float.%brain.16 = neg 5.0;
    let $k: float.%brain.16 = 3.0;
    std::print.shortest($angle mod.floor $turn, " ", $a mod.floor $b, " ", $h mod.floor $k, newln);
    std::print.shortest(($angle mod.trunc $turn), " ", ($a mod.trunc $b), newln);
    # a tiny negative would round up to the divisor itself; it is 0 instead, so the answer stays below 360
    let $tiny: float.bin.64 = neg 0.00000000000000000001;
    std::print.shortest($tiny mod.floor $turn, newln);
!end of MAIN
