# Wrapping arithmetic (decided 2026-09-20): plus.wrap minus.wrap times.wrap pow.wrap are always defined — modulo the
# type's size — so a hash needs no no_check. Plain plus minus times pow stay overflow-checked.
fn hash [lent $text: str.utf8] -> int.unsign.32:
    var $h: int.unsign.32 = 2_166_136_261;                 # FNV-1a
    for $b in std::str.each.byte(lend $text):
        change $h to $h xor $b;                          # FNV-1a: xor, then a wrapping multiply
        change $h to $h times.wrap 16_777_619;                    # wraps instead of overflowing
    end of for
    ret $h;
end of fn hash

!MAIN:
    std::print.shortest(hash("hello"), newln);
    var $c: int.unsign.8 = 250;
    change $c to $c plus.wrap 10;                                # 260 wraps to 4
    std::print.shortest($c, newln);
    var $i: int.sign.8 = 127;
    change $i to $i plus.wrap 1;                                 # 128 wraps to -128
    std::print.shortest($i, newln);
    var $p: int.unsign.16 = 3;
    change $p to $p pow.wrap 20;                               # 3^20 modulo 65536
    std::print.shortest($p, newln);
!end of MAIN
