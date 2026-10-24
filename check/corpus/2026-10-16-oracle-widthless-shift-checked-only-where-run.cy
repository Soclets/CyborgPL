# Finding 2026-10-16 (the corpus's 2026-10-03-precog-value-set-misses-a-call, revived in d4b6b976,
# cut down by hand): a shift with no width anywhere is refused by the oracle only where it runs.
# `3 shr to.i64['v']` in a comparison: the shifted number is bare, a comparison gives no width, the
# count never does (4866c834), so it is refused, as kayo refuses it when building. The oracle refuses
# it when 'b' is true and runs the program when 'b' is false, the shift not reached: like the
# const change, the return type and the let over a visible name before it (2026-10-08 to 10-10).
MAIN {
    let 'b': bool = false;
    let 'v': i64 = 2;
    if 'b' and.then ((3 shr to.i64('v')) at_least 9) {
        std::print.useful("yes", newln);
    }
    std::print.useful("end", newln);
}
