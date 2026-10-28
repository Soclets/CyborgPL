# a d64 into a d32 says what happens when the digits do not fit (owner, 2026-09-29: a)
MAIN:
    let $x: %d64 := 1.5;
    let $y := to.%d32($x);
;MAIN
