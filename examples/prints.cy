# The two prints. Both are std's; the base language has no print.
MAIN {
    let 'name' := "Ann";
    let 'count' := 6;
    let 'x': f64 = 0.1;

    # print: any values, each in its stored form, nothing between
    std::print.stored["x is ", 'x', newln];

    # print.useful: the same, each in its useful form
    std::print.useful["x is ", 'x', newln];

    # a joined text is already text: print and print.useful agree on it
    let 'line' := 'name' concat " scored " concat to.stored.str.utf8['count'];
    std::print.stored['line', newln];
    std::print.useful['line', newln];

    # stderr
    std::print.useful.stderr["warning: ", 'count', " is low", newln];
}
