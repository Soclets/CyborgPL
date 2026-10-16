# An empty value worked out at run time, handed to a length-bounded parameter (cord claude, 2026-10-14):
# with an argument it stops at the call, as the oracle does; without one it prints 1.
fn first [lent 'xs': list[int64][min 1]] -> uint64 { return list.length[lend 'xs']; }

MAIN ['argc': int32, 'argv': ptr[ptr[uint8]]] {
    let mut 'w': list[int64] = [1,];
    if 'argc' > 1 { change 'w' = list.empty; }
    std::print.useful["before", newln];
    std::print.useful[first[lend 'w'], newln];
}
