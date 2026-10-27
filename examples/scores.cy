# Score handling: small functions, `if` / `else if`, and clamping a value into a range.
function is_even [$n: integer.signed.32_bit] -> boolean:
    return ($n modulo.floor 2) equal_to 0;
end of function is_even

function clamp [$n: integer.signed.32_bit, $low: integer.signed.32_bit, $high: integer.signed.32_bit] -> integer.signed.32_bit:
    if $n less_than $low:
        return $low;
    end of if
    else if $n greater_than $high:
        return $high;
    end of else if
    else:
        return $n;
    end of else
end of function clamp

!MAIN:
    let $scores := [72, 95, 38, 100, 61, 87];

    variable $count := 0;
    variable $total := 0;
    variable $evens := 0;
    variable $best := 0;

    for $score in lend $scores:
        change $count = $count plus 1;
        change $total = $total plus $score;
        if is_even($score):
            change $evens = $evens plus 1;
        end of if
        if $score greater_than $best:
            change $best = $score;
        end of if
    end of for

    let $average := $total divide.floor $count;
    let $curved := clamp($average plus 10, 0, 100);

    std::print.shortest("scored ", $count, " tests", newline);
    std::print.shortest("total ", $total, ", best ", $best, newline);
    std::print.shortest("even scores: ", $evens, newline);
    std::print.shortest("average ", $average, ", curved ", $curved, newline);

    if $average at_least 90:
        std::print.shortest("grade A", newline);
    end of if
    else if $average at_least 75:
        std::print.shortest("grade B", newline);
    end of else if
    else:
        std::print.shortest("keep going", newline);
    end of else
!end of MAIN
