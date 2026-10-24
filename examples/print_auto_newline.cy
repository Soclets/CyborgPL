# `std::print.useful.auto_newln` ends the line itself, so no `newln` is written (owner, 2026-09-30).
MAIN {
    std::print.useful.auto_newln("Hello, World!");
    let $n := 3;
    std::print.useful.auto_newln("n is ", $n);
    std::print.useful("the same, by hand: ", $n, newln);
}
