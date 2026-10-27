# True and false: `not`, `and`, and comparing a bool.
!MAIN:
    let $'correct?': bool = true;
    let $'wrong?' := not $'correct?';
    if $'correct?' and (not $'wrong?'):
        std::print.stored("fine", newln);
    end of if
    if $'wrong?' eq false:
        std::print.stored($'correct?', " ", $'wrong?', newln);
    end of if
!end of MAIN
