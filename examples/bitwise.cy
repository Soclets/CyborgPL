# Bitwise operators (decided 2026-09-21): the boolean words, per bit.
!MAIN:
    let $a: integer.unsigned.8_bit = 12;                       # 0000 1100
    let $b: integer.unsigned.8_bit = 10;                       # 0000 1010
    std::print.shortest($a and.bit $b, newline);   # 0000 1000 = 8
    std::print.shortest($a or.bit $b, newline);    # 0000 1110 = 14
    std::print.shortest($a xor $b, newline);       # 0000 0110 = 6
    std::print.shortest(not.bit $a, newline);       # 1111 0011 = 243
    std::print.shortest($a shift_left 2, " ", $a shift_right 2, newline);   # 48 3
    variable $flags: integer.unsigned.8_bit = 0;
    change $flags = $flags or.bit 4;                # set bit 2
    change $flags = $flags or.bit 1;                # set bit 0
    std::print.shortest($flags, " ", ($flags and.bit 4) not_equal_to 0, newline);   # 5 true — parenthesised next to not_equal 
!end of MAIN
