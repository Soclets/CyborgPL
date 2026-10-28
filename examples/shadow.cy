# `shadow`: a new variable with an old name, in an inner block or after.
MAIN:
    let $x := 1;
    if true:
        let shadow $x := 2;
        std::println($x);
    ;if
    std::println($x);
    let shadow $x: i32 := 3;
    std::println($x);
;MAIN
