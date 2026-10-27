# No bin or brain16 result is ever -0 (owner, 2026-10-07: "`a`"): a zero times a negative, a
# negated zero, a product too small to hold, are all plain 0. Each operator's `.ieee` form keeps
# IEEE's -0 where it is wanted ("yea"), and `negate.ieee` (was `neg.ieee`) is IEEE's negation ("yea"): `-.ieee $x`
# "defeats the purpose of a symbol". Every operator that can make a -0 has its `.ieee` form, `mod`
# and `pow` included ("yea").
!MAIN:
    let $z: float.bin.64 = 0.0;
    let $n: float.bin.64 = neg 2.0;
    let $e30: float.bin.64 = 0.000000000000000000000000000001;
    let $e60: float.bin.64 = $e30 times $e30;
    let $e120: float.bin.64 = $e60 times $e60;
    let $tiny: float.bin.64 = $e120 times $e120;
    std::print.shortest($z times $n, " ", $z times.ieee $n, newln);
    std::print.shortest(neg $z, " ", neg.ieee $z, newln);
    std::print.shortest($tiny times (neg $tiny), " ", $tiny times.ieee (neg $tiny), newln);
    std::print.shortest($z div.near $n, " ", $z div.near.ieee $n, newln);
    let $m: float.bin.64 = neg.ieee $z;
    std::print.shortest($m plus $m, " ", $m plus.ieee $m, " ", $m minus $z, " ", $m minus.ieee $z, newln);
    let $b: float.%brain.16 = 0.0;
    let $c: float.%brain.16 = neg 3.0;
    std::print.shortest($b times $c, " ", $b times.ieee $c, " ", neg $b, " ", neg.ieee $b, newln);
    let $w: float.bin.32 = $e30 times $e30 times (neg 1.0);
    std::print.shortest($w, " ", 3.0 mod.floor (neg 3.0), " ", 3.0 mod.floor.ieee (neg 3.0), " ", (neg 3.0) mod.trunc 3.0, " ", (neg 3.0) mod.trunc.ieee 3.0, newln);
    # `mod` and `pow` keep it too (owner, 2026-10-07: "yea")
    std::print.shortest((neg $e120) pow 3, " ", (neg $e120) pow.ieee 3, " ", (neg $e120) std::pow.ieee 3, newln);
!end of MAIN
