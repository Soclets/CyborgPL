# No bin or brain16 result is ever -0 (owner, 2026-10-07: "`a`"): a zero times a negative, a
# negated zero, a product too small to hold, are all plain 0. Each operator's `.ieee` form keeps
# IEEE's -0 where it is wanted ("yea"), and `negate.ieee` (was `neg.ieee`) is IEEE's negation ("yea"): `-.ieee $x`
# "defeats the purpose of a symbol". Every operator that can make a -0 has its `.ieee` form, `mod`
# and `pow` included ("yea").
!MAIN:
    let $z: float.binary.64_bit = 0.0;
    let $n: float.binary.64_bit = negative 2.0;
    let $e30: float.binary.64_bit = 0.000000000000000000000000000001;
    let $e60: float.binary.64_bit = $e30 times $e30;
    let $e120: float.binary.64_bit = $e60 times $e60;
    let $tiny: float.binary.64_bit = $e120 times $e120;
    std::print.shortest($z times $n, " ", $z times.ieee $n, newline);
    std::print.shortest(negate $z, " ", negate.ieee $z, newline);
    std::print.shortest($tiny times (negate $tiny), " ", $tiny times.ieee (negate $tiny), newline);
    std::print.shortest($z divide.nearest $n, " ", $z divide.nearest.ieee $n, newline);
    let $m: float.binary.64_bit = negate.ieee $z;
    std::print.shortest($m plus $m, " ", $m plus.ieee $m, " ", $m minus $z, " ", $m minus.ieee $z, newline);
    let $b: float.%brain.16_bit = 0.0;
    let $c: float.%brain.16_bit = negative 3.0;
    std::print.shortest($b times $c, " ", $b times.ieee $c, " ", negate $b, " ", negate.ieee $b, newline);
    let $w: float.binary.32_bit = $e30 times $e30 times (negative 1.0);
    std::print.shortest($w, " ", 3.0 modulo.floor (negative 3.0), " ", 3.0 modulo.floor.ieee (negative 3.0), " ", (negative 3.0) modulo.truncate 3.0, " ", (negative 3.0) modulo.truncate.ieee 3.0, newline);
    # `mod` and `pow` keep it too (owner, 2026-10-07: "yea")
    std::print.shortest((negate $e120) power 3, " ", (negate $e120) pow.ieee 3, " ", (negate $e120) std::power.ieee 3, newline);
!end of MAIN
