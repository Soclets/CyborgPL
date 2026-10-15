# A whole quotient that rounds to zero is 0, never -0 (owner, 2026-10-07: "`b`"): 0.5 / -1.75 is
# -0.2857…, rounded up to 0. `.ieee` keeps IEEE's own -0, written where it is wanted ("`a`").
MAIN {
    let 'q': bin64 = 0.5 div.ceil (-1.75);
    let 'r': bin64 = 0.5 div.ceil.ieee (-1.75);
    let 'b': brain16 = 0.5 div.trunc (-1.75);
    std::print.useful['q', " ", 'r', " ", 'b', " ", 'q' equal_to 'r', newln];
}
