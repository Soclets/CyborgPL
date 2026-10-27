# The two prints. Both are std's; the base language has no print.
!MAIN:
    let $name := "Ann";
    let $count := 6;
    let $x: float.binary.64_bit = 0.1;

    # print: any values, each in its stored form, nothing between
    std::print.stored("x is ", $x, newline);

    # print.useful: the same, each in its useful form
    std::print.shortest("x is ", $x, newline);

    # a joined text is already text: print and print.useful agree on it
    let $line := $name concatenate " scored " concatenate to.stored.string.utf8($count);
    std::print.stored($line, newline);
    std::print.shortest($line, newline);

    # stderr
    std::print.shortest.stderr("warning: ", $count, " is low", newline);
!end of MAIN
