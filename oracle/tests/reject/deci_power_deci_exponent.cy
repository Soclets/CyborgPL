# a deci power's exponent is an integer or a whole literal (owner, 2026-09-29: a)
!MAIN:
    let $r: float.%deci.64 := 1.05;
    let $k: float.%deci.64 := 3.00;
    let $q := $r pow $k;
!end of MAIN
