# Precog may turn a pure function over a small proven input range into a table at compile time; how big a table
# it may make is the programmer's (owner, 2026-10-19): `table_bytes` per target in `CyJet.cyr`, and in front of a
# `fn` for that one function. `table_bytes 0` turns tables off for it. The answers never change.
table_bytes 1_024 fn square [let $x: int.unsign.8] -> int.unsign.16:
    ret to.int.unsign.16($x) times to.int.unsign.16($x);
end of fn square

table_bytes 0 fn fib [let $n: int.sign.64[min: 0, max: 92]] -> int.sign.64:
    if $n less_than 2:
        ret $n;
    end of if
    ret fib($n minus 1) plus fib($n minus 2);
end of fn fib

!MAIN:
    std::print.shortest(square(200), " ", fib(30), newln);
!end of MAIN
