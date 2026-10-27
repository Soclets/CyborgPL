# FizzBuzz.
procedure say [$n: integer.signed.32_bit]:
    let $by3 := ($n modulo.floor 3) equal_to 0;
    let $by5 := ($n modulo.floor 5) equal_to 0;
    if $by3 and $by5:
        std::print.stored("FizzBuzz", newline);
    end of if
    else if $by3:
        std::print.stored("Fizz", newline);
    end of else if
    else if $by5:
        std::print.stored("Buzz", newline);
    end of else if
    else:
        std::print.stored($n, newline);
    end of else
end of procedure say

!MAIN:
    let $limit: integer.signed.32_bit = 15;
    variable $i := 1;
    while $i at_most $limit:
        say($i);
        change $i = $i plus 1;
    end of while
!end of MAIN
