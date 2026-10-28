# the borrow rule holds on every path, not only the one a run takes (check claude, 2026-09-29)
MAIN:
    let $n: i32 := 3;
    var $t: std::ls[i32] = [5,];
    if $n > 5:
        let $q := lend $t;
        std::ls.append(lend.mut $t, value: 1);
        std::println(std::ls.len(lend $q));
    ;if
;MAIN
