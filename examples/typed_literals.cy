# A typed literal: `5: u8` (owner, 2026-10-17: "yea", and "(a)": no brackets). The `:` covers only the
# literal right before it, as it covers only the name in `$x: u8`; a variable is converted with `to.…[…]`.
!MAIN:
    # a shift's count gives nothing inside it a width, so the inner shift says its own
    let $r: integer.signed.64_bit = to.integer.signed.64_bit(1) shift_left (5: integer.unsigned.8_bit shift_right 1);
    std::print.shortest($r, newline);
    let $a := 200: integer.unsigned.8_bit;
    std::print.shortest($a plus 55, newline);
    let $n := negative 128: integer.signed.8_bit;
    std::print.shortest($n, newline);
    std::print.shortest(3: integer.unsigned.8_bit times 2, newline);
    let $d := exactly 0.1: float.%decimal.64_bit;
    std::print.shortest($d plus exactly 0.2, newline);
    let $small := exactly 2.5: float.%decimal.32_bit;
    std::print.shortest($small, newline);
    let $p := 5: integer.unsigned.64_bit[minimum: 1];
    std::print.shortest($p, newline);
!end of MAIN
