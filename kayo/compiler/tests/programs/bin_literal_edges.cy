# A literal into a f16 or f32 is decided and written from its digits,
# rounded once (oracle 8705014): read at 64 bits first, the first two
# land on the limit, tie to infinity, and were refused.
%extern.C proc exit [$code: i32];
MAIN:
    var $got: i32 = 0;
    let $a: f32 := 340282356779733661637539395458142568447.0;
    if $a == 340282346638528859811704183484516925440.0:
        change $got to $got + 1;
    ;if
    let $b: f16 := 65519.999999999999999999;
    if $b == 65504.0:
        change $got to $got + 2;
    ;if
    let $c: f32 := 1.00000005960464477539062500001;
    if $c == 1.00000011920928955078125:
        change $got to $got + 4;
    ;if
    let $d: f16 := -65519.9;
    if $d == -65504.0:
        change $got to $got + 8;
    ;if
    no_check my.exit($got);
;MAIN
