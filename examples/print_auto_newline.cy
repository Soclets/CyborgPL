# `std::print.useful.auto_newln` ends the line itself, so no `newln` is written (owner, 2026-09-30).
!MAIN:
    std::print.shortest.auto_newline("Hello, World!");
    let $n := 3;
    std::print.shortest.auto_newline("n is ", $n);
    std::print.shortest("the same, by hand: ", $n, newln);
!end of MAIN
