# The operators are words (owner, 2026-10-02/03): `plus`, `minus`, `times`, and `.wrap` for the
# wrapping ones; a negative keeps `-`. The power is std's `pow` (bare or `std::pow`), which has no
# precedence yet, so it is bracketed beside another operator.
MAIN {
    let 'a': int32 = 7;
    let 'b': int32 = 3;
    std::print.useful[('a' + 'b'), " ", ('a' - 'b'), " ", ('a' x 'b'), " ", ('a' xx 'b'), " ", ('a' std::pow 2), " ", -'a', newln];
    let 'h': uint8 = 200;
    std::print.useful[('h' plus.wrap 100), " ", ('h' times.wrap 2), " ", ('h' pow.wrap 2), newln];
    std::print.useful[(2 xx 3) + 1, " ", 1 + 2 x 3, newln];
}
