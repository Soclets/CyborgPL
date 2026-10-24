# The places a `d` or a percent keeps, in its limits bracket (owner, 2026-10-17; `places` since 2026-10-20): `max_places`
# refuses a literal with more and asks a result that could have more to be rounded on purpose; `min_places` only
# pads when printing (5 as `5.00`). A percent's places count as written: `7.5%` has 1.
MAIN {
    let $price: d64[min: 0.0, max: 1000000.0, max_places: 2] = exactly 19.99;
    let $tax := exactly 7%;
    let $total: d64[max_places: 2] = std::round.near($price x to.d64($tax), places: 2);
    let $sum: d64[max_places: 2] = $price + $total;
    let $shown: d64[min_places: 2] = exactly 5;
    let $rate: percent[d32][min: 0%, max: 100%, max_places: 1] = exactly 7.5%;
    std::print.useful($price, " ", $total, " ", $sum, " ", $shown, " ", $rate, newln);
}
