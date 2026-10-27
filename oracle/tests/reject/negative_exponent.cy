# a negative exponent is a check site, for pow and pow.wrap alike (owner, 2026-09-24: a)
!MAIN:
    let $n: integer.signed.32_bit = negative 1;
    let $h: integer.unsigned.32_bit = 2;
    let $w := $h pow.wrap $n;
!end of MAIN
