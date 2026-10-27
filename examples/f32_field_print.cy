# A struct field prints in its own type (check claude, 2026-10-25): an f32 field shows f32's shortest form, as the same
# value in an f32 variable does; and an exact tie between two shortest forms goes to the even digit, as Ryu does.
rec Pr:
    var a: float.bin.32;
    var b: float.bin.32;
end of rec Pr
!MAIN:
    let $v: float.bin.32 = neg 3080.765625;
    let $p := Pr[a = $v, b = 1.5];
    std::print.shortest($v, " ", $p.a, newln);
    let $t: float.bin.32 = 5305.65625;
    std::print.shortest($t, newln);
!end of MAIN
