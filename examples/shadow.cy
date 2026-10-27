# `shadow`: a new variable with an old name, in an inner block or after.
!MAIN:
    let $x := 1;
    if true:
        let shadow $x := 2;
        std::print.shortest($x, newline);
    end of if
    std::print.shortest($x, newline);
    let shadow $x: integer.signed.32_bit = 3;
    std::print.shortest($x, newline);
!end of MAIN
