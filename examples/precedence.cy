# Which operator binds first: `times` before `plus`, `pow` before both, and brackets.
MAIN {
    std::print.useful[(1 plus 2 times 3), newln];
    std::print.useful[(2 times (3 pow 2)), newln];
    std::print.useful[-(2 pow 2), newln];
    std::print.useful[((-2) pow 2), newln];
    std::print.useful[(10 minus 4 minus 3), newln];
    std::print.useful[(6 div.floor (2 times 3)), newln];
    std::print.useful[((6 div.floor 2) times 3), newln];
    let 'a' := 5;
    let 'b' := 7;
    if 'a' plus 1 > 'b' minus 2 and not ('a' equal_to 'b') or false {
        std::print.stored["logic holds", newln];
    }
}
