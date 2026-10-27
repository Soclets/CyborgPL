# A keyword may name a function, reached by path, never bare (2026-09-19):
# bare `to` is the conversion and bare `true` the boolean, so these are `my.to()`, `my.true()`.
function to [] -> integer.signed.32_bit:
    return 5;
end of function to
function true [] -> integer.signed.32_bit:
    return 1;
end of function true
!MAIN:
    let $x := my.to();
    let $y := my.true();
    std::print.shortest($x, " ", $y, newline);
!end of MAIN
