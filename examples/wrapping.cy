# Wrapping arithmetic (decided 2026-09-20): plus.wrap minus.wrap times.wrap pow.wrap are always defined — modulo the
# type's size — so a hash needs no no_check. Plain plus minus times pow stay overflow-checked.
function hash [lent $text: string.utf8] -> integer.unsigned.32_bit:
    variable $h: integer.unsigned.32_bit = 2_166_136_261;                 # FNV-1a
    for $b in std::string.each.byte(lend $text):
        change $h = $h xor $b;                          # FNV-1a: xor, then a wrapping multiply
        change $h = $h times.wrap 16_777_619;                    # wraps instead of overflowing
    end of for
    return $h;
end of function hash

!MAIN:
    std::print.shortest(hash("hello"), newline);
    variable $c: integer.unsigned.8_bit = 250;
    change $c = $c plus.wrap 10;                                # 260 wraps to 4
    std::print.shortest($c, newline);
    variable $i: integer.signed.8_bit = 127;
    change $i = $i plus.wrap 1;                                 # 128 wraps to -128
    std::print.shortest($i, newline);
    variable $p: integer.unsigned.16_bit = 3;
    change $p = $p pow.wrap 20;                               # 3^20 modulo 65536
    std::print.shortest($p, newline);
!end of MAIN
