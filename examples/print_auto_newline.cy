# `std::print.useful.auto_newln` ends the line itself, so no `newln` is written (owner, 2026-09-30).
MAIN:
    std::print("Hello, World!", newln);
    let $n := 3;
    std::print("n is ", $n, newln);
    std::print("the same, by hand: ", $n, newln);
;MAIN
