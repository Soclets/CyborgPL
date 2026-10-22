# A deci has no -0 (2026-10-07), so a zero deci or percent negated is 0, as the literal -0% is (check claude, seed 236).
MAIN {
    let 'tx' := exactly 0%;
    let 'z': d64 = exactly 0.0: d64;
    std::print.useful[-'tx', " ", -0%, " ", -23%, " ", -'z', newln];
}
