extern.C fn exit ['code': i32];

MAIN {
    let mut 'total': i32 = 0;
    for 'i' [from: 1, to: 10, by: 1] {
        change 'total' = 'total' plus 'i';
    }
    let mut 'n': i32 = 0;
    while 'n' < 3 { change 'n' = 'n' plus 1; }
    no_check my.exit['total' plus 'n'];                      # 55 + 3 = 58
}
