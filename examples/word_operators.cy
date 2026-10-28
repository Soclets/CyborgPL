# The operators (words 2026-10-02/03, symbols since 2026-10-28): `+`, `-`, `x`, and the words for the
# narrowings, `plus.wrap`, `times.wrap`; a negative is `-`. Power is std's, `std::pow` and `std::pow.wrap`.
MAIN:
    let $a: i32 := 7;
    let $b: i32 := 3;
    std::print(($a + $b), " ", ($a - $b), " ", ($a x $b), " ", ($a std::pow $b), " ", ($a std::pow 2), " ", -$a, newln);
    let $h: u8 := 200;
    std::print(($h plus.wrap 100), " ", ($h times.wrap 2), " ", ($h std::pow.wrap 2), newln);
    std::print((2 std::pow 3) + 1, " ", 1 + 2 x 3, newln);
;MAIN
