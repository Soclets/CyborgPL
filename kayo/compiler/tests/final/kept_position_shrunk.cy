# A position kept from a walk, in a list made one shorter since, may be past its end.
%extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
%extern.C fn atoi [$s: ptr[u8]] -> i32;
struct Net {
    stations: std::list[i32],
    others: std::list[i32]
}

fn pick [lent $net: Net] -> i32 {
    return 0;
}

fn own [$n: i32] -> i32 {
    let $xs: std::list[i32] = std::list.empty;
    for $j [from: 1, to: $n, by: 1] { std::list.append(lend.mut $xs, value: $j); }
    let $at := 1;
    for $i, $s in lend $xs {
        if $s at_least 3 { change $at = $i; }
    }
    if std::list.length(lend $xs) equal_to 0 { return 0; }
    std::list.remove(lend.mut $xs, pos: 1);
    if std::list.length(lend $xs) equal_to 0 { return 0; }
    return $xs[pos: $at];
}

MAIN {
    let $k := "N";
    no_check let $n: i32 = atoi(getenv(lend $k));
    let $net := Net[stations = std::list.empty, others = std::list.empty];
    for $j [from: 1, to: $n, by: 1] { std::list.append(lend.mut $net.stations, value: $j); }
    std::print.useful(pick(lend $net), own($n), newln);
}
