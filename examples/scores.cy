# Score handling: small functions, `if` / `else if`, and clamping a value into a range.
fn is_even [let $n: i32] -> bool:
    ret ($n mod.floor 2) == 0;
;fn is_even

fn clamp [let $n: i32, let $low: i32, let $high: i32] -> i32:
    if $n < $low:
        ret $low;
    else if $n > $high:
        ret $high;
    else:
        ret $n;
    ;if
;fn clamp

MAIN:
    let $scores := [72, 95, 38, 100, 61, 87];

    var $count = 0;
    var $total = 0;
    var $evens = 0;
    var $best = 0;

    for $score in lend $scores:
        change $count += 1;
        change $total += $score;
        if is_even($score):
            change $evens += 1;
        ;if
        if $score > $best:
            change $best to $score;
        ;if
    ;for

    let $average := $total div.floor $count;
    let $curved := clamp($average + 10, 0, 100);

    std::println("scored ", $count, " tests");
    std::println("total ", $total, ", best ", $best);
    std::println("even scores: ", $evens);
    std::println("average ", $average, ", curved ", $curved);

    if $average >= 90:
        std::println("grade A");
    else if $average >= 75:
        std::println("grade B");
    else:
        std::println("keep going");
    ;if
;MAIN
