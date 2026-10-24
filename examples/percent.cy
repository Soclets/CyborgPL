# Percentages (owner, 2026-10-16): `80%` or `80 percent` is the number over 100, and `exactly 80%` exact, as every
# decimal has its `exactly` form; any digits, `exactly 12.3456789%`. Under `:=` it is a `percent`, its own number
# family (owner, 2026-10-17): percents combine with percents, a plain number is converted to or from one, and only
# `print.useful` shows the `%`. Into a plain number's place it is that number (`let $f: f64 = 80% x 2.0;` is 1.6).
MAIN {
    let $price := exactly 19.99;
    let $tax := exactly 7%;
    let $off := exactly 12.5 percent;
    std::print.useful($price x to.d64($tax), " ", $price x to.d64($off), newln);
    std::print.useful(80%, " ", 5%, " ", 100%, " ", exactly 0.0001%, " ", exactly 12.3456789%, newln);
    let $f: f64 = 80% x 2.0;
    std::print.useful($f, newln);
    let $both := $tax + $off;
    let $rate: percent[d32][min: 0%, max: 100%] = exactly 12.25%;
    std::print.useful($both, " ", $rate, " ", to.percent[d64](exactly 0.07), " ", $tax > $off, newln);
    std::print.stored($tax, " ", $rate, newln);
    let $g: f64 = 2.0;
    let $sym: %[d32] = exactly 2.5%;
    std::print.useful($g + 80%, " ", -$tax, " ", $sym, newln);   # a `%` literal beside a plain number is its fraction
}
