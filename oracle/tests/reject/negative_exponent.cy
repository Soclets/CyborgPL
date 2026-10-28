# a negative exponent is a check site, for pow and pow.wrap alike (owner, 2026-09-24: a)
MAIN:
    let $n: i32 := -1;
    let $h: u32 := 2;
    let $w := $h std::pow.wrap $n;
;MAIN
