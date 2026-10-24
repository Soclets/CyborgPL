extern.C fn exit [$code: i32];

fn grade [$n: i32] -> i32 {
    if $n > 90 { return 1; }
    else if $n > 50 { return 2; }
    else { return 3; }
}

MAIN {
    let $a: i32 = grade(95);
    let $b: i32 = grade(60);
    let $c: i32 = grade(10);
    no_check my.exit(($a times 100) plus ($b times 10) plus $c);     # 123
}
