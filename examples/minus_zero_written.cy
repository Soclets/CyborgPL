# A written -0.0 is 0: no bin is -0 except through the .ieee forms (owner, 2026-10-07: "a").
# check claude, 2026-10-19 (corpus 3878ea56): the oracle kept the sign of a written -0.0.
%layout.C union Un:
    i: i32;
    u: u32;
    f: f32;
;%layout.C union Un

MAIN:
    let $a: f32 := -0.0;
    let $b := -0.0: f64;
    var $un = Un[i = 1];
    change $un.f to -0.0;
    no_check let $u := $un.u;
    std::print.stored($a, " ", $b, " ", $u, newln);
;MAIN
