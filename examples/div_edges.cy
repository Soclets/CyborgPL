# Edges of `div.X` / `mod.X` on numbers with a fraction (owner, 2026-10-02). The quotient is the true
# one, rounded as named (Python's `//` agrees); a remainder that rounds to the divisor itself is 0, so
# it stays below the divisor, checked again at the type's own width (f32, brain16).
!MAIN:
    let $x: float.bin.64 := 0.00000000000000000001;
    let $n: float.bin.64 := neg 0.00000000000000000001;
    let $two: float.bin.64 := 2.0;
    let $turn: float.bin.64 := 360.0;
    std::print.shortest(($x div.ceil $two), " ", ($x mod.ceil $two), " ", ($n div.floor $turn), " ", ($n mod.floor $turn), newln);
    let $a: float.bin.32 := neg 0.000001;
    let $b: float.bin.32 := 360.0;
    std::print.shortest(($a mod.floor $b), newln);
    let $h: float.%brain.16 := neg 1.0;
    let $k: float.%brain.16 := 360.0;
    let $one: float.%brain.16 := 1.0;
    std::print.shortest(($h mod.floor $k), " ", ($one mod.ceil $k), newln);
!end of MAIN
