extern.C fn exit [$code: i32];
MAIN {
    let $xs: std::list[i32] = [4, 5, 6];
    let mut $acc: i32 = 0;
    for $x in lend $xs {
        change $acc = $acc plus $x;
    }
    no_check my.exit($acc);
}
