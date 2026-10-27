# Edges of `div.X` / `mod.X` on numbers with a fraction (owner, 2026-10-02). The quotient is the true
# one, rounded as named (Python's `//` agrees); a remainder that rounds to the divisor itself is 0, so
# it stays below the divisor, checked again at the type's own width (f32, brain16).
!MAIN:
    let $x: float.binary.64_bit = 0.00000000000000000001;
    let $n: float.binary.64_bit = negative 0.00000000000000000001;
    let $two: float.binary.64_bit = 2.0;
    let $turn: float.binary.64_bit = 360.0;
    std::print.shortest(($x divide.ceiling $two), " ", ($x modulo.ceiling $two), " ", ($n divide.floor $turn), " ", ($n modulo.floor $turn), newline);
    let $a: float.binary.32_bit = negative 0.000001;
    let $b: float.binary.32_bit = 360.0;
    std::print.shortest(($a modulo.floor $b), newline);
    let $h: float.%brain.16_bit = negative 1.0;
    let $k: float.%brain.16_bit = 360.0;
    let $one: float.%brain.16_bit = 1.0;
    std::print.shortest(($h modulo.floor $k), " ", ($one modulo.ceiling $k), newline);
!end of MAIN
