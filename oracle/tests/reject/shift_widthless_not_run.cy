# A shift with no width is refused where it never runs too (check claude, 2026-10-16, corpus dfce8e06)
MAIN {
    let $b: bool = false;
    let $v: i64 = 2;
    if $b and.then ((3 shr to.i64($v)) at_least 9) {
        std::print.useful("yes", newln);
    }
    std::print.useful("end", newln);
}
