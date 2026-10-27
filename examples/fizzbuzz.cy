# FizzBuzz.
proc say [let $n: int.sign.32]:
    let $by3 := ($n mod.floor 3) eq 0;
    let $by5 := ($n mod.floor 5) eq 0;
    if $by3 and $by5:
        std::print.stored("FizzBuzz", newln);
    else if $by3:
        std::print.stored("Fizz", newln);
    else if $by5:
        std::print.stored("Buzz", newln);
    else:
        std::print.stored($n, newln);
    end of if
end of proc say

!MAIN:
    let $limit: int.sign.32 := 15;
    var $i = 1;
    while $i at_most $limit:
        say($i);
        change $i to $i plus 1;
    end of while
!end of MAIN
