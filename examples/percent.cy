# Percentages (owner, 2026-10-16): `80%` or `80 percent` is the number over 100, and `exactly 80%` exact, as every
# decimal has its `exactly` form; any digits, `exactly 12.3456789%`. Under `:=` it is a `percent`, its own number
# family (owner, 2026-10-17): percents combine with percents, a plain number is converted to or from one, and only
# `print.useful` shows the `%`. Into a plain number's place it is that number (`let $f: f64 = 80% x 2.0;` is 1.6).
!MAIN:
    let $price := exactly 19.99;
    let $tax := exactly 7 percent;
    let $off := exactly 12.5 percent;
    std::print.shortest($price times to.float.%decimal.64_bit($tax), " ", $price times to.float.%decimal.64_bit($off), newline);
    std::print.shortest(80 percent, " ", 5 percent, " ", 100 percent, " ", exactly 0.0001 percent, " ", exactly 12.3456789 percent, newline);
    let $f: float.binary.64_bit = 80 percent times 2.0;
    std::print.shortest($f, newline);
    let $both := $tax plus $off;
    let $rate: percent[float.%decimal.32_bit][minimum: 0 percent, maximum: 100 percent] = exactly 12.25 percent;
    std::print.shortest($both, " ", $rate, " ", to.percent[float.%decimal.64_bit](exactly 0.07), " ", $tax greater_than $off, newline);
    std::print.stored($tax, " ", $rate, newline);
    let $g: float.binary.64_bit = 2.0;
    let $sym: percent[float.%decimal.32_bit] = exactly 2.5 percent;
    std::print.shortest($g plus 80 percent, " ", negate $tax, " ", $sym, newline);   # a `%` literal beside a plain number is its fraction
!end of MAIN
