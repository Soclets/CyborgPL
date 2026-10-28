# Constants, worked out while building, `pub` and not, used in MAIN.

pub const $max := 8;
const $pi: f64 := 3.141_592_653_589_793;
const $twice_max := $max x 2;
const $greeting := "hello " ++ "world";
pub rec P:
    var x: i32;
;rec P
pub fn area [let $r: f64] -> f64:
    ret $pi x $r x $r;
;fn area
MAIN:
    std::println($max, " ", $twice_max, " ", $greeting, " ", area(2.0));
    let $s := $greeting;
    let $p := P[x = $max];
    for $i [from: 1, to: $max, by: 4]:
        std::print($i, " ");
    ;for
    std::println.stored();
;MAIN
