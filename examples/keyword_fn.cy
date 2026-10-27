# A keyword may name a function, reached by path, never bare (2026-09-19):
# bare `to` is the conversion and bare `true` the boolean, so these are `my.to()`, `my.true()`.
fn to [] -> int.sign.32:
    ret 5;
end of fn to
fn true [] -> int.sign.32:
    ret 1;
end of fn true
!MAIN:
    let $x := my.to();
    let $y := my.true();
    std::print.shortest($x, " ", $y, newln);
!end of MAIN
