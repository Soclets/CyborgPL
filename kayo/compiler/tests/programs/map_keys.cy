%extern.C proc exit [let $code: i32];
rec Point:
    var x: i32; var y: i32;
;rec Point
MAIN:
    var $big: std::map[i32, i32] = std::map.empty;
    for $i: i32 [from: 1, to: 200, by: 1]:
        std::map.set(lend.mut $big, key: $i, value: $i x 2);
    ;for
    let $v := std::map.get(lend $big, key: 150) ?? 0;
    let $gone := std::map.remove(lend.mut $big, key: 1) ?? 0;
    let $n: i32 := std::map.count(lend $big);
    var $grid: std::map[Point, i32] = std::map.empty;
    std::map.set(lend.mut $grid, key: Point[x = 1, y = 2], value: 9);
    let $p := std::map.get(lend $grid, key: Point[x = 1, y = 2]) ?? 0;
    let $q := std::map.get(lend $grid, key: Point[x = 9, y = 9]) ?? 5;
    no_check my.exit($v - 290 + $gone + $n - 197 + $p + $q);
;MAIN
