# Over `lend.mut $xs`, the loop's element is the list's own element: changing it changes the list
# (owner, 2026-09-30: "the program looks legal to me. `mut`'s already there"). Over `lend`, it cannot
# be changed.
!MAIN:
    variable $prices: std::list[integer.signed.32_bit] = [10, 20, 30];
    for $x in lend.mutable $prices:
        change $x = $x times 2;
    end of for
    std::print.stored($prices, newline);
!end of MAIN
