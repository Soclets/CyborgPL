# Wrapping arithmetic (decided 2026-09-20): plus.wrap minus.wrap times.wrap pow.wrap are always defined — modulo the
# type's size — so a hash needs no no_check. Plain plus minus times pow stay overflow-checked.
fn hash [lent $text: str.utf8] -> u32 {
    let $h: u32 = 2_166_136_261;                 # FNV-1a
    for $b in std::str.each.byte(lend $text) {
        change $h = $h xor $b;                          # FNV-1a: xor, then a wrapping multiply
        change $h = $h times.wrap 16_777_619;                    # wraps instead of overflowing
    }
    return $h;
}

MAIN {
    std::print.useful(hash("hello"), newln);
    let $c: u8 = 250;
    change $c = $c plus.wrap 10;                                # 260 wraps to 4
    std::print.useful($c, newln);
    let $i: i8 = 127;
    change $i = $i plus.wrap 1;                                 # 128 wraps to -128
    std::print.useful($i, newln);
    let $p: u16 = 3;
    change $p = $p pow.wrap 20;                               # 3^20 modulo 65536
    std::print.useful($p, newln);
}
