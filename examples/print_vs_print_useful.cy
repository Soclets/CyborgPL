# `print.stored` shows a value as stored; `print.useful` shows it for people.
MAIN {
    let 'x': bin64 = 0.1;
    std::print.stored['x', newln];
    std::print.useful['x', newln];
    let 'y': bin16 = 1000;
    std::print.useful['y', newln];
    let 'big' := 1_000_000 x 1_000_000;
    std::print.stored['big', newln];
    let 's' := "x is " concat to.stored.str.utf8['x'];
    std::print.stored['s', newln];
    std::print.useful['s', newln];
    std::print.useful["x is ", 'x', newln];
}
