# Every division says how it rounds (owner, 2026-10-02): `floor` down, `ceil` up, `trunc` toward
# zero, `near` to the nearest (a tie to even). `mod.X` is what `div.X` leaves over:
# a equal_to (a div.X b) times b plus (a mod.X b). On numbers with a fraction, `div.near` is ordinary division
# and `floor`/`ceil`/`trunc` give a whole quotient.
!MAIN:
    std::print.shortest((7 divide.floor 2), " ", (7 divide.ceiling 2), " ", (7 divide.truncate 2), " ", (7 divide.nearest 2), newline);
    std::print.shortest((negative 7 divide.floor 2), " ", (negative 7 divide.ceiling 2), " ", (negative 7 divide.truncate 2), " ", (negative 7 divide.nearest 2), newline);
    std::print.shortest((7 modulo.floor 2), " ", (7 modulo.ceiling 2), " ", (7 modulo.truncate 2), " ", (7 modulo.nearest 2), newline);
    std::print.shortest((negative 7 modulo.floor 2), " ", (negative 7 modulo.ceiling 2), " ", (negative 7 modulo.truncate 2), " ", (negative 7 modulo.nearest 2), newline);
    std::print.shortest((5 divide.nearest 2), " ", (5 modulo.nearest 2), newline);        # 2.5 is a tie: to the even 2, 1 left
    let $items: integer.signed.32_bit = 7;
    let $per_page: integer.signed.32_bit = 2;
    std::print.shortest("pages: ", ($items divide.ceiling $per_page), newline);
    let $board: float.binary.64_bit = 7.5;
    let $plank: float.binary.64_bit = 2.0;
    std::print.shortest(($board divide.nearest $plank), " ", ($board divide.floor $plank), " ", ($board modulo.floor $plank), " ", ($board divide.ceiling $plank), " ", ($board modulo.ceiling $plank), newline);
    let $p: float.%decimal.64_bit = negative 7.5;
    let $q: float.%decimal.64_bit = 2.0;
    std::print.shortest(($p divide.nearest $q), " ", ($p divide.floor $q), " ", ($p modulo.floor $q), " ", ($p divide.truncate $q), " ", ($p modulo.truncate $q), " ", ($p divide.nearest $q), newline);
!end of MAIN
