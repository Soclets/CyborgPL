# `xx` is the symbol of std's `pow` (2026-10-16), and a kit's operators, glyphs included, may carry its path (owner,
# 2026-10-22: "glyphs are optional too"), so `std::xx`, `xx` and `std::pow` are one operator.
!MAIN:
    let $a: int.sign.64 = 7;
    std::print.shortest($a std::pow 2, " ", $a pow 2, " ", $a std::pow 2, newln);
!end of MAIN
