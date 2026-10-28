# The type a value is stored into stops at a comparison, as it stops at a call: a `bool`
# holds the comparison's answer, and its operands take their width from each other (2026-09-23).
MAIN:
    let $w: u32 := 3;
    let $b: bool := (5 minus.wrap $w) > 1;          # a u32 subtraction, 'w' says so, not a bool one
    let $c: bool := 1 < (0 minus.wrap $w);          # wraps to 4294967293
    let $n: u64 := 5;
    let $d: bool := $n > (0 minus.wrap 1);          # but the other side sizes it: 0 minus.wrap 1 wraps in u64
    std::print($b, " ", $c, " ", 0 minus.wrap $w, " ", $d, newln);
;MAIN
