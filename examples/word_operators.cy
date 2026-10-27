# The operators are words (owner, 2026-10-02/03): `plus`, `minus`, `times`, and `.wrap` for the
# wrapping ones; a negative keeps `-`. The power is std's `pow` (bare or `std::pow`), which has no
# precedence yet, so it is bracketed beside another operator.
!MAIN:
    let $a: integer.signed.32_bit = 7;
    let $b: integer.signed.32_bit = 3;
    std::print.shortest(($a plus $b), " ", ($a minus $b), " ", ($a times $b), " ", ($a power $b), " ", ($a std::power 2), " ", negate $a, newline);
    let $h: integer.unsigned.8_bit = 200;
    std::print.shortest(($h plus.wrap 100), " ", ($h times.wrap 2), " ", ($h pow.wrap 2), newline);
    std::print.shortest((2 power 3) plus 1, " ", 1 plus 2 times 3, newline);
!end of MAIN
