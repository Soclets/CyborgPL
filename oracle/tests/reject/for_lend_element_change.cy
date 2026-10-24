MAIN {
    let $xs: std::list[i32] = [1, 2];
    for $x in lend $xs {
        change $x = $x plus 1;
    }
}
