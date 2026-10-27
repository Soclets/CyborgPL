# Joining text with `++` (`concat`), and converting numbers to text first.
!MAIN:
    let $n := 3;
    let $s := "n=" concatenate to.stored.string.utf8($n plus 4);  # arithmetic, convert, then join
    std::print.stored($s, newline);
    std::print.stored("count: " concatenate to.stored.string.utf8(10), newline);   # a number is converted, in its stored form
    let $x: float.binary.64_bit = 0.1;
    let $t := "x is " concatenate to.stored.string.utf8($x);   # stored form, always
    std::print.shortest($t, newline);             # print.useful cannot undo a join…
    std::print.shortest("x is ", $x, newline);    # …but renders its own values usefully
    if "a" concatenate "b" equal_to "ab":
        std::print.stored("joined", newline);
    end of if
    let $c := `é` concatenate "!";
    std::print.stored($c, newline);
!end of MAIN
