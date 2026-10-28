# FizzBuzz.
proc say [let $n: i32]:
    let $by3 := ($n mod.floor 3) == 0;
    let $by5 := ($n mod.floor 5) == 0;
    if $by3 and $by5:
        std::print.stored("FizzBuzz", newln);
    else if $by3:
        std::print.stored("Fizz", newln);
    else if $by5:
        std::print.stored("Buzz", newln);
    else:
        std::print.stored($n, newln);
    ;if
;proc say

MAIN:
    let $limit: i32 := 15;
    var $i = 1;
    while $i <= $limit:
        say($i);
        change $i += 1;
    ;while
;MAIN
