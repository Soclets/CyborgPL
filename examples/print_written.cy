# `print.written` (owner, 2026-10-17): each value as its literal was typed, so an f16 holding 0.123456789 prints
# 0.123456789, where `print.stored` shows what the f16 holds and `print.shortest` the shortest that reads back. A value
# that was not typed (computed, read, changed) has no written form: the print says what to do instead,
# `otherwise std::print.shortest` (or `.stored`), or it is refused.
MAIN:
    let $x: f16 := 0.123456789;
    let $tax := exactly 7.5 percent;
    std::print.stored($x, newln);
    std::print($x, newln);
    std::print.written($x, " ", $tax, " ", -2.50, " ", exactly 51.0 percent, newln);
    let $y := $x + 0.5: f16;
    std::print.written($y, newln) otherwise std::print.shortest;
    var $z: f64 = 1.25;
    change $z to $z x 2.0;
    std::print.written($z, newln) otherwise std::print.stored;
;MAIN
