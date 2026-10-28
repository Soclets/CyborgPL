# True and false: `not`, `and`, and comparing a bool.
MAIN:
    let $'correct?': bool := true;
    let $'wrong?' := not $'correct?';
    if $'correct?' and (not $'wrong?'):
        std::print.stored("fine", newln);
    ;if
    if $'wrong?' == false:
        std::print.stored($'correct?', " ", $'wrong?', newln);
    ;if
;MAIN
