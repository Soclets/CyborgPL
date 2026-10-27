# Precog may turn a pure function over a small proven input range into a table at compile time; how big a table
# it may make is the programmer's (owner, 2026-10-19): `table_bytes` per target in `CyJet.cyr`, and in front of a
# `fn` for that one function. `table_bytes 0` turns tables off for it. The answers never change.
table_bytes 1_024 function square [$x: integer.unsigned.8_bit] -> integer.unsigned.16_bit:
    return to.integer.unsigned.16_bit($x) times to.integer.unsigned.16_bit($x);
end of function square

table_bytes 0 function fib [$n: integer.signed.64_bit[minimum: 0, maximum: 92]] -> integer.signed.64_bit:
    if $n less_than 2:
        return $n;
    end of if
    return fib($n minus 1) plus fib($n minus 2);
end of function fib

!MAIN:
    std::print.shortest(square(200), " ", fib(30), newline);
!end of MAIN
