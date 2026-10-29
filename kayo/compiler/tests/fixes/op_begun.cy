# unknown_operator: a word that begins one of the language's operators (`equal` is `equal_to`).
MAIN:
    let $a: i32 := 1;
    if $a equal 0:
        std::exit(code: 1);
    ;if
;MAIN
