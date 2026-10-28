# Division is exact and checked where it lands, as `plus`, `minus`, `times` and `pow` are (owner, 2026-10-02: a): the
# smallest i8 divided by -1 is 128, which no i8 holds, but a print takes the true answer.
MAIN:
    let $a: i8 := -128;
    std::println(($a div.trunc -1), " ", ($a div.floor -1), " ", ($a mod.floor -1));
    let $b: i16 := $a div.near -1;     # lands in an i16: fits
    std::println($b);
;MAIN
