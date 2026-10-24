struct Point {
    x: i32,
    y: i32
}
MAIN {
    let $p := Point[x = 3, y = 4];
    let $t := to.str.utf8($p);
}
