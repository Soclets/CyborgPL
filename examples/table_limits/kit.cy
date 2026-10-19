# Precog may turn a pure function over a small proven input range into a table at compile time; how big a table
# it may make is the programmer's (owner, 2026-10-19): `table_bytes` per target in `CyJet.cyr`, and in front of a
# `fn` for that one function. `table_bytes 0` turns tables off for it. The answers never change.
table_bytes 1_024 fn square ['x': uint8] -> uint16 {
    return to.uint16['x'] x to.uint16['x'];
}

table_bytes 0 fn fib ['n': int64[min 0, max 92]] -> int64 {
    if 'n' < 2 { return 'n'; }
    return fib['n' - 1] + fib['n' - 2];
}

MAIN {
    std::print.useful[square[200], " ", fib[30], newln];
}
