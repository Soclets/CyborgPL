# A position kept from a walk, starting at 1, is inside a list that is not empty.
%extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
%extern.C fn atoi [$s: ptr[u8]] -> i32;
struct Net {
    stations: std::list[i32]
}

fn pick [lent $net: Net] -> i32 {
    let $at := 1;
    for $i, $s in lend $net.stations {
        if $s at_least 3 { change $at = $i; }
    }
    if std::list.length(lend $net.stations) equal_to 0 { return 0; }
    return $net.stations[pos: $at];
}

MAIN {
    let $k := "N";
    no_check let $n: i32 = atoi(getenv(lend $k));
    let $net := Net[stations = std::list.empty];
    for $j [from: 1, to: $n, by: 1] { std::list.append(lend.mut $net.stations, value: $j); }
    std::print.useful(pick(lend $net), newln);
}
