# Edges of `div.X` / `mod.X` on numbers with a fraction (owner, 2026-10-02). The quotient is the true
# one, rounded as named (Python's `//` agrees); a remainder that rounds to the divisor itself is 0, so
# it stays below the divisor, checked again at the type's own width (f32, brain16).
MAIN:
    let $x: f64 := 0.00000000000000000001;
    let $n: f64 := -0.00000000000000000001;
    let $two: f64 := 2.0;
    let $turn: f64 := 360.0;
    std::print(($x div.ceil $two), " ", ($x mod.ceil $two), " ", ($n div.floor $turn), " ", ($n mod.floor $turn), newln);
    let $a: f32 := -0.000001;
    let $b: f32 := 360.0;
    std::print(($a mod.floor $b), newln);
    let $h: %bf16 := -1.0;
    let $k: %bf16 := 360.0;
    let $one: %bf16 := 1.0;
    std::print(($h mod.floor $k), " ", ($one mod.ceil $k), newln);
;MAIN
