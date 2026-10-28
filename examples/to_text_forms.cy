# Text in two forms (owner, 2026-09-30): `to.stored.str.utf8` is the exact value, as `print.stored` shows it;
# `to.useful.str.utf8` is the useful one, as `print.useful` shows it. `.useful` follows the verb.
union.tag Size:
    small;
    big;
;union.tag Size

MAIN:
    let $x: f64 := 0.1;
    let $d: %d64 := 1.50;
    let $n: i32 := 42;
    let $ok := true;
    let $s := Size.big;
    std::println.stored(to.stored.str.utf8($x), " | ", to.shortest.str.utf8($x));
    std::println.stored(to.stored.str.utf8($d), " | ", to.shortest.str.utf8($d));
    std::println.stored(to.stored.str.utf8($n), " | ", to.shortest.str.utf8($n));
    std::println.stored(to.stored.str.utf8($ok), " | ", to.shortest.str.utf8($ok));
    std::println.stored(to.stored.str.utf8($s), " | ", to.shortest.str.utf8($s));
    std::println($s);
;MAIN
