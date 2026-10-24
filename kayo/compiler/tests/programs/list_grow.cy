extern.C fn exit [$code: i32];
MAIN {
    let mut $xs: std::list[i32] = [];
    for $i: i32 [from: 1, to: 100, by: 1] {
        std::list.append(lend.mut $xs, value: $i);
    }
    let $len: i32 = std::list.length(lend $xs);
    let $last: i32 = $xs[pos: 100];
    let $first: i32 = std::list.remove(lend.mut $xs, pos: 1);
    let $now_first: i32 = $xs[pos: 1];
    let $shorter: i32 = std::list.length(lend $xs);
    no_check my.exit($len minus $last plus $first plus $now_first plus $shorter minus 99);
}
