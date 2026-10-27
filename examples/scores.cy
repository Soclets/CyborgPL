# Score handling: small functions, `if` / `else if`, and clamping a value into a range.
fn is_even [let $n: int.sign.32] -> bool:
    ret ($n mod.floor 2) eq 0;
end of fn is_even

fn clamp [let $n: int.sign.32, let $low: int.sign.32, let $high: int.sign.32] -> int.sign.32:
    if $n lt $low:
        ret $low;
    else if $n gt $high:
        ret $high;
    else:
        ret $n;
    end of if
end of fn clamp

!MAIN:
    let $scores := [72, 95, 38, 100, 61, 87];

    var $count = 0;
    var $total = 0;
    var $evens = 0;
    var $best = 0;

    for $score in lend $scores:
        change $count to $count plus 1;
        change $total to $total plus $score;
        if is_even($score):
            change $evens to $evens plus 1;
        end of if
        if $score gt $best:
            change $best to $score;
        end of if
    end of for

    let $average := $total div.floor $count;
    let $curved := clamp($average plus 10, 0, 100);

    std::print.shortest("scored ", $count, " tests", newln);
    std::print.shortest("total ", $total, ", best ", $best, newln);
    std::print.shortest("even scores: ", $evens, newln);
    std::print.shortest("average ", $average, ", curved ", $curved, newln);

    if $average at_least 90:
        std::print.shortest("grade A", newln);
    else if $average at_least 75:
        std::print.shortest("grade B", newln);
    else:
        std::print.shortest("keep going", newln);
    end of if
!end of MAIN
