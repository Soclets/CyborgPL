# Rounding is one family, `decimals` optional (owner, 2026-10-17: "(c)"); a bare `round` is refused, since it
# names no way to round a tie (owner: "(a)"). A bin is rounded from its exact digits: 2.675 is a little under.
!MAIN:
    let $y: float.binary.64_bit = 2.5;
    std::print.shortest(std::round.nearest($y), " ", std::round.half_up($y), " ", std::round.floor($y), " ", std::round.ceiling($y), " ", std::round.truncate(0.0 minus $y), newline);
    std::print.shortest(std::round.nearest(2.675: float.binary.64_bit, places: 2), " ", std::round.floor(negative 1.234: float.binary.64_bit, places: 1), newline);
    let $d := exactly 2.675;
    std::print.shortest(std::round.nearest($d, places: 2), " ", std::round.half_up($d, places: 2), " ", std::round.floor(0.0 minus $d, places: 2), " ", std::round.ceiling($d, places: 1), newline);
    std::print.shortest(std::round.nearest(exactly 2.665: float.%decimal.64_bit, places: 2), " ", std::round.truncate(exactly 2.665: float.%decimal.64_bit, places: 1), newline);
!end of MAIN
