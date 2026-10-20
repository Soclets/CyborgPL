extern.C fn exit ['code': i32];

struct Point { x: i32, y: i32 }

fn area [lent 'p': Point] -> i32 { return 'p'.x times 'p'.y; }

MAIN {
    let 'p' := Point[x = 6, y = 7];
    no_check my.exit[area[lend 'p']];                     # 42
}
