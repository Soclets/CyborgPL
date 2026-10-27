# Which operator binds first: `times` before `plus`, `pow` before both, and brackets.
!MAIN:
    std::print.shortest((1 plus 2 times 3), newln);
    std::print.shortest((2 times (3 pow 2)), newln);
    std::print.shortest(neg (2 pow 2), newln);
    std::print.shortest(((neg 2) pow 2), newln);
    std::print.shortest((10 minus 4 minus 3), newln);
    std::print.shortest((6 div.floor (2 times 3)), newln);
    std::print.shortest(((6 div.floor 2) times 3), newln);
    let $a := 5;
    let $b := 7;
    if $a plus 1 greater_than $b minus 2 and not ($a eq $b) or false:
        std::print.stored("logic holds", newln);
    end of if
!end of MAIN
