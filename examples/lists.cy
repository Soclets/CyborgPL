# Lists of lists, and a generic function that counts any list.
fn count [<T>: any] [lent $items: std::ls[<T>]] -> i32:
    var $n = 0;
    for $item in lend $items:
        change $n to $n + 1;
    ;for
    ret $n;
;fn count

MAIN:
    let $grid := [[1, 2], [3, 4]];
    std::print($grid, newln);
    for $row in lend $grid:
        std::print(count(lend $row), newln);
    ;for
    std::print(count(["a", "b", "c"]), newln);
    let $one := [7,];
    std::print($one, " ", count(lend $one), newln);
;MAIN
