# FizzBuzz.
proc say [let $n: i32]:
    let $by3 := ($n mod.floor 3) == 0;
    let $by5 := ($n mod.floor 5) == 0;
    if $by3 & $by5:
        std::println.stored("FizzBuzz");
    else if $by3:
        std::println.stored("Fizz");
    else if $by5:
        std::println.stored("Buzz");
    else:
        std::println.stored($n);
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
