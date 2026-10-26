# Finding 2026-10-22 (generator, seed 236, the percent episode): a `deci` has no `-0` and there is no
# `-0` anywhere outside the `.ieee` forms (owner, 2026-10-07, DECISIONS.md "Then no `-0` anywhere").
# `-'tx'` of `exactly 0%` prints "0%" in kayo; the oracle prints "-0%".
MAIN {
    let 'tx' := exactly 0 percent;
    std::print.useful(-'tx', " ", -0 percent, " ", -23 percent, newln);
}
