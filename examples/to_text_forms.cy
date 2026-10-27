# Text in two forms (owner, 2026-09-30): `to.stored.str.utf8` is the exact value, as `print.stored` shows it;
# `to.useful.str.utf8` is the useful one, as `print.useful` shows it. `.useful` follows the verb.
union.tag Size:
    small;
    big;
end of union.tag Size

!MAIN:
    let $x: float.bin.64 = 0.1;
    let $d: float.%deci.64 = 1.50;
    let $n: int.sign.32 = 42;
    let $ok := true;
    let $s := Size.big;
    std::print.stored(to.stored.str.utf8($x), " | ", to.shortest.str.utf8($x), newln);
    std::print.stored(to.stored.str.utf8($d), " | ", to.shortest.str.utf8($d), newln);
    std::print.stored(to.stored.str.utf8($n), " | ", to.shortest.str.utf8($n), newln);
    std::print.stored(to.stored.str.utf8($ok), " | ", to.shortest.str.utf8($ok), newln);
    std::print.stored(to.stored.str.utf8($s), " | ", to.shortest.str.utf8($s), newln);
    std::print.shortest($s, newln);
!end of MAIN
