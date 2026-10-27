# deci, IEEE 754 decimal (owner, 2026-09-29): 0.1 plus 0.2 is exactly 0.3; every operation rounds to the
# width half to even; trailing zeros are kept (print shows 2.50, print.useful 2.5); a literal with too
# many digits is an error; text, and an integer whose type could have too many digits, fail
# too_many_digits; to.useful.d64[bin] is the shortest decimal, to.stored.d64[bin] the exact value (the form is always written).
!MAIN:
    let $a: float.%decimal.64_bit = 0.1;
    let $b: float.%decimal.64_bit = $a plus 0.2;
    std::print.stored($b, " ", $b equal_to 0.3, newline);
    let $price: float.%decimal.64_bit = 19.99;
    let $sub: float.%decimal.64_bit = $price times 3;
    let $tax: float.%decimal.64_bit = $sub times 0.075;
    std::print.stored($sub, " ", $tax, " ", $sub plus $tax, newline);
    let $h: float.%decimal.64_bit = 2.50;
    std::print.stored($h times 2, " ", $h, " ", 1.50 equal_to 1.5, newline);
    std::print.shortest($h times 2, " ", $h, newline);
    let $third: float.%decimal.64_bit = 1;
    std::print.stored($third divide.nearest 3, " ", $h divide.nearest 2, " ", 10.0 divide.nearest 4, newline);
    let $r: float.%decimal.32_bit = 2.5;
    let $e1: float.%decimal.32_bit = 1.000005;
    std::print.stored($e1 times 1, " ", $e1 plus 0.0000005, newline);
    let $w: float.binary.64_bit = 0.1;
    std::print.stored(to.stored.float.decimal.64_bit($w), " ", to.stored.float.decimal.128_bit($w), " ", to.shortest.float.decimal.128_bit($w), newline);
    let $n: integer.signed.32_bit = 12345;
    std::print.stored(to.float.%decimal.64_bit($n), newline);
    let $big: integer.signed.64_bit = 12345678901234567;
    let $bd := to.float.%decimal.64_bit($big) otherwise $price;
    std::print.stored($bd, newline);
    let $t := to.float.%decimal.64_bit("1.250") otherwise $price;
    let $u := to.float.%decimal.64_bit("0.12345678901234567") otherwise $price;
    let $v := to.float.%decimal.64_bit("abc") otherwise $h;
    std::print.stored($t, " ", $u, " ", $v, newline);
    let $i := to.integer.signed.32_bit($h times 2) otherwise negative 1;
    let $j := to.integer.signed.32_bit($h) otherwise negative 1;
    std::print.stored($i, " ", $j, " ", to.float.binary.64_bit($h), " ", to.stored.string.utf8($h), newline);
    let $neg: float.%decimal.64_bit = negative 1.5;
    std::print.stored($neg, " ", negate $neg, " ", $neg less_than $h, newline);
!end of MAIN
