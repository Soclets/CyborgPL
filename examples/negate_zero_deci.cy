# A deci has no -0 (2026-10-07), so a zero deci or percent negated is 0, as the literal -0% is (check claude, seed 236).
!MAIN:
    let $tx := exactly 0 percent;
    let $z: float.%decimal.64_bit = exactly 0.0: float.%decimal.64_bit;
    std::print.shortest(negate $tx, " ", negative 0 percent, " ", negative 23 percent, " ", negate $z, newline);
!end of MAIN
