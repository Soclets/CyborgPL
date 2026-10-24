# the borrow rule holds on every path, not only the one a run takes (check claude, 2026-09-29)
MAIN {
    let $n: i32 = 3;
    let mut $t: std::list[i32] = [5,];
    if $n > 5 {
        let $q := lend $t;
        std::list.append(lend.mut $t, value: 1);
        std::print.useful(std::list.length(lend $q), newln);
    }
}
