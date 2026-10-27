# Constants, worked out while building, `pub` and not, used in MAIN.

pub const $max := 8;
const $pi: float.bin.64 = 3.141_592_653_589_793;
const $twice_max := $max times 2;
const $greeting := "hello " concat "world";
pub rec P:
    var x: int.sign.32;
end of rec P
pub fn area [let $r: float.bin.64] -> float.bin.64:
    ret $pi times $r times $r;
end of fn area
!MAIN:
    std::print.shortest($max, " ", $twice_max, " ", $greeting, " ", area(2.0), newln);
    let $s := $greeting;
    let $p := P[x = $max];
    for $i [from: 1, to: $max, by: 4]:
        std::print.shortest($i, " ");
    end of for
    std::print.stored(newln);
!end of MAIN
