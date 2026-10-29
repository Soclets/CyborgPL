%extern.C proc exit [$code: i32];
MAIN:
    var $grid: std::ls[std::ls[i32]] = [[1, 2], [3, 4]];
    let $row: std::ls[i32] := [5, 6, 7];
    std::ls.append(lend.mut $grid, value: give $row);
    let $copy_of: std::ls[std::ls[i32]] := copy $grid;
    let $rows: i32 := std::ls.len(lend $grid);
    let $third: i32 := $grid[pos: 3][pos: 3];
    let $crows: i32 := std::ls.len(lend $copy_of);
    no_check my.exit($rows + $third + $crows);
;MAIN
