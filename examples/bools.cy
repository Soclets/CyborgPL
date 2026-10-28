# True and false: `not`, `and`, and comparing a bool.
MAIN:
    let $'correct?': bool := true;
    let $'wrong?' := not $'correct?';
    if $'correct?' and (not $'wrong?'):
        std::println.stored("fine");
    ;if
    if $'wrong?' == false:
        std::println.stored($'correct?', " ", $'wrong?');
    ;if
;MAIN
