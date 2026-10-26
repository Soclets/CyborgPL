# The whole struct replaced: the guard on its old field says nothing.
%extern.C fn getenv [$name: ptr[u8]] -> ptr[u8];
%extern.C fn atoi [$s: ptr[u8]] -> i32;
struct Clock {
    now: i32[min: 0, max: 2895],
    stop: i32[min: 0, max: 2880]
}

fn bump [lent.mut $c: Clock] {
    change $c.now = 2895;
}

fn Clock.next [lent.mut $c: Clock] -> optional[i32] {
    if $c.now at_least 2880 { return optional.none; }
    change $c = Clock[now = 2890, stop = 0];
    change $c.now = $c.now + 15;
    return optional.none;
}

MAIN {
    let $k := "N";
    no_check let $n: i32 = atoi(getenv(lend $k));
    if $n at_least 0 and $n at_most 2880 {
        let mut $quarters := Clock[now = $n, stop = $n];
        for $t in lend.mut $quarters {
            std::print.useful(" ", $t);
        }
    }
}
