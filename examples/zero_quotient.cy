# A whole quotient that rounds to zero is 0, never -0 (owner, 2026-10-07: "`b`"): 0.5 / -1.75 is
# -0.2857…, rounded up to 0. `.ieee` keeps IEEE's own -0, written where it is wanted ("`a`").
!MAIN:
    let $q: float.binary.64_bit = 0.5 divide.ceiling (negative 1.75);
    let $r: float.binary.64_bit = 0.5 divide.ceiling.ieee (negative 1.75);
    let $b: float.%brain.16_bit = 0.5 divide.truncate (negative 1.75);
    std::print.shortest($q, " ", $r, " ", $b, " ", $q equal_to $r, newline);
!end of MAIN
