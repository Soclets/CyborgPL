# deci (owner, 2026-09-29): deci is its own family
!MAIN:
    let $a: float.%decimal.64_bit = 1.5;
    let $w: float.binary.64_bit = 2.0;
    let $c := $a plus $w;
!end of MAIN
