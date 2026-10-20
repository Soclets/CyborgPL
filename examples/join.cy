# Joining text with `++` (`concat`), and converting numbers to text first.
MAIN {
    let 'n' := 3;
    let 's' := "n=" concat to.stored.str.utf8['n' + 4];  # arithmetic, convert, then join
    std::print.stored['s', newln];
    std::print.stored["count: " concat to.stored.str.utf8[10], newln];   # a number is converted, in its stored form
    let 'x': f64 = 0.1;
    let 't' := "x is " concat to.stored.str.utf8['x'];   # stored form, always
    std::print.useful['t', newln];             # print.useful cannot undo a join…
    std::print.useful["x is ", 'x', newln];    # …but renders its own values usefully
    if "a" concat "b" equal_to "ab" { std::print.stored["joined", newln]; }
    let 'c' := `é` concat "!";
    std::print.stored['c', newln];
}
