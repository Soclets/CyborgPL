# deci (owner, 2026-09-29): two deci widths say which
!MAIN:
    let $a: float.%decimal.64_bit = 1.5;
    let $b: float.%decimal.32_bit = 1.5;
    let $c := $a plus $b;
!end of MAIN
