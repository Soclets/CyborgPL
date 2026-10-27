# Reading, appending to and summing a list.
function sum [lent $items: std::list[integer.signed.32_bit]] -> integer.signed.32_bit:
    variable $total := 0;
    for $n in lend $items:
        change $total = $total plus $n;
    end of for
    return $total;
end of function sum

!MAIN:
    variable $xs: std::list[integer.signed.32_bit] = [10, 20, 30];
    std::print.shortest("first ", $xs[position: 1], ", last ", $xs[position: 3], newline);
    let $n := std::list.length(lend $xs);
    std::print.shortest("length ", $n, newline);
    std::list.append(lend.mutable $xs, value: 40);
    std::list.remove(lend.mutable $xs, position: 1);
    std::print.shortest($xs, " length ", std::list.length(lend $xs), newline);
    std::print.shortest("sum ", sum(lend $xs), newline);
    for $i [from: 1, to: std::list.length(lend $xs), by: 1]:
        std::print.shortest($xs[position: $i], " ");
    end of for
    std::print.stored(newline);
    let $days: array[string.utf8] = ["mon", "tue"];
    std::print.stored($days[position: 2], " of ", array.length(lend $days), newline);
    let $grid := [[1, 2], [3, 4]];
    std::print.shortest($grid[position: 2], newline);
    for permanent $i [from: 1, to: std::list.length(lend $xs), by: 1]:
        if $xs[position: $i] equal_to 30:
            break;
        end of if
    end of for
    change $xs[position: $i] = 33;
    std::print.shortest($xs, newline);
!end of MAIN
