# A deci has no -0 (2026-10-07), so a zero deci or percent negated is 0, as the literal -0% is (check claude, seed 236).
MAIN:
    let $tx := exactly 0 percent;
    let $z: %d64 := exactly 0.0: %d64;
    std::println(-$tx, " ", -0 percent, " ", -23 percent, " ", -$z);
;MAIN
