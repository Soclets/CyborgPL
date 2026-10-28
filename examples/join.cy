# Joining text with `++` (`concat`), and converting numbers to text first.
MAIN:
    let $n := 3;
    let $s := "n=" ++ to.stored.str.utf8($n + 4);  # arithmetic, convert, then join
    std::println.stored($s);
    std::println.stored("count: " ++ to.stored.str.utf8(10));   # a number is converted, in its stored form
    let $x: f64 := 0.1;
    let $t := "x is " ++ to.stored.str.utf8($x);   # stored form, always
    std::println($t);             # print.useful cannot undo a join…
    std::println("x is ", $x);    # …but renders its own values usefully
    if "a" ++ "b" == "ab":
        std::println.stored("joined");
    ;if
    let $c := `é` ++ "!";
    std::println.stored($c);
;MAIN
