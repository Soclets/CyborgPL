# Bitwise operators (decided 2026-09-21): the boolean words, per bit.
MAIN:
    let $a: u8 := 12;                       # 0000 1100
    let $b: u8 := 10;                       # 0000 1010
    std::println($a and.bit $b);   # 0000 1000 = 8
    std::println($a or.bit $b);    # 0000 1110 = 14
    std::println($a xor $b);       # 0000 0110 = 6
    std::println(not.bit $a);       # 1111 0011 = 243
    std::println($a shl 2, " ", $a shr 2);   # 48 3
    var $flags: u8 = 0;
    change $flags to $flags or.bit 4;                # set bit 2
    change $flags to $flags or.bit 1;                # set bit 0
    std::println($flags, " ", ($flags and.bit 4) != 0);   # 5 true — parenthesised next to not_equal 
;MAIN
