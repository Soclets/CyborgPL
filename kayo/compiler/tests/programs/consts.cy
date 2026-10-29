# Consts, worked out once at compile time to the literal each is: integers,
# text, f64 and powers, used in a struct, a call, a loop's bound.


%extern.C proc exit [let $code: i32];
%extern.C proc strlen [let $s: ptr[u8]] -> u64;
pub const $max := 8;
const $pi: f64 := 3.141_592_653_589_793;
const $twice_max := $max x 2;
const $greeting := "hello " concat "world";
const $big: i64 := $max std::pow 20;
pub rec P:
    var x: i32;
;rec P
pub fn area [let $r: f64] -> f64:
    ret $pi x $r x $r;
;fn area
MAIN:
    var $got: i32 = 0;
    if $twice_max == 16:
        change $got to $got + 1;
    ;if
    let $s := $greeting;
    no_check if strlen(lend $s) == 11:
        change $got to $got + 2;
    ;if
    let $p := P[x = $max];
    if $p.x == 8:
        change $got to $got + 4;
    ;if
    let $a := area(2.0);
    if $a > 12.56 and $a < 12.57:
        change $got to $got + 8;
    ;if
    var $n: i32 = 0;
    for $i [from: 1, to: $max, by: 4]:
        change $n to $n + 1;
    ;for
    if $n == 2:
        change $got to $got + 16;
    ;if
    if $big == 1152921504606846976:
        change $got to $got + 32;
    ;if
    no_check my.exit($got);
;MAIN
