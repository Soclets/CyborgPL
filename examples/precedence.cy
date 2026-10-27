# Which operator binds first: `times` before `plus`, `pow` before both, and brackets.
!MAIN:
    std::print.shortest((1 plus 2 times 3), newline);
    std::print.shortest((2 times (3 power 2)), newline);
    std::print.shortest(negate (2 power 2), newline);
    std::print.shortest(((negative 2) power 2), newline);
    std::print.shortest((10 minus 4 minus 3), newline);
    std::print.shortest((6 divide.floor (2 times 3)), newline);
    std::print.shortest(((6 divide.floor 2) times 3), newline);
    let $a := 5;
    let $b := 7;
    if $a plus 1 greater_than $b minus 2 and not ($a equal_to $b) or false:
        std::print.stored("logic holds", newline);
    end of if
!end of MAIN
