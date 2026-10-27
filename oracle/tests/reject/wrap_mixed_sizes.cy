# wrapping is at one size, always written: a mixed sum says which (owner, 2026-09-23: b)
!MAIN:
    let $a: integer.signed.8_bit = 100;
    let $b: integer.signed.16_bit = 1000;
    let $w := ($a plus $b) plus.wrap 1;
!end of MAIN
