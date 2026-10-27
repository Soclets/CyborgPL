# A deci has no -0 (owner, 2026-10-07): a zero result is 0 whatever made it, negation, times a negative, a
# difference, a quotient (check claude, seed 206 and after).
!MAIN:
    let $a: float.%decimal.64_bit = negative 5.0;
    let $m: float.%decimal.64_bit = $a modulo.ceiling 2.5;
    let $t: float.%decimal.64_bit = $m times 1.7;
    let $z: float.%decimal.64_bit = 0.0;
    let $n: float.%decimal.64_bit = negative 1.7;
    let $k: float.%decimal.64_bit = negative 3.0;
    std::print.stored($m, " ", $t, " ", negate $t, " ", negate $z, " ", $z times $n, " ", $z minus $z, " ", $z divide.nearest $k, newline);
!end of MAIN
