# Empty lists: `[]` with a type, and `list.empty`, then filled with `list.append`.
!MAIN:
    variable $xs: std::list[integer.signed.32_bit] = [];
    std::list.append(lend.mutable $xs, value: 7);
    std::list.append(lend.mutable $xs, value: 8);
    std::print.shortest($xs, " ", std::list.length(lend $xs), newline);
    variable $ys := std::list.empty;
    std::print.shortest(std::list.length(lend $ys), newline);
    std::list.append(lend.mutable $ys, value: "a");
    std::print.shortest($ys, newline);
!end of MAIN
