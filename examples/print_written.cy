# `print.written` (owner, 2026-10-17): each value as its literal was typed, so an f16 holding 0.123456789 prints
# 0.123456789, where `print.stored` shows what the f16 holds and `print.useful` the shortest that reads back. A value
# that was not typed (computed, read, changed) has no written form: the print says what to do instead,
# `otherwise std::print.useful` (or `.stored`), or it is refused.
!MAIN:
    let $x: float.binary.16_bit = 0.123456789;
    let $tax := exactly 7.5 percent;
    std::print.stored($x, newline);
    std::print.shortest($x, newline);
    std::print.written($x, " ", $tax, " ", negative 2.50, " ", exactly 51.0 percent, newline);
    let $y := $x plus 0.5: float.binary.16_bit;
    std::print.written($y, newline) otherwise std::print.shortest;
    variable $z: float.binary.64_bit = 1.25;
    change $z = $z times 2.0;
    std::print.written($z, newline) otherwise std::print.stored;
!end of MAIN
