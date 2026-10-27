# Reading, appending to and summing a list.
fn sum [lent $items: std::ls[int.sign.32]] -> int.sign.32:
    var $total := 0;
    for $n in lend $items:
        change $total = $total plus $n;
    end of for
    ret $total;
end of fn sum

!MAIN:
    var $xs: std::ls[int.sign.32] = [10, 20, 30];
    std::print.shortest("first ", $xs[pos: 1], ", last ", $xs[pos: 3], newln);
    let $n := std::ls.len(lend $xs);
    std::print.shortest("length ", $n, newln);
    std::ls.append(lend.mut $xs, value: 40);
    std::ls.remove(lend.mut $xs, pos: 1);
    std::print.shortest($xs, " length ", std::ls.len(lend $xs), newln);
    std::print.shortest("sum ", sum(lend $xs), newln);
    for $i [from: 1, to: std::ls.len(lend $xs), by: 1]:
        std::print.shortest($xs[pos: $i], " ");
    end of for
    std::print.stored(newln);
    let $days: arr[str.utf8] = ["mon", "tue"];
    std::print.stored($days[pos: 2], " of ", arr.len(lend $days), newln);
    let $grid := [[1, 2], [3, 4]];
    std::print.shortest($grid[pos: 2], newln);
    for perm $i [from: 1, to: std::ls.len(lend $xs), by: 1]:
        if $xs[pos: $i] eq 30:
            break;
        end of if
    end of for
    change $xs[pos: $i] = 33;
    std::print.shortest($xs, newln);
!end of MAIN
