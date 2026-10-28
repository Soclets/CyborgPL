# Lists of lists, and a generic function that counts any list.
fn count <T: any> [lent $items: std::ls[<T>]] -> i32:
    var $n = 0;
    for $item in lend $items:
        change $n += 1;
    ;for
    ret $n;
;fn count

MAIN:
    let $grid := [[1, 2], [3, 4]];
    std::println($grid);
    for $row in lend $grid:
        std::println(count(lend $row));
    ;for
    std::println(count(["a", "b", "c"]));
    let $one := [7,];
    std::println($one, " ", count(lend $one));
;MAIN
