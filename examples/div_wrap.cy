# A division that wraps like the hardware (owner, 2026-10-03): `div.trunc.wrap` and the other three give
# the true quotient, wrapped at the type's width, so the smallest value divided by -1 is itself, as on
# ARM. Plain `div.trunc` gives the true 128, checked where it lands. `mod` never overflows: no `.wrap`.
MAIN {
    let $a: i8 = -128;
    std::print.useful(($a div.trunc.wrap -1), " ", ($a div.floor.wrap -1), " ", ($a div.trunc -1), newln);
    let $m: i128 = -170141183460469231731687303715884105728;
    std::print.useful(($m div.trunc.wrap -1), " ", ($m div.near.wrap 2), newln);
}
