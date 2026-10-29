# The two prints. Both are std's; the base language has no print.
MAIN:
    let $name := "Ann";
    let $count := 6;
    let $x: f64 := 0.1;

    # print.stored: any values, each in its stored form, nothing between
    std::print.stored("x is ", $x, newln);

    # print, which is print.shortest: the same, each in its shortest form
    std::print("x is ", $x, newln);

    # a joined text is already text: print.stored and print agree on it
    let $line := $name concat " scored " concat to.stored.str.utf8($count);
    std::print.stored($line, newln);
    std::print($line, newln);

    # stderr
    std::print.stderr("warning: ", $count, " is low", newln);
;MAIN
