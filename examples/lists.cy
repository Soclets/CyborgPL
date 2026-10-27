# Lists of lists, and a generic function that counts any list.
fn count <T: any> [lent $items: std::ls[<T>]] -> int.sign.32:
    var $n = 0;
    for $item in lend $items:
        change $n to $n plus 1;
    end of for
    ret $n;
end of fn count

!MAIN:
    let $grid := [[1, 2], [3, 4]];
    std::print.shortest($grid, newln);
    for $row in lend $grid:
        std::print.shortest(count(lend $row), newln);
    end of for
    std::print.shortest(count(["a", "b", "c"]), newln);
    let $one := [7,];
    std::print.shortest($one, " ", count(lend $one), newln);
!end of MAIN
