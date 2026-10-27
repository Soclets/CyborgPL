# Rounding is one family, `decimals` optional (owner, 2026-10-17: "(c)"); a bare `round` is refused, since it
# names no way to round a tie (owner: "(a)"). A bin is rounded from its exact digits: 2.675 is a little under.
!MAIN:
    let $y: float.bin.64 = 2.5;
    std::print.shortest(std::round.near($y), " ", std::round.half_up($y), " ", std::round.floor($y), " ", std::round.ceil($y), " ", std::round.trunc(0.0 minus $y), newln);
    std::print.shortest(std::round.near(2.675: float.bin.64, places: 2), " ", std::round.floor(neg 1.234: float.bin.64, places: 1), newln);
    let $d := exactly 2.675;
    std::print.shortest(std::round.near($d, places: 2), " ", std::round.half_up($d, places: 2), " ", std::round.floor(0.0 minus $d, places: 2), " ", std::round.ceil($d, places: 1), newln);
    std::print.shortest(std::round.near(exactly 2.665: float.%deci.64, places: 2), " ", std::round.trunc(exactly 2.665: float.%deci.64, places: 1), newln);
!end of MAIN
