# Precog may turn a pure function over a small proven input range into a table at compile time; how big a table
# it may make is the programmer's (owner, 2026-10-19): `table_bytes` per target in `CyJet.cyr`, and in front of a
# `fn` for that one function. `table_bytes 0` turns tables off for it. The answers never change.
table_bytes 1_024 fn square [let $x: u8] -> u16:
    ret to.u16($x) x to.u16($x);
;fn square

table_bytes 0 fn fib [let $n: i64[min: 0, max: 92]] -> i64:
    if $n < 2:
        ret $n;
    ;if
    ret fib($n - 1) + fib($n - 2);
;fn fib

MAIN:
    std::print(square(200), " ", fib(30), newln);
;MAIN
