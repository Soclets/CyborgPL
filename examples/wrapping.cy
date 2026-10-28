# Wrapping arithmetic (decided 2026-09-20): plus.wrap minus.wrap times.wrap pow.wrap are always defined — modulo the
# type's size — so a hash needs no no_check. Plain plus minus times pow stay overflow-checked.
fn hash [lent $text: str.utf8] -> u32:
    var $h: u32 = 2_166_136_261;                 # FNV-1a
    for $b in std::str.each.byte(lend $text):
        change $h to $h xor $b;                          # FNV-1a: xor, then a wrapping multiply
        change $h to $h times.wrap 16_777_619;                    # wraps instead of overflowing
    ;for
    ret $h;
;fn hash

MAIN:
    std::print(hash("hello"), newln);
    var $c: u8 = 250;
    change $c to $c plus.wrap 10;                                # 260 wraps to 4
    std::print($c, newln);
    var $i: i8 = 127;
    change $i to $i plus.wrap 1;                                 # 128 wraps to -128
    std::print($i, newln);
    var $p: u16 = 3;
    change $p to $p std::pow.wrap 20;                               # 3^20 modulo 65536
    std::print($p, newln);
;MAIN
