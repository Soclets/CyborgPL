# Text in two forms (owner, 2026-09-30): `to.stored.str.utf8` is the exact value, as `print.stored` shows it;
# `to.useful.str.utf8` is the useful one, as `print.useful` shows it. `.useful` follows the verb.
union.tagged Size:
    small;
    big;
end of union.tagged Size

!MAIN:
    let $x: float.binary.64_bit = 0.1;
    let $d: float.%decimal.64_bit = 1.50;
    let $n: integer.signed.32_bit = 42;
    let $ok := true;
    let $s := Size.big;
    std::print.stored(to.stored.string.utf8($x), " | ", to.shortest.string.utf8($x), newline);
    std::print.stored(to.stored.string.utf8($d), " | ", to.shortest.string.utf8($d), newline);
    std::print.stored(to.stored.string.utf8($n), " | ", to.shortest.string.utf8($n), newline);
    std::print.stored(to.stored.string.utf8($ok), " | ", to.shortest.string.utf8($ok), newline);
    std::print.stored(to.stored.string.utf8($s), " | ", to.shortest.string.utf8($s), newline);
    std::print.shortest($s, newline);
!end of MAIN
