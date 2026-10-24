extern.C fn exit [$code: i32];
struct Point { x: i32, y: i32 }
MAIN {
    let mut $big: std::map[i32, i32] = std::map.empty;
    for $i: i32 [from: 1, to: 200, by: 1] {
        std::map.set(lend.mut $big, key: $i, value: $i times 2);
    }
    let $v := std::map.get(lend $big, key: 150) otherwise 0;
    let $gone := std::map.remove(lend.mut $big, key: 1) otherwise 0;
    let $n: i32 = std::map.length(lend $big);
    let mut $grid: std::map[Point, i32] = std::map.empty;
    std::map.set(lend.mut $grid, key: Point[x = 1, y = 2], value: 9);
    let $p := std::map.get(lend $grid, key: Point[x = 1, y = 2]) otherwise 0;
    let $q := std::map.get(lend $grid, key: Point[x = 9, y = 9]) otherwise 5;
    no_check my.exit($v minus 290 plus $gone plus $n minus 197 plus $p plus $q);
}
