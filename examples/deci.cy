# deci, IEEE 754 decimal (owner, 2026-09-29): 0.1 plus 0.2 is exactly 0.3; every operation rounds to the
# width half to even; trailing zeros are kept (print shows 2.50, print.useful 2.5); a literal with too
# many digits is an error; text, and an integer whose type could have too many digits, fail
# too_many_digits; to.useful.d64[bin] is the shortest decimal, to.stored.d64[bin] the exact value (the form is always written).
!MAIN:
    let $a: float.%deci.64 := 0.1;
    let $b: float.%deci.64 := $a plus 0.2;
    std::print.stored($b, " ", $b eq 0.3, newln);
    let $price: float.%deci.64 := 19.99;
    let $sub: float.%deci.64 := $price times 3;
    let $tax: float.%deci.64 := $sub times 0.075;
    std::print.stored($sub, " ", $tax, " ", $sub plus $tax, newln);
    let $h: float.%deci.64 := 2.50;
    std::print.stored($h times 2, " ", $h, " ", 1.50 eq 1.5, newln);
    std::print.shortest($h times 2, " ", $h, newln);
    let $third: float.%deci.64 := 1;
    std::print.stored($third div.near 3, " ", $h div.near 2, " ", 10.0 div.near 4, newln);
    let $r: float.%deci.32 := 2.5;
    let $e1: float.%deci.32 := 1.000005;
    std::print.stored($e1 times 1, " ", $e1 plus 0.0000005, newln);
    let $w: float.bin.64 := 0.1;
    std::print.stored(to.stored.float.%deci.64($w), " ", to.stored.float.%deci.128($w), " ", to.shortest.float.%deci.128($w), newln);
    let $n: int.sign.32 := 12345;
    std::print.stored(to.float.%deci.64($n), newln);
    let $big: int.sign.64 := 12345678901234567;
    let $bd := to.float.%deci.64($big) otherwise $price;
    std::print.stored($bd, newln);
    let $t := to.float.%deci.64("1.250") otherwise $price;
    let $u := to.float.%deci.64("0.12345678901234567") otherwise $price;
    let $v := to.float.%deci.64("abc") otherwise $h;
    std::print.stored($t, " ", $u, " ", $v, newln);
    let $i := to.int.sign.32($h times 2) otherwise neg 1;
    let $j := to.int.sign.32($h) otherwise neg 1;
    std::print.stored($i, " ", $j, " ", to.float.bin.64($h), " ", to.stored.str.utf8($h), newln);
    let $neg: float.%deci.64 := neg 1.5;
    std::print.stored($neg, " ", neg $neg, " ", $neg lt $h, newln);
!end of MAIN
