# A keyword may name a function, reached by path, never bare (2026-09-19):
# bare `to` is the conversion and bare `true` the boolean, so these are `my.to()`, `my.true()`.
fn to [] -> i32:
    ret 5;
;fn to
fn true [] -> i32:
    ret 1;
;fn true
MAIN:
    let $x := my.to();
    let $y := my.true();
    std::println($x, " ", $y);
;MAIN
