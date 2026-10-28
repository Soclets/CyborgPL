# The two prints. Both are std's; the base language has no print.
MAIN:
    let $name := "Ann";
    let $count := 6;
    let $x: f64 := 0.1;

    # print: any values, each in its stored form, nothing between
    std::println.stored("x is ", $x);

    # print.useful: the same, each in its useful form
    std::println("x is ", $x);

    # a joined text is already text: print and print.useful agree on it
    let $line := $name ++ " scored " ++ to.stored.str.utf8($count);
    std::println.stored($line);
    std::println($line);

    # stderr
    std::print.shortest.stderr("warning: ", $count, " is low", newln);
;MAIN
