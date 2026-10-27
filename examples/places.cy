# The places a `d` or a percent keeps, in its limits bracket (owner, 2026-10-17; `places` since 2026-10-20): `max_places`
# refuses a literal with more and asks a result that could have more to be rounded on purpose; `min_places` only
# pads when printing (5 as `5.00`). A percent's places count as written: `7.5%` has 1.
!MAIN:
    let $price: float.%decimal.64_bit[minimum: 0.0, maximum: 1000000.0, maximum_places: 2] = exactly 19.99;
    let $tax := exactly 7 percent;
    let $total: float.%decimal.64_bit[maximum_places: 2] = std::round.nearest($price times to.float.%decimal.64_bit($tax), places: 2);
    let $sum: float.%decimal.64_bit[maximum_places: 2] = $price plus $total;
    let $shown: float.%decimal.64_bit[minimum_places: 2] = exactly 5;
    let $rate: percent[float.%decimal.32_bit][minimum: 0 percent, maximum: 100 percent, maximum_places: 1] = exactly 7.5 percent;
    std::print.shortest($price, " ", $total, " ", $sum, " ", $shown, " ", $rate, newline);
!end of MAIN
