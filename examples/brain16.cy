# The brain float (owner, 2026-10-02): `brain16`. f32's range in 16 bits,
# with 7 bits of precision; its own family; each operation rounds to the nearest brain16.
!MAIN:
    let $a: float.%brain.16_bit = 0.1;                     # a literal rounds to the nearest: 0.10009765625
    let $b: float.%brain.16_bit = 0.2;
    let $c: float.%brain.16_bit = $a plus $b;
    std::print.stored($a, " ", $c, newline);       # what is stored
    std::print.shortest($a, " ", $c, newline);  # the shortest text that rounds back to it
    let $x: float.binary.64_bit = 300.7;
    let $w := to.float.%brain.16_bit($x);                 # into it rounds: 300 (7 bits)
    let $back := to.float.binary.32_bit($w);                # out of it into a f32 is exact
    let $n := to.integer.signed.32_bit($w) otherwise negative 1;      # whole, so it converts
    std::print.shortest($w, " ", $back, " ", $n, newline);
    let $xs: std::list[float.%brain.16_bit] = [0.5, 1.5, 2.5];
    std::print.format("{} {places: 2} {form: stored}{newline}", $xs, $a, $a);
!end of MAIN
