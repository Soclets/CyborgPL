# A division that wraps like the hardware (owner, 2026-10-03): `div.trunc.wrap` and the other three give
# the true quotient, wrapped at the type's width, so the smallest value divided by -1 is itself, as on
# ARM. Plain `div.trunc` gives the true 128, checked where it lands. `mod` never overflows: no `.wrap`.
!MAIN:
    let $a: integer.signed.8_bit = negative 128;
    std::print.shortest(($a divide.truncate.wrap negative 1), " ", ($a divide.floor.wrap negative 1), " ", ($a divide.truncate negative 1), newline);
    let $m: integer.signed.%128_bit = negative 170141183460469231731687303715884105728;
    std::print.shortest(($m divide.truncate.wrap negative 1), " ", ($m divide.nearest.wrap 2), newline);
!end of MAIN
