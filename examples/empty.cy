# Empty lists: `[]` with a type, and `list.empty`, then filled with `list.append`.
MAIN:
    var $xs: std::ls[i32] = [];
    std::ls.append(lend.mut $xs, value: 7);
    std::ls.append(lend.mut $xs, value: 8);
    std::println($xs, " ", std::ls.len(lend $xs));
    var $ys = std::ls.empty;
    std::println(std::ls.len(lend $ys));
    std::ls.append(lend.mut $ys, value: "a");
    std::println($ys);
;MAIN
