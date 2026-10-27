# deci (owner, 2026-09-29): deci is its own family
!MAIN:
    let $a: float.%deci.64 = 1.5;
    let $w: float.bin.64 = 2.0;
    let $c := $a plus $w;
!end of MAIN
