# Which operator binds first: `x` before `+`, `std::pow` before both, and brackets.
MAIN:
    std::print((1 + 2 x 3), newln);
    std::print((2 x (3 std::pow 2)), newln);
    std::print(-(2 std::pow 2), newln);
    std::print(((-2) std::pow 2), newln);
    std::print((10 - 4 - 3), newln);
    std::print((6 div.floor (2 x 3)), newln);
    std::print(((6 div.floor 2) x 3), newln);
    let $a := 5;
    let $b := 7;
    if $a + 1 > $b - 2 and not ($a == $b) or false:
        std::print.stored("logic holds", newln);
    ;if
;MAIN
