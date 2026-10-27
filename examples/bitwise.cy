# Bitwise operators (decided 2026-09-21): the boolean words, per bit.
!MAIN:
    let $a: int.unsign.8 = 12;                       # 0000 1100
    let $b: int.unsign.8 = 10;                       # 0000 1010
    std::print.shortest($a and.bit $b, newln);   # 0000 1000 = 8
    std::print.shortest($a or.bit $b, newln);    # 0000 1110 = 14
    std::print.shortest($a xor $b, newln);       # 0000 0110 = 6
    std::print.shortest(not.bit $a, newln);       # 1111 0011 = 243
    std::print.shortest($a shl 2, " ", $a shr 2, newln);   # 48 3
    var $flags: int.unsign.8 = 0;
    change $flags = $flags or.bit 4;                # set bit 2
    change $flags = $flags or.bit 1;                # set bit 0
    std::print.shortest($flags, " ", ($flags and.bit 4) not_eq 0, newln);   # 5 true — parenthesised next to not_equal 
!end of MAIN
