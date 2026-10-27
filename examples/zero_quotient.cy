# A whole quotient that rounds to zero is 0, never -0 (owner, 2026-10-07: "`b`"): 0.5 / -1.75 is
# -0.2857…, rounded up to 0. `.ieee` keeps IEEE's own -0, written where it is wanted ("`a`").
!MAIN:
    let $q: float.bin.64 = 0.5 div.ceil (neg 1.75);
    let $r: float.bin.64 = 0.5 div.ceil.ieee (neg 1.75);
    let $b: float.%brain.16 = 0.5 div.trunc (neg 1.75);
    std::print.shortest($q, " ", $r, " ", $b, " ", $q eq $r, newln);
!end of MAIN
