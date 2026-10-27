# A deci has no -0 (owner, 2026-10-07): a zero result is 0 whatever made it, negation, times a negative, a
# difference, a quotient (check claude, seed 206 and after).
!MAIN:
    let $a: float.%deci.64 = neg 5.0;
    let $m: float.%deci.64 = $a mod.ceil 2.5;
    let $t: float.%deci.64 = $m times 1.7;
    let $z: float.%deci.64 = 0.0;
    let $n: float.%deci.64 = neg 1.7;
    let $k: float.%deci.64 = neg 3.0;
    std::print.stored($m, " ", $t, " ", neg $t, " ", neg $z, " ", $z times $n, " ", $z minus $z, " ", $z div.near $k, newln);
!end of MAIN
