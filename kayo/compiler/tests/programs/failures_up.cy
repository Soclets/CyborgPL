extern.C fn exit [$code: i32];
fn inner [$n: i32] -> i32 or fail[bad: std::list[i32]] {
    if $n < 0 { fail bad[[1, 2, 3]]; }
    return $n;
}
fn outer [$n: i32] -> i32 or fail[bad: std::list[i32]] {
    let $v := inner($n) otherwise fail;
    return $v plus 1;
}
MAIN {
    let $ok := outer(5) otherwise 0;
    let mut $n: i32 = 0;
    outer(-1) otherwise {
        bad $why { change $n = std::list.length(lend $why); }
    };
    no_check my.exit($ok plus $n);
}
