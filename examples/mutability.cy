# Plain, `mut` and `immut` variables, and an `immut` parameter.
fn twice [immut 'n': i32] -> i32 {
    return 'n' x 2;
}

MAIN {
    let 'plain' := 1;
    change 'plain' = 'plain' + 1;
    let mut 'counter' := 0;
    change 'counter' = 'counter' + 1;
    let immut 'limit': i32 = 10;
    std::print.useful['plain', " ", 'counter', " ", 'limit', " ", twice['limit'], newln];
}
