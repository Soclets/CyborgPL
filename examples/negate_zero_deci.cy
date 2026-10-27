# A deci has no -0 (2026-10-07), so a zero deci or percent negated is 0, as the literal -0% is (check claude, seed 236).
!MAIN:
    let $tx := exactly 0 percent;
    let $z: float.%deci.64 := exactly 0.0: float.%deci.64;
    std::print.shortest(neg $tx, " ", neg 0 percent, " ", neg 23 percent, " ", neg $z, newln);
!end of MAIN
