# Reading, appending to and summing a list.
fn sum [lent $items: std::ls[i32]] -> i32:
    var $total = 0;
    for $n in lend $items:
        change $total to $total + $n;
    ;for
    ret $total;
;fn sum

MAIN:
    var $xs: std::ls[i32] = [10, 20, 30];
    std::print("first ", $xs[pos: 1], ", last ", $xs[pos: 3], newln);
    let $n := std::ls.len(lend $xs);
    std::print("length ", $n, newln);
    std::ls.append(lend.mut $xs, value: 40);
    std::ls.remove(lend.mut $xs, pos: 1);
    std::print($xs, " length ", std::ls.len(lend $xs), newln);
    std::print("sum ", sum(lend $xs), newln);
    for $i [from: 1, to: std::ls.len(lend $xs), by: 1]:
        std::print($xs[pos: $i], " ");
    ;for
    std::print.stored(newln);
    let $days: arr[str.utf8] := ["mon", "tue"];
    std::print.stored($days[pos: 2], " of ", arr.len(lend $days), newln);
    let $grid := [[1, 2], [3, 4]];
    std::print($grid[pos: 2], newln);
    for perm $i [from: 1, to: std::ls.len(lend $xs), by: 1]:
        if $xs[pos: $i] == 30:
            break;
        ;if
    ;for
    change $xs[pos: $i] to 33;
    std::print($xs, newln);
;MAIN
