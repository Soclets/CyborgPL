# `mod.floor` on a bin takes the divisor's sign, as on integers and decis (owner, 2026-10-02: a), so an
# angle stays 0 to 360; C's `fmod` (the dividend's sign) is `mod.trunc`, by name.
!MAIN:
    let $angle: float.binary.64_bit = negative 370.0;
    let $turn: float.binary.64_bit = 360.0;
    let $a: float.binary.64_bit = 7.5;
    let $b: float.binary.64_bit = negative 2.0;
    let $h: float.%brain.16_bit = negative 5.0;
    let $k: float.%brain.16_bit = 3.0;
    std::print.shortest($angle modulo.floor $turn, " ", $a modulo.floor $b, " ", $h modulo.floor $k, newline);
    std::print.shortest(($angle modulo.truncate $turn), " ", ($a modulo.truncate $b), newline);
    # a tiny negative would round up to the divisor itself; it is 0 instead, so the answer stays below 360
    let $tiny: float.binary.64_bit = negative 0.00000000000000000001;
    std::print.shortest($tiny modulo.floor $turn, newline);
!end of MAIN
