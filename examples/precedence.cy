# Which operator binds first: `times` before `plus`, `pow` before both, and brackets.
MAIN {
    std::print.useful[(1 + 2 x 3), newln];
    std::print.useful[(2 x (3 xx 2)), newln];
    std::print.useful[-(2 xx 2), newln];
    std::print.useful[((-2) xx 2), newln];
    std::print.useful[(10 - 4 - 3), newln];
    std::print.useful[(6 div.floor (2 x 3)), newln];
    std::print.useful[((6 div.floor 2) x 3), newln];
    let 'a' := 5;
    let 'b' := 7;
    if 'a' + 1 > 'b' - 2 and not ('a' equal_to 'b') or false {
        std::print.stored["logic holds", newln];
    }
}
