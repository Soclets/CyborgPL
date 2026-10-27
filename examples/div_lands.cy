# Division is exact and checked where it lands, as `plus`, `minus`, `times` and `pow` are (owner, 2026-10-02: a): the
# smallest i8 divided by -1 is 128, which no i8 holds, but a print takes the true answer.
!MAIN:
    let $a: int.sign.8 := neg 128;
    std::print.shortest(($a div.trunc neg 1), " ", ($a div.floor neg 1), " ", ($a mod.floor neg 1), newln);
    let $b: int.sign.16 := $a div.near neg 1;     # lands in an i16: fits
    std::print.shortest($b, newln);
!end of MAIN
