# `print.stored` shows a value as stored; `print.useful` shows it for people.
MAIN:
    let $x: f64 := 0.1;
    std::println.stored($x);
    std::println($x);
    let $y: f16 := 1000;
    std::println($y);
    let $big := 1_000_000 x 1_000_000;
    std::println.stored($big);
    let $s := "x is " ++ to.stored.str.utf8($x);
    std::println.stored($s);
    std::println($s);
    std::println("x is ", $x);
;MAIN
