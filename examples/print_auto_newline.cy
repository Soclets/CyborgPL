# `std::print.useful.auto_newln` ends the line itself, so no `newln` is written (owner, 2026-09-30).
MAIN:
    std::println("Hello, World!");
    let $n := 3;
    std::println("n is ", $n);
    std::println("the same, by hand: ", $n);
;MAIN
