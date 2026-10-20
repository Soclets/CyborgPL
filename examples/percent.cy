# Percentages (owner, 2026-10-16): `80%` or `80 percent` is the number over 100, a bin as `0.80` is, and
# `exactly 80%` an exact deci, as every decimal has its `exactly` form. Any digits: `exactly 12.3456789%`.
MAIN {
    let 'price' := exactly 19.99;
    let 'tax' := exactly 7%;
    let 'off' := exactly 12.5 percent;
    std::print.useful['price' x 'tax', " ", 'price' x 'off', newln];
    std::print.useful[80%, " ", 5%, " ", 100%, " ", exactly 0.0001%, " ", exactly 12.3456789%, newln];
    let 'f': f64 = 80% x 2.0;
    std::print.useful['f', newln];
}
