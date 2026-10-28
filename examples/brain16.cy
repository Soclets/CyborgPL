# The brain float (owner, 2026-10-02): `brain16`. f32's range in 16 bits,
# with 7 bits of precision; its own family; each operation rounds to the nearest brain16.
MAIN:
    let $a: %bf16 := 0.1;                     # a literal rounds to the nearest: 0.10009765625
    let $b: %bf16 := 0.2;
    let $c: %bf16 := $a + $b;
    std::println.stored($a, " ", $c);       # what is stored
    std::println($a, " ", $c);  # the shortest text that rounds back to it
    let $x: f64 := 300.7;
    let $w := to.%bf16($x);                 # into it rounds: 300 (7 bits)
    let $back := to.f32($w);                # out of it into a f32 is exact
    let $n := to.i32($w) ?? -1;      # whole, so it converts
    std::println($w, " ", $back, " ", $n);
    let $xs: std::ls[%bf16] := [0.5, 1.5, 2.5];
    std::println.format("{} {places: 2} {form: stored}", $xs, $a, $a);
;MAIN
