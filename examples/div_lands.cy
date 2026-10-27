# Division is exact and checked where it lands, as `plus`, `minus`, `times` and `pow` are (owner, 2026-10-02: a): the
# smallest i8 divided by -1 is 128, which no i8 holds, but a print takes the true answer.
!MAIN:
    let $a: integer.signed.8_bit = negative 128;
    std::print.shortest(($a divide.truncate negative 1), " ", ($a divide.floor negative 1), " ", ($a modulo.floor negative 1), newline);
    let $b: integer.signed.16_bit = $a divide.nearest negative 1;     # lands in an i16: fits
    std::print.shortest($b, newline);
!end of MAIN
