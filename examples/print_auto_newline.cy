# `std::print.useful.auto_newln` ended the line itself (owner, 2026-09-30); it was retired 2026-10-27, and every line
# now ends with `newln`.
MAIN:
    std::print("Hello, World!", newln);
    let $n := 3;
    std::print("n is ", $n, newln);
    std::print("the same, by hand: ", $n, newln);
;MAIN
