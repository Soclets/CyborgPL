# `print.stored` shows a value as stored; `print.useful` shows it for people.
!MAIN:
    let $x: float.bin.64 = 0.1;
    std::print.stored($x, newln);
    std::print.shortest($x, newln);
    let $y: float.bin.16 = 1000;
    std::print.shortest($y, newln);
    let $big := 1_000_000 times 1_000_000;
    std::print.stored($big, newln);
    let $s := "x is " concat to.stored.str.utf8($x);
    std::print.stored($s, newln);
    std::print.shortest($s, newln);
    std::print.shortest("x is ", $x, newln);
!end of MAIN
