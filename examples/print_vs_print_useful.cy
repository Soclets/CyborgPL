# `print.stored` shows a value as stored; `print`, which is `print.shortest`, shows it for people.
MAIN:
    let $x: f64 := 0.1;
    std::print.stored($x, newln);
    std::print($x, newln);
    let $y: f16 := 1000;
    std::print($y, newln);
    let $big := 1_000_000 x 1_000_000;
    std::print.stored($big, newln);
    let $s := "x is " concat to.stored.str.utf8($x);
    std::print.stored($s, newln);
    std::print($s, newln);
    std::print("x is ", $x, newln);
;MAIN
