%extern.C fn getpid [] -> i32;
%extern.C fn exit [$code: i32];
MAIN {
    let $xs: std::list[i32] = [1, 2, 3];
    no_check let $n: i32 = getpid();
    let $v: i32 = $xs[pos: $n];
    no_check my.exit($v);
}
