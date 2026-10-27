# The brain float (owner, 2026-10-02): `brain16`. f32's range in 16 bits,
# with 7 bits of precision; its own family; each operation rounds to the nearest brain16.
!MAIN:
    let $a: float.%brain.16 := 0.1;                     # a literal rounds to the nearest: 0.10009765625
    let $b: float.%brain.16 := 0.2;
    let $c: float.%brain.16 := $a plus $b;
    std::print.stored($a, " ", $c, newln);       # what is stored
    std::print.shortest($a, " ", $c, newln);  # the shortest text that rounds back to it
    let $x: float.bin.64 := 300.7;
    let $w := to.float.%brain.16($x);                 # into it rounds: 300 (7 bits)
    let $back := to.float.bin.32($w);                # out of it into a f32 is exact
    let $n := to.int.sign.32($w) otherwise neg 1;      # whole, so it converts
    std::print.shortest($w, " ", $back, " ", $n, newln);
    let $xs: std::ls[float.%brain.16] := [0.5, 1.5, 2.5];
    std::print.format("{} {places: 2} {form: stored}{newln}", $xs, $a, $a);
!end of MAIN
