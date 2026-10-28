# a shorter name for other kits, `pub fn short = area;`, is decided (2026-09-23) and not built in the oracle yet


fn area [let $r: f64] -> f64:
    ret $r x $r;
;fn area
pub fn short = area;
MAIN:
;MAIN
