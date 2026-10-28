# a bin never holds an infinity (owner, 2026-09-29: a)
MAIN:
    let $big: f64 := 10.0 ^ 308.0;
    let $inf: f64 := $big x 10.0;
;MAIN
