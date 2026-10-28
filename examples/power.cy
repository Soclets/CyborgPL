# `^`, the power, and how it groups (to the right).
MAIN:
    std::print((2 ^ (3 ^ 2)), newln);
    std::print(((2 ^ 3) ^ 2), newln);
    let $kilo := 2 ^ 10;
    std::print($kilo, newln);
    std::print((10 - 3 - 2), newln);
;MAIN
