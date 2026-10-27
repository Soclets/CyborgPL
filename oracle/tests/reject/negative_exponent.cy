# a negative exponent is a check site, for pow and pow.wrap alike (owner, 2026-09-24: a)
!MAIN:
    let $n: int.sign.32 = neg 1;
    let $h: int.unsign.32 = 2;
    let $w := $h pow.wrap $n;
!end of MAIN
