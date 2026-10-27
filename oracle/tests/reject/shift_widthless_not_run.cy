# A shift with no width is refused where it never runs too (check claude, 2026-10-16, corpus dfce8e06)
!MAIN:
    let $b: boolean = false;
    let $v: integer.signed.64_bit = 2;
    if $b and.then ((3 shift_right to.integer.signed.64_bit($v)) at_least 9):
        std::print.shortest("yes", newline);
    end of if
    std::print.shortest("end", newline);
!end of MAIN
