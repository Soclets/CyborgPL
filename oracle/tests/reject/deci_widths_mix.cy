# deci (owner, 2026-09-29): two deci widths say which
!MAIN:
    let $a: float.%deci.64 = 1.5;
    let $b: float.%deci.32 = 1.5;
    let $c := $a plus $b;
!end of MAIN
