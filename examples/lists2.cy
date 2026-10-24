# Reading, appending to and summing a list.
fn sum [lent $items: std::list[i32]] -> i32 {
    let $total := 0;
    for $n in lend $items {
        change $total = $total + $n;
    }
    return $total;
}

MAIN {
    let $xs: std::list[i32] = [10, 20, 30];
    std::print.useful("first ", $xs[pos: 1], ", last ", $xs[pos: 3], newln);
    let $n := std::list.length(lend $xs);
    std::print.useful("length ", $n, newln);
    std::list.append(lend.mut $xs, value: 40);
    std::list.remove[lend.mut $xs, pos: 1];
    std::print.useful($xs, " length ", std::list.length(lend $xs), newln);
    std::print.useful("sum ", sum(lend $xs), newln);
    for $i [from: 1, to: std::list.length(lend $xs), by: 1] {
        std::print.useful($xs[pos: $i], " ");
    }
    std::print.stored(newln);
    let $days: array[str.utf8] = ["mon", "tue"];
    std::print.stored($days[pos: 2], " of ", array.length(lend $days), newln);
    let $grid := [[1, 2], [3, 4]];
    std::print.useful($grid[pos: 2], newln);
    for perm $i [from: 1, to: std::list.length(lend $xs), by: 1] {
        if $xs[pos: $i] equal_to 30 { break; }
    }
    change $xs[pos: $i] = 33;
    std::print.useful($xs, newln);
}
