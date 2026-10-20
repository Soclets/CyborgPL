# Functions that take any number of arguments, typed or not.
fn sum [...'nums'] -> f64 {
    let 'total': f64 = 0;
    for 'n' in lend 'nums' { change 'total' = 'total' + to.f64['n']; }
    return 'total';
}

fn count_ints [...'nums': i32] -> u64 {
    return list.length[lend 'nums'];
}

fn show [lent 'label': str.utf8, lent ...'parts'] {
    std::print.useful['label', ": "];
    for 'p' in lend 'parts' { std::print.useful['p', " "]; }
    std::print.stored[newln];
}

MAIN {
    std::print.useful[sum[1, 2.5, 3], newln];
    std::print.useful[sum[], newln];
    std::print.useful[count_ints[4, 5, 6], newln];
    let 'name' := "Ann";
    show["mixed", lend 'name', 7, true, `x`];
    show["none"];
}
