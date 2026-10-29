# A position kept from a walk, in a list made one shorter since, may be past its end.
%extern.C proc getenv [let $name: ptr[u8]] -> ptr[u8];
%extern.C proc atoi [let $s: ptr[u8]] -> i32;
rec Net:
    var stations: std::ls[i32];
    var others: std::ls[i32];
;rec Net

fn pick [lent $net: Net] -> i32:
    ret 0;
;fn pick

fn own [let $n: i32] -> i32:
    var $xs: std::ls[i32] = std::ls.empty;
    for $j [from: 1, to: $n, by: 1]:
        std::ls.append(lend.mut $xs, value: $j);
    ;for
    var $at = 1;
    for $i, $s in lend $xs:
        if $s >= 3:
            change $at to $i;
        ;if
    ;for
    if std::ls.len(lend $xs) == 0:
        ret 0;
    ;if
    std::ls.remove(lend.mut $xs, pos: 1);
    if std::ls.len(lend $xs) == 0:
        ret 0;
    ;if
    ret $xs[pos: $at];
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
