# `print.stored` shows a value as stored; `print.useful` shows it for people.
!MAIN:
    let $x: float.binary.64_bit = 0.1;
    std::print.stored($x, newline);
    std::print.shortest($x, newline);
    let $y: float.binary.16_bit = 1000;
    std::print.shortest($y, newline);
    let $big := 1_000_000 times 1_000_000;
    std::print.stored($big, newline);
    let $s := "x is " concatenate to.stored.string.utf8($x);
    std::print.stored($s, newline);
    std::print.shortest($s, newline);
    std::print.shortest("x is ", $x, newline);
!end of MAIN
