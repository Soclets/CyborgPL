# The brain float (owner, 2026-10-02): `brain16`. f32's range in 16 bits,
# with 7 bits of precision; its own family; each operation rounds to the nearest brain16.
MAIN {
    let $a: brain16 = 0.1;                     # a literal rounds to the nearest: 0.10009765625
    let $b: brain16 = 0.2;
    let $c: brain16 = $a + $b;
    std::print.stored($a, " ", $c, newln);       # what is stored
    std::print.useful($a, " ", $c, newln);  # the shortest text that rounds back to it
    let $x: f64 = 300.7;
    let $w := to.brain16($x);                 # into it rounds: 300 (7 bits)
    let $back := to.f32($w);                # out of it into a f32 is exact
    let $n := to.i32($w) otherwise -1;      # whole, so it converts
    std::print.useful($w, " ", $back, " ", $n, newln);
    let $xs: std::list[brain16] = [0.5, 1.5, 2.5];
    std::print.format("{} {:.2} {:?}\n", $xs, $a, $a);
}
