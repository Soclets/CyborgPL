# deci, IEEE 754 decimal (owner, 2026-09-29), as examples/deci.cpls has it
# but checked by the program itself: each value's text against the
# oracle's, one point each, so the exit is how many agree (30).
%extern.C proc exit [let $code: i32];
fn same [let $v: %d64, let $want: str.utf8] -> i32:
    if to.stored.str.utf8($v) == $want:
        ret 1;
    ;if
    ret 0;
;fn same
MAIN:
    var $got: i32 = 0;
    let $a: %d64 := 0.1;
    let $b: %d64 := $a + 0.2;
    change $got to $got + same($b, "0.3");
    if $b == 0.3:
        change $got to $got + 1;
    ;if
    let $price: %d64 := 19.99;
    let $sub: %d64 := $price x 3;
    let $tax: %d64 := $sub x 0.075;
    change $got to $got + same($sub, "59.97") + same($tax, "4.49775") + same($sub + $tax, "64.46775");
    let $h: %d64 := 2.50;
    change $got to $got + same($h x 2, "5.00") + same($h, "2.50");
    if $h x 2 == 5:
        change $got to $got + 1;
    ;if
    let $third: %d64 := 1;
    change $got to $got + same($third div.near 3, "0.3333333333333333") + same($h div.near 2, "1.25");
    let $e1: %d32 := 1.000005;
    if to.stored.str.utf8($e1 + 0.0000005) == "1.000006":
        change $got to $got + 1;
    ;if
    let $w: f64 := 0.1;
    change $got to $got + same(to.stored.%d64($w), "0.1000000000000000");
    if to.stored.str.utf8(to.stored.%d128($w)) == "0.1000000000000000055511151231257827":
        change $got to $got + 1;
    ;if
    if to.stored.str.utf8(to.shortest.%d128($w)) == "0.1":
        change $got to $got + 1;
    ;if
    let $n: i32 := 12345;
    change $got to $got + same(to.%d64($n), "12345");
    let $big: i64 := 12345678901234567;
    let $bd := to.%d64($big) ?? $price;
    let $fits: i64 := 1234567890123456;
    let $fd := to.%d64($fits) ?? $price;
    change $got to $got + same($bd, "19.99") + same($fd, "1234567890123456");
    let $t := to.%d64("1.250") ?? $price;
    let $u := to.%d64("0.12345678901234567")-
    otherwise:
        case too_many_digits:
            is $a;
        ;case too_many_digits
        case not_a_number:
            is $h;
        ;case not_a_number
        case out_of_range:
            is $h;
        ;case out_of_range
    ;otherwise;
    let $v := to.%d64("abc")-
    otherwise:
        case not_a_number:
            is $h;
        ;case not_a_number
        case too_many_digits:
            is $a;
        ;case too_many_digits
        case out_of_range:
            is $a;
        ;case out_of_range
    ;otherwise;
    let $x := to.%d64("1e999")-
    otherwise:
        case out_of_range:
            is $third;
        ;case out_of_range
        case too_many_digits:
            is $a;
        ;case too_many_digits
        case not_a_number:
            is $a;
        ;case not_a_number
    ;otherwise;
    change $got to $got + same($t, "1.250") + same($u, "0.1") + same($v, "2.50") + same($x, "1");
    let $i := to.i32($h x 2) ?? -1;
    let $j := to.i32($h) ?? -1;
    if $i == 5 and $j == -1:
        change $got to $got + 1;
    ;if
    if to.f64($h) == 2.5:
        change $got to $got + 1;
    ;if
    let $neg: %d64 := -1.5;
    change $got to $got + same(-$neg, "1.5") + same($neg, "-1.5");
    if $neg < $h and not ($h < $neg) and $h >= 2.5:
        change $got to $got + 1;
    ;if
    let $g: %d64 := 0.1 + 0.2;
    change $got to $got + same($g, "0.3");
    let $r: %d32 := 2.5;
    let $wide := to.%d64($r);
    change $got to $got + same($wide x $h, "6.250");
    let $z: %d64 := 0;
    change $got to $got + same(-$z, "0") + same($h - $h, "0.00");
    no_check my.exit($got);
;MAIN
