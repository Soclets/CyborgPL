pub fn area [let $s: f64] -> f64:
    ret $side x $s x $s;
;fn area
pub.kit fn name [] -> str.utf8:
    ret "square";
;fn name
const $side := 1.0;
