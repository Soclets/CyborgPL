# `print.written` (owner, 2026-10-17): each value as its literal was typed, so an f16 holding 0.123456789 prints
# 0.123456789, where `print.stored` shows what the f16 holds and `print.useful` the shortest that reads back. A value
# that was not typed (computed, read, changed) has no written form: the print says what to do instead,
# `otherwise std::print.useful` (or `.stored`), or it is refused.
!MAIN:
    let $x: float.bin.16 = 0.123456789;
    let $tax := exactly 7.5 percent;
    std::print.stored($x, newln);
    std::print.shortest($x, newln);
    std::print.written($x, " ", $tax, " ", neg 2.50, " ", exactly 51.0 percent, newln);
    let $y := $x plus 0.5: float.bin.16;
    std::print.written($y, newln) otherwise std::print.shortest;
    var $z: float.bin.64 = 1.25;
    change $z = $z times 2.0;
    std::print.written($z, newln) otherwise std::print.stored;
!end of MAIN
