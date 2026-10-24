struct Point { x: i32, y: i32 }
MAIN {
    let mut $p := Point[x = 1, y = 2];
    for $v in lend.mut $p { }
}
