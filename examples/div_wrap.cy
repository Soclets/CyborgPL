# A division that wraps like the hardware (owner, 2026-10-03): `div.trunc.wrap` and the other three give
# the true quotient, wrapped at the type's width, so the smallest value divided by -1 is itself, as on
# ARM. Plain `div.trunc` gives the true 128, checked where it lands. `mod` never overflows: no `.wrap`.
!MAIN:
    let $a: int.sign.8 = neg 128;
    std::print.shortest(($a div.trunc.wrap neg 1), " ", ($a div.floor.wrap neg 1), " ", ($a div.trunc neg 1), newln);
    let $m: int.sign.%128 = neg 170141183460469231731687303715884105728;
    std::print.shortest(($m div.trunc.wrap neg 1), " ", ($m div.near.wrap 2), newln);
!end of MAIN
