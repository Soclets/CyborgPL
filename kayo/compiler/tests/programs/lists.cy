%extern.C fn exit [$code: i32];
MAIN {
    let mut $xs: std::list[i32] = [10, 20, 30];
    std::list.append(lend.mut $xs, value: 12);
    let $n: i32 = std::list.length(lend $xs);
    let $a: i32 = $xs[pos: 1];
    let $d: i32 = $xs[pos: 4];
    no_check my.exit($n plus $a plus $d);
}
