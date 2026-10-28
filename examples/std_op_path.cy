# A kit's operators may carry its path or not (owner, 2026-10-22: "It's always optional"), so std's own `pow` may be
# written `std::pow`. The base's operators are no kit's, so they never take `std::` (owner, 2026-10-28: "`plus` isn't
# even `std`").

MAIN:
    let $a: i32 := 2 + 3 x 4;
    let $b: i32 := 7 div.floor 2;
    let $c: u32 := 1 shl 3;
    let $d: i32 := 2 std::pow 3;
    std::print($a, " ", $b, " ", $c, " ", $d, newln);
;MAIN
