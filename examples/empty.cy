# Empty lists: `[]` with a type, and `list.empty`, then filled with `list.append`.
!MAIN:
    var $xs: std::ls[int.sign.32] = [];
    std::ls.append(lend.mut $xs, value: 7);
    std::ls.append(lend.mut $xs, value: 8);
    std::print.shortest($xs, " ", std::ls.len(lend $xs), newln);
    var $ys = std::ls.empty;
    std::print.shortest(std::ls.len(lend $ys), newln);
    std::ls.append(lend.mut $ys, value: "a");
    std::print.shortest($ys, newln);
!end of MAIN
