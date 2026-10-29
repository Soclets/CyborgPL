# A position kept from a walk, starting at 1, is inside a list that is not empty.
%extern.C proc getenv [$name: ptr[u8]] -> ptr[u8];
%extern.C proc atoi [$s: ptr[u8]] -> i32;
rec Net:
    var stations: std::ls[i32];
;rec Net

fn pick [lent $net: Net] -> i32:
    var $at = 1;
    for $i, $s in lend $net.stations:
        if $s >= 3:
            change $at to $i;
        ;if
    ;for
    if std::ls.len(lend $net.stations) == 0:
        ret 0;
    ;if
    ret $net.stations[pos: $at];
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
