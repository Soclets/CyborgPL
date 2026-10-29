# A position kept from a walk that may have kept nothing is 0, no position.
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
%extern.C proc atoi [let $s: ptr[u8]] -> i32;
rec Net:
    var stations: std::ls[i32];
    var others: std::ls[i32];
;rec Net

fn pick [lent $net: Net] -> i32:
    var $at = 0;
    for $i, $s in lend $net.stations:
        if $s >= 3:
            change $at to $i;
        ;if
    ;for
    ret $net.stations[pos: $at];
;fn pick

fn own [let $n: i32] -> i32:
    ret 0;
;fn own

MAIN:
    let $k := "N";
    no_check let $n: i32 := atoi(getenv(lend $k));
    var $net = Net[stations = std::ls.empty, others = std::ls.empty];
    for $j [from: 1, to: $n, by: 1]:
        std::ls.append(lend.mut $net.stations, value: $j);
    ;for
    std::print(pick(lend $net), own($n), newln);
;MAIN
