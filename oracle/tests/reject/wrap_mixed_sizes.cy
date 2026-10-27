# wrapping is at one size, always written: a mixed sum says which (owner, 2026-09-23: b)
!MAIN:
    let $a: int.sign.8 = 100;
    let $b: int.sign.16 = 1000;
    let $w := ($a plus $b) plus.wrap 1;
!end of MAIN
