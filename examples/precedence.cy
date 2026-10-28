# Which operator binds first: `times` before `plus`, `pow` before both, and brackets.
MAIN:
    std::println((1 + 2 x 3));
    std::println((2 x (3 ^ 2)));
    std::println(-(2 ^ 2));
    std::println(((-2) ^ 2));
    std::println((10 - 4 - 3));
    std::println((6 div.floor (2 x 3)));
    std::println(((6 div.floor 2) x 3));
    let $a := 5;
    let $b := 7;
    if $a + 1 > $b - 2 & not ($a == $b) | false:
        std::println.stored("logic holds");
    ;if
;MAIN
