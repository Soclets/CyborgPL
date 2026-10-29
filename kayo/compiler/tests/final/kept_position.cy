# A position kept from a walk over a list, tested not 0, is inside it (design claude, 2026-10-26).
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
%extern.C proc atoi [let $s: ptr[u8]] -> i32;
rec Net:
    var stations: std::ls[i32];
;rec Net

fn pick [lent $net: Net] -> i32:
    var $at = 0;
    for $i, $s in lend $net.stations:
        if $s >= 3:
            change $at to $i;
        ;if
    ;for
    if $at != 0:
        ret $net.stations[pos: $at];
    ;if
    ret 0;
;fn pick

MAIN:
    let $k := "N";
    no_check let $n: i32 := atoi(getenv(lend $k));
    var $net = Net[stations = std::ls.empty];
    for $j [from: 1, to: $n, by: 1]:
        std::ls.append(lend.mut $net.stations, value: $j);
    ;for
    std::print(pick(lend $net), newln);
;MAIN
