# An empty value worked out at run time, handed to a length-bounded parameter (cord claude, 2026-10-14):
# with an argument it stops at the call, as the oracle does; without one it prints 1.
fn first [lent 'part': str.utf8[min 1 bytes]] -> uint64 { return str.bytes[lend 'part']; }

MAIN ['argc': int32, 'argv': ptr[ptr[uint8]]] {
    let mut 'w': str.utf8 = "x";
    if 'argc' > 1 { change 'w' = str.empty; }
    std::print.useful["before", newln];
    std::print.useful[first[lend 'w'], newln];
}
