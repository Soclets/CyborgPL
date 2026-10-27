# A written -0.0 is 0: no bin is -0 except through the .ieee forms (owner, 2026-10-07: "a").
# check claude, 2026-10-19 (corpus 3878ea56): the oracle kept the sign of a written -0.0.
%layout.C union Un:
    i: integer.signed.32_bit;
    u: integer.unsigned.32_bit;
    f: float.binary.32_bit;
end of union Un

!MAIN:
    let $a: float.binary.32_bit = negative 0.0;
    let $b := negative 0.0: float.binary.64_bit;
    variable $un := Un[i = 1];
    change $un.f = negative 0.0;
    !no_check let $u := $un.u;
    std::print.stored($a, " ", $b, " ", $u, newline);
!end of MAIN
