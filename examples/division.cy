# Division and remainder: `div.floor` and `mod.floor` by default, `div.trunc` and `mod.trunc` by name.
!MAIN:
    # floor division and the mathematician's mod.floor (decided 2026-09-20)
    std::print.shortest(7 divide.floor 2, " ", 7 modulo.floor 2, newline);
    std::print.shortest(negative 7 divide.floor 2, " ", negative 7 modulo.floor 2, newline);        # -4 and 1: the clock never goes negative
    std::print.shortest(7 divide.floor negative 2, " ", 7 modulo.floor negative 2, newline);        # -4 and -1: the result has the divisor's sign
    let $hour := negative 1;
    std::print.shortest(($hour modulo.floor 12) plus 1, newline);         # an hour before 1 o'clock is 12
    # hardware truncation, by name, when a C library's answer must match bit for bit
    std::print.shortest((negative 7 divide.truncate 2), " ", (negative 7 modulo.truncate 2), newline);
!end of MAIN
