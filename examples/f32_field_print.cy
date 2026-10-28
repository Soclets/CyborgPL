# A struct field prints in its own type (check claude, 2026-10-25): an f32 field shows f32's shortest form, as the same
# value in an f32 variable does; and an exact tie between two shortest forms goes to the even digit, as Ryu does.
rec Pr:
    var a: f32;
    var b: f32;
;rec Pr
MAIN:
    let $v: f32 := -3080.765625;
    let $p := Pr[a = $v, b = 1.5];
    std::println($v, " ", $p.a);
    let $t: f32 := 5305.65625;
    std::println($t);
;MAIN
