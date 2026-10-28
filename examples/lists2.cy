# Reading, appending to and summing a list.
fn sum [lent $items: std::ls[i32]] -> i32:
    var $total = 0;
    for $n in lend $items:
        change $total += $n;
    ;for
    ret $total;
;fn sum

MAIN:
    var $xs: std::ls[i32] = [10, 20, 30];
    std::println("first ", $xs[pos: 1], ", last ", $xs[pos: 3]);
    let $n := std::ls.len(lend $xs);
    std::println("length ", $n);
    std::ls.append(lend.mut $xs, value: 40);
    std::ls.remove(lend.mut $xs, pos: 1);
    std::println($xs, " length ", std::ls.len(lend $xs));
    std::println("sum ", sum(lend $xs));
    for $i [from: 1, to: std::ls.len(lend $xs), by: 1]:
        std::print($xs[pos: $i], " ");
    ;for
    std::println.stored();
    let $days: arr[str.utf8] := ["mon", "tue"];
    std::println.stored($days[pos: 2], " of ", arr.len(lend $days));
    let $grid := [[1, 2], [3, 4]];
    std::println($grid[pos: 2]);
    for perm $i [from: 1, to: std::ls.len(lend $xs), by: 1]:
        if $xs[pos: $i] == 30:
            break;
        ;if
    ;for
    change $xs[pos: $i] to 33;
    std::println($xs);
;MAIN
