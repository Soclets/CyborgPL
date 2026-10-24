MAIN {
    let $xs: std::list[i32] = [1, 2, 3];
    let $z := std::list.slice[lend $xs, pos: 1, pos: 2];
}
