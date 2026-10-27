# `xx` is the symbol of std's `pow` (2026-10-16), and a kit's operators, glyphs included, may carry its path (owner,
# 2026-10-22: "glyphs are optional too"), so `std::xx`, `xx` and `std::pow` are one operator.
!MAIN:
    let $a: integer.signed.64_bit = 7;
    std::print.shortest($a std::power 2, " ", $a power 2, " ", $a std::power 2, newline);
!end of MAIN
