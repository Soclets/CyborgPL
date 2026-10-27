!MAIN:
    variable $x: integer.signed.32_bit = 1;
    shadow variable $x: integer.signed.32_bit = 2;
    change $x = 3;
    std::print.shortest($x, newline);
!end of MAIN
