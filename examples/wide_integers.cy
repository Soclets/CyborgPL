# Integers past 128 bits (owner, 2026-10-03): `int.unsign.%not_std.256` and up, to LLVM's 2^23 bits. The maths is
# exact; in between, the limit is the widest operand's width, and the answer is checked where it lands.
MAIN:
    let $max: %not_std.u256 := 115792089237316195423570985008687907853269984665640564039457584007913129639935;
    let $half: %not_std.u256 := $max div.floor 2;
    let $top: %not_std.u256 := 2 ^ 255;
    std::println($half);
    std::println(($top - 1 + $top));
    std::println(($top plus.wrap $top), " ", ($top times.wrap 3));
    let $p: %not_std.i256 := -(2 ^ 200);
    std::println(($p div.near 3), " ", ($p mod.floor 7));
    let $c: %i128 := 170141183460469231731687303715884105727;
    std::println((($c + $c) div.floor 2));       # two int128s add at 128 bits in between
    let $m: %u128 := 340282366920938463463374607431768211455;
    std::println(($m plus.wrap 1), " ", ($m times.wrap $m));
;MAIN
