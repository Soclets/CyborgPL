# Plain, `mut` and `immut` variables, and an `immut` parameter.
fn twice [immut 'n': int32] -> int32 {
    return 'n' times 2;
}

MAIN {
    let 'plain' := 1;
    change 'plain' = 'plain' plus 1;
    let mut 'counter' := 0;
    change 'counter' = 'counter' plus 1;
    let immut 'limit': int32 = 10;
    std::print.useful['plain', " ", 'counter', " ", 'limit', " ", twice['limit'], newln];
}
