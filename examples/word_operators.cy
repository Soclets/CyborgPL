# The operators are words (owner, 2026-10-02/03): `plus`, `minus`, `times`, and `.wrap` for the
# wrapping ones; a negative keeps `-`. The power is std's `pow` (bare or `std::pow`), which has no
# precedence yet, so it is bracketed beside another operator.
MAIN:
    let $a: i32 := 7;
    let $b: i32 := 3;
    std::println(($a + $b), " ", ($a - $b), " ", ($a x $b), " ", ($a ^ $b), " ", ($a std::pow 2), " ", -$a);
    let $h: u8 := 200;
    std::println(($h plus.wrap 100), " ", ($h times.wrap 2), " ", ($h pow.wrap 2));
    std::println((2 ^ 3) + 1, " ", 1 + 2 x 3);
;MAIN
