# Integers past 128 bits (owner, 2026-10-03): `not_std.u256` and up, to LLVM's 2^23 bits. The maths is
# exact; in between, the limit is the widest operand's width, and the answer is checked where it lands.
!MAIN:
    let $max: %not_std.u256 = 115792089237316195423570985008687907853269984665640564039457584007913129639935;
    let $half: %not_std.u256 = $max divide.floor 2;
    let $top: %not_std.u256 = 2 power 255;
    std::print.shortest($half, newline);
    std::print.shortest(($top minus 1 plus $top), newline);
    std::print.shortest(($top plus.wrap $top), " ", ($top times.wrap 3), newline);
    let $p: %not_std.i256 = negate (2 power 200);
    std::print.shortest(($p divide.nearest 3), " ", ($p modulo.floor 7), newline);
    let $c: integer.signed.%128_bit = 170141183460469231731687303715884105727;
    std::print.shortest((($c plus $c) divide.floor 2), newline);       # two int128s add at 128 bits in between
    let $m: integer.unsigned.%128_bit = 340282366920938463463374607431768211455;
    std::print.shortest(($m plus.wrap 1), " ", ($m times.wrap $m), newline);
!end of MAIN
