# The operators are words (owner, 2026-10-02/03): `plus`, `minus`, `times`, and `.wrap` for the
# wrapping ones; a negative keeps `-`. The power is std's `pow` (bare or `std::pow`), which has no
# precedence yet, so it is bracketed beside another operator.
!MAIN:
    let $a: int.sign.32 = 7;
    let $b: int.sign.32 = 3;
    std::print.shortest(($a plus $b), " ", ($a minus $b), " ", ($a times $b), " ", ($a pow $b), " ", ($a std::pow 2), " ", neg $a, newln);
    let $h: int.unsign.8 = 200;
    std::print.shortest(($h plus.wrap 100), " ", ($h times.wrap 2), " ", ($h pow.wrap 2), newln);
    std::print.shortest((2 pow 3) plus 1, " ", 1 plus 2 times 3, newln);
!end of MAIN
