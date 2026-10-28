# deci, IEEE 754 decimal (owner, 2026-09-29): 0.1 plus 0.2 is exactly 0.3; every operation rounds to the
# width half to even; trailing zeros are kept (print shows 2.50, print.useful 2.5); a literal with too
# many digits is an error; text, and an integer whose type could have too many digits, fail
# too_many_digits; to.useful.d64[bin] is the shortest decimal, to.stored.d64[bin] the exact value (the form is always written).
MAIN:
    let $a: %d64 := 0.1;
    let $b: %d64 := $a + 0.2;
    std::println.stored($b, " ", $b == 0.3);
    let $price: %d64 := 19.99;
    let $sub: %d64 := $price x 3;
    let $tax: %d64 := $sub x 0.075;
    std::println.stored($sub, " ", $tax, " ", $sub + $tax);
    let $h: %d64 := 2.50;
    std::println.stored($h x 2, " ", $h, " ", 1.50 == 1.5);
    std::println($h x 2, " ", $h);
    let $third: %d64 := 1;
    std::println.stored($third div.near 3, " ", $h div.near 2, " ", 10.0 div.near 4);
    let $r: %d32 := 2.5;
    let $e1: %d32 := 1.000005;
    std::println.stored($e1 x 1, " ", $e1 + 0.0000005);
    let $w: f64 := 0.1;
    std::println.stored(to.stored.%d64($w), " ", to.stored.%d128($w), " ", to.shortest.%d128($w));
    let $n: i32 := 12345;
    std::println.stored(to.%d64($n));
    let $big: i64 := 12345678901234567;
    let $bd := to.%d64($big) ?? $price;
    std::println.stored($bd);
    let $t := to.%d64("1.250") ?? $price;
    let $u := to.%d64("0.12345678901234567") ?? $price;
    let $v := to.%d64("abc") ?? $h;
    std::println.stored($t, " ", $u, " ", $v);
    let $i := to.i32($h x 2) ?? -1;
    let $j := to.i32($h) ?? -1;
    std::println.stored($i, " ", $j, " ", to.f64($h), " ", to.stored.str.utf8($h));
    let $neg: %d64 := -1.5;
    std::println.stored($neg, " ", -$neg, " ", $neg < $h);
;MAIN
