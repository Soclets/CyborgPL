# `shadow`: a new variable with an old name, in an inner block or after.
!MAIN:
    let $x := 1;
    if true:
        let shadow $x := 2;
        std::print.shortest($x, newln);
    end of if
    std::print.shortest($x, newln);
    let shadow $x: int.sign.32 := 3;
    std::print.shortest($x, newln);
!end of MAIN
