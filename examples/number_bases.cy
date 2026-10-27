# A whole number in another base is written with a word in front, as `exactly` is (owner, 2026-10-19: "(a)"):
# `binary`, `hex`, `octal`, with `_` grouping as in decimals. The same number either way: bin 1010 is 10.
!MAIN:
    let $flags: integer.unsigned.8_bit = binary 1010_0101;
    let $mask: integer.unsigned.32_bit = hexadecimal FF00_00FF;
    let $low: integer.unsigned.32_bit = hexadecimal ff;
    let $mode: integer.unsigned.16_bit = octal 755;
    let $n := negative hexadecimal 10;
    std::print.shortest($flags, " ", $mask, " ", $low, " ", $mode, " ", $n, " ", $flags and.bit binary 1111, newline);
!end of MAIN
