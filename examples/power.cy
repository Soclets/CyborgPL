# `pow`, and how it groups (to the right).
!MAIN:
    std::print.shortest((2 pow (3 pow 2)), newln);
    std::print.shortest(((2 pow 3) pow 2), newln);
    let $kilo := 2 pow 10;
    std::print.shortest($kilo, newln);
    std::print.shortest((10 minus 3 minus 2), newln);
!end of MAIN
