# True and false: `not`, `and`, and comparing a bool.
!MAIN:
    let $'correct?': boolean = true;
    let $'wrong?' := not $'correct?';
    if $'correct?' and (not $'wrong?'):
        std::print.stored("fine", newline);
    end of if
    if $'wrong?' equal_to false:
        std::print.stored($'correct?', " ", $'wrong?', newline);
    end of if
!end of MAIN
