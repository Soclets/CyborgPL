# `xx` was the symbol of std's `pow` (2026-10-16), and a kit's operators, glyphs included, may carry its path (owner,
# 2026-10-22: "glyphs are optional too"). `xx` is gone and `^` is not an operator (2026-10-28): power is `std::pow`.
MAIN:
    let $a: i64 := 7;
    std::print($a std::pow 2, " ", $a std::pow 2, " ", $a std::pow 2, newln);
;MAIN
