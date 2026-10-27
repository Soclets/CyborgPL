# Lists of lists, and a generic function that counts any list.
function count <T: any> [lent $items: std::list[<T>]] -> integer.signed.32_bit:
    variable $n := 0;
    for $item in lend $items:
        change $n = $n plus 1;
    end of for
    return $n;
end of function count

!MAIN:
    let $grid := [[1, 2], [3, 4]];
    std::print.shortest($grid, newline);
    for $row in lend $grid:
        std::print.shortest(count(lend $row), newline);
    end of for
    std::print.shortest(count(["a", "b", "c"]), newline);
    let $one := [7,];
    std::print.shortest($one, " ", count(lend $one), newline);
!end of MAIN
