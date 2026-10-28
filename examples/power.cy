# Power, std's `std::pow` (`^` is not an operator), and how it groups (to the right).
MAIN:
    std::print((2 std::pow (3 std::pow 2)), newln);
    std::print(((2 std::pow 3) std::pow 2), newln);
    let $kilo := 2 std::pow 10;
    std::print($kilo, newln);
    std::print((10 - 3 - 2), newln);
;MAIN
