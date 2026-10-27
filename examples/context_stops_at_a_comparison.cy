# The type a value is stored into stops at a comparison, as it stops at a call: a `bool`
# holds the comparison's answer, and its operands take their width from each other (2026-09-23).
!MAIN:
    let $w: integer.unsigned.32_bit = 3;
    let $b: boolean = (5 minus.wrap $w) greater_than 1;          # a u32 subtraction, 'w' says so, not a bool one
    let $c: boolean = 1 less_than (0 minus.wrap $w);          # wraps to 4294967293
    let $n: integer.unsigned.64_bit = 5;
    let $d: boolean = $n greater_than (0 minus.wrap 1);          # but the other side sizes it: 0 minus.wrap 1 wraps in u64
    std::print.shortest($b, " ", $c, " ", 0 minus.wrap $w, " ", $d, newline);
!end of MAIN
