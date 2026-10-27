# A struct field prints in its own type (check claude, 2026-10-25): an f32 field shows f32's shortest form, as the same
# value in an f32 variable does; and an exact tie between two shortest forms goes to the even digit, as Ryu does.
record Pr:
    a: float.binary.32_bit; b: float.binary.32_bit;
end of record Pr
!MAIN:
    let $v: float.binary.32_bit = negative 3080.765625;
    let $p := Pr[a = $v, b = 1.5];
    std::print.shortest($v, " ", $p.a, newline);
    let $t: float.binary.32_bit = 5305.65625;
    std::print.shortest($t, newline);
!end of MAIN
